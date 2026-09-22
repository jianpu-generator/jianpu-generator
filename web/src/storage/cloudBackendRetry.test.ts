import { describe, expect, it, vi } from 'vitest'
import { DEMO_FILE_NAMES, type FileStoreState } from '../fileStore'
import { createCloudBackend } from './cloudBackend'
import {
  config,
  emptyResponse,
  fetchMock,
  jsonResponse,
  lastCall,
  parsedBody,
} from './cloudBackendTestHelpers'

// forceOverwrite/single-flight/offline-retry/name-collision-retry coverage
// for `cloudBackend.ts` -- split out of `cloudBackend.test.ts` to stay
// under this repo's 400-line cap. See that file for load/createFile/
// saveContent coverage.

describe('createCloudBackend: forceOverwrite()', () => {
  it('realigns the tracked revision to the conflict-reported one, then retries the save', async () => {
    const backend = createCloudBackend(config)
    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'mine' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-a' },
    }

    fetchMock.mockResolvedValueOnce(jsonResponse(409, { currentRevision: 9 }))
    await backend.saveContent(state)
    expect(backend.lastError()).toEqual({
      kind: 'conflict',
      currentRevision: 9,
    })

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 10 }))
    await backend.forceOverwrite(state)

    expect(parsedBody(lastCall().init)).toMatchObject({ expectedRevision: 9 })
    expect(backend.status()).toBe('idle')
    expect(backend.lastError()).toBeNull()
  })
})

describe('createCloudBackend: single-flight save serialization', () => {
  it('never runs two saveContent network calls concurrently', async () => {
    let active = 0
    let maxConcurrent = 0

    fetchMock.mockImplementation(async () => {
      active++
      maxConcurrent = Math.max(maxConcurrent, active)
      await new Promise((resolve) => setTimeout(resolve, 10))
      active--
      return jsonResponse(200, { revision: 1 })
    })

    const backend = createCloudBackend(config)
    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'A' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-a' },
    }

    await Promise.all([backend.saveContent(state), backend.saveContent(state)])

    expect(maxConcurrent).toBe(1)
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })
})

describe('createCloudBackend: offline retry-on-reconnect', () => {
  it('replays the pending save once the browser comes back online', async () => {
    const listeners: Record<string, () => void> = {}
    vi.stubGlobal('window', {
      addEventListener: (event: string, callback: () => void) => {
        listeners[event] = callback
      },
      removeEventListener: () => {},
    })

    const backend = createCloudBackend(config)
    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'new content' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-a' },
    }

    fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'))
    await expect(backend.saveContent(state)).rejects.toThrow('fetch failed')
    expect(backend.status()).toBe('offline')
    expect(backend.lastError()).toEqual({ kind: 'network' })

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 1 }))
    listeners.online?.()
    await vi.waitFor(() => expect(backend.status()).toBe('idle'))

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(parsedBody(lastCall().init)).toMatchObject({
      content: 'new content',
    })

    vi.unstubAllGlobals()
  })
})

describe('createCloudBackend: name-collision retry (create/rename/restore)', () => {
  it('retries createFile once with a freshly recomputed name after a 409 name_taken response', async () => {
    fetchMock
      .mockResolvedValueOnce(jsonResponse(409, { code: 'name_taken' }))
      .mockResolvedValueOnce(
        jsonResponse(200, {
          id: 'new-id',
          name: 'untitled 2.jianpu',
          content: '',
          revision: 0,
          trashedAt: null,
        }),
      )

    const backend = createCloudBackend(config)
    const state: FileStoreState = {
      active: DEMO_FILE_NAMES[0] ?? '',
      userFiles: {},
      bin: {},
      fileIds: {},
    }
    const nextState = await backend.createFile(state)

    expect(fetchMock).toHaveBeenCalledTimes(2)
    const firstBody = JSON.parse(fetchMock.mock.calls[0]?.[1].body as string)
    const secondBody = JSON.parse(fetchMock.mock.calls[1]?.[1].body as string)
    expect(firstBody.name).toBe('untitled.jianpu')
    expect(secondBody.name).toBe('untitled 2.jianpu')
    expect(secondBody.id).toBe(firstBody.id)

    expect(nextState.active).toBe('untitled 2.jianpu')
    expect(nextState.userFiles).toHaveProperty('untitled 2.jianpu')
    expect(nextState.userFiles).not.toHaveProperty('untitled.jianpu')
    expect(backend.status()).toBe('idle')
    expect(backend.lastError()).toBeNull()
  })

  it('retries renameFile once, transparently, on a single name_taken collision', async () => {
    fetchMock
      .mockResolvedValueOnce(jsonResponse(409, { code: 'name_taken' }))
      .mockResolvedValueOnce(emptyResponse(204))

    const backend = createCloudBackend(config)
    const state: FileStoreState = {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'content' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-a' },
    }
    const nextState = await backend.renameFile(state, 'a.jianpu', 'b.jianpu')

    expect(fetchMock).toHaveBeenCalledTimes(2)
    const firstBody = JSON.parse(fetchMock.mock.calls[0]?.[1].body as string)
    const secondBody = JSON.parse(fetchMock.mock.calls[1]?.[1].body as string)
    expect(firstBody.name).toBe('b.jianpu')
    expect(secondBody.name).toBe('b 2.jianpu')
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      'http://localhost:8787/files/id-a/rename',
    )

    expect(nextState.active).toBe('b 2.jianpu')
  })

  it('degrades to an unknown error after a second consecutive name_taken collision', async () => {
    fetchMock
      .mockResolvedValueOnce(jsonResponse(409, { code: 'name_taken' }))
      .mockResolvedValueOnce(jsonResponse(409, { code: 'name_taken' }))

    const backend = createCloudBackend(config)
    const state: FileStoreState = {
      active: DEMO_FILE_NAMES[0] ?? '',
      userFiles: {},
      bin: {},
      fileIds: {},
    }

    await expect(backend.createFile(state)).rejects.toThrow()

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toMatchObject({ kind: 'unknown' })
  })

  it('retries restoreFile once on a name_taken collision against an active file', async () => {
    fetchMock
      .mockResolvedValueOnce(jsonResponse(409, { code: 'name_taken' }))
      .mockResolvedValueOnce(emptyResponse(204))

    const backend = createCloudBackend(config)
    const state: FileStoreState = {
      active: 'original.jianpu',
      userFiles: { 'original.jianpu': 'active content' },
      bin: { 'binned.jianpu': 'binned content' },
      fileIds: { 'original.jianpu': 'id-active', 'binned.jianpu': 'id-binned' },
    }
    const nextState = await backend.restoreFile(state, 'binned.jianpu')

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0]?.[0]).toBe(
      'http://localhost:8787/files/id-binned/restore',
    )
    const firstBody = JSON.parse(fetchMock.mock.calls[0]?.[1].body as string)
    const secondBody = JSON.parse(fetchMock.mock.calls[1]?.[1].body as string)
    expect(firstBody.name).toBe('binned.jianpu')
    expect(secondBody.name).toBe('binned 2.jianpu')

    expect(nextState.userFiles).toHaveProperty('binned 2.jianpu')
    expect(nextState.bin).not.toHaveProperty('binned.jianpu')
  })
})
