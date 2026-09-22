import { describe, expect, it } from 'vitest'
import { DEMO_FILE_NAMES, type FileStoreState } from '../fileStore'
import { createCloudBackend } from './cloudBackend'
import {
  config,
  fetchMock,
  jsonResponse,
  lastCall,
  parsedBody,
} from './cloudBackendTestHelpers'

// See also `cloudBackendRetry.test.ts` for forceOverwrite/single-flight/
// offline-retry/name-collision-retry coverage -- split out to stay under
// this repo's 400-line cap.

describe('createCloudBackend: load()', () => {
  it('POSTs identityToken to /files/list and partitions rows by trashedAt', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, {
        files: [
          {
            id: 'id-a',
            name: 'a.jianpu',
            content: '1 2 3',
            revision: 3,
            trashedAt: null,
          },
          {
            id: 'id-b',
            name: 'b.jianpu',
            content: 'x',
            revision: 0,
            trashedAt: 12345,
          },
        ],
      }),
    )

    const backend = createCloudBackend(config)
    const state = await backend.load()

    const { url, init } = lastCall()
    expect(url).toBe('http://localhost:8787/files/list')
    expect(init.method).toBe('POST')
    expect(parsedBody(init)).toEqual({ identityToken: 'test-token' })

    expect(state.userFiles).toEqual({ 'a.jianpu': '1 2 3' })
    expect(state.bin).toEqual({ 'b.jianpu': 'x' })
    expect(state.fileIds).toEqual({ 'a.jianpu': 'id-a', 'b.jianpu': 'id-b' })
  })
})

describe('createCloudBackend: createFile()', () => {
  it('POSTs the generated id/name/content to /files', async () => {
    fetchMock.mockResolvedValue(
      jsonResponse(200, {
        id: 'whatever',
        name: 'untitled.jianpu',
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

    const { url, init } = lastCall()
    expect(url).toBe('http://localhost:8787/files')
    const body = parsedBody(init)
    expect(body.identityToken).toBe('test-token')
    expect(body.name).toBe('untitled.jianpu')
    expect(typeof body.id).toBe('string')
    expect(nextState.active).toBe('untitled.jianpu')
  })
})

describe('createCloudBackend: saveContent()', () => {
  function seededState(): FileStoreState {
    return {
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'new content' },
      bin: {},
      fileIds: { 'a.jianpu': 'id-a' },
    }
  }

  async function loadWithRevision(
    backend: ReturnType<typeof createCloudBackend>,
    revision: number,
  ): Promise<void> {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        files: [
          {
            id: 'id-a',
            name: 'a.jianpu',
            content: 'old content',
            revision,
            trashedAt: null,
          },
        ],
      }),
    )
    await backend.load()
  }

  it('sends the tracked revision as expectedRevision and advances it on success', async () => {
    const backend = createCloudBackend(config)
    await loadWithRevision(backend, 3)

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 4 }))
    await backend.saveContent(seededState())

    const { url, init } = lastCall()
    expect(url).toBe('http://localhost:8787/files/id-a/content')
    expect(parsedBody(init)).toEqual({
      identityToken: 'test-token',
      content: 'new content',
      expectedRevision: 3,
    })
    expect(backend.status()).toBe('idle')

    // A second save should now send the advanced revision.
    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 5 }))
    await backend.saveContent(seededState())
    expect(parsedBody(lastCall().init)).toMatchObject({ expectedRevision: 4 })
  })

  it('defaults expectedRevision to 0 for a file never seen via load()', async () => {
    const backend = createCloudBackend(config)
    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 1 }))

    await backend.saveContent(seededState())

    expect(parsedBody(lastCall().init)).toMatchObject({ expectedRevision: 0 })
  })

  it('on a 409 revision-conflict response, sets lastError without throwing and leaves the revision unadvanced', async () => {
    const backend = createCloudBackend(config)
    await loadWithRevision(backend, 3)

    fetchMock.mockResolvedValueOnce(jsonResponse(409, { currentRevision: 7 }))

    await expect(backend.saveContent(seededState())).resolves.toBeUndefined()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({
      kind: 'conflict',
      currentRevision: 7,
    })

    // The revision map wasn't advanced to 7 -- a follow-up save (without
    // going through forceOverwrite) still uses the pre-conflict revision.
    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 4 }))
    await backend.saveContent(seededState())
    expect(parsedBody(lastCall().init)).toMatchObject({ expectedRevision: 3 })
  })

  it('401 classifies as an auth error and rejects', async () => {
    const backend = createCloudBackend(config)
    fetchMock.mockResolvedValueOnce(jsonResponse(401, { reason: 'expired' }))

    await expect(backend.saveContent(seededState())).rejects.toThrow()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({ kind: 'auth' })
  })

  it('classifies an unexpected non-2xx response as unknown', async () => {
    const backend = createCloudBackend(config)
    fetchMock.mockResolvedValueOnce(jsonResponse(500, { message: 'boom' }))

    await expect(backend.saveContent(seededState())).rejects.toThrow()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toMatchObject({ kind: 'unknown' })
  })
})
