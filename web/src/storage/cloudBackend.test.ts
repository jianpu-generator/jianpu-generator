import { describe, expect, it } from 'vitest'
import { DEMO_FILE_NAMES, type FileStoreState } from '../fileStore'
import {
  apiErrorResponse,
  createTestBackend,
  fetchMock,
  jsonResponse,
  lastCall,
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

    const backend = createTestBackend().backend
    const state = await backend.load()

    const { url, method, body } = await lastCall()
    expect(url).toBe('http://localhost:8787/files/list')
    expect(method).toBe('POST')
    expect(body).toEqual({ identityToken: 'test-token' })

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

    const { backend, looper } = createTestBackend()
    const state: FileStoreState = {
      active: DEMO_FILE_NAMES[0] ?? '',
      userFiles: {},
      bin: {},
      fileIds: {},
    }
    const nextState = await backend.createFile(state)
    await looper.whenIdle()

    const { url, body } = await lastCall()
    expect(url).toBe('http://localhost:8787/files')
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
    backend: ReturnType<typeof createTestBackend>['backend'],
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

  it('returns once the save is persisted, before it is delivered', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockImplementation(() => new Promise(() => undefined))

    await backend.saveContent(seededState())

    expect(looper.snapshot().summary.totalPending).toBe(1)
    expect(backend.status()).toBe('saving')
  })

  it('sends the tracked revision as expectedRevision and advances it on success', async () => {
    const { backend, looper } = createTestBackend()
    await loadWithRevision(backend, 3)

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 4 }))
    await backend.saveContent(seededState())
    await looper.whenIdle()

    const { url, body } = await lastCall()
    expect(url).toBe('http://localhost:8787/files/id-a/content')
    expect(body).toEqual({
      identityToken: 'test-token',
      content: 'new content',
      expectedRevision: 3,
    })
    expect(backend.status()).toBe('idle')

    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 5 }))
    await backend.saveContent(seededState())
    await looper.whenIdle()
    expect((await lastCall()).body).toMatchObject({ expectedRevision: 4 })
  })

  it('defaults expectedRevision to 0 for a file never seen via load()', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockResolvedValueOnce(jsonResponse(200, { revision: 1 }))

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect((await lastCall()).body).toMatchObject({ expectedRevision: 0 })
  })

  it('on a 409 revision conflict, reports the conflict and keeps the message', async () => {
    const { backend, looper } = createTestBackend()
    await loadWithRevision(backend, 3)
    fetchMock.mockResolvedValueOnce(
      apiErrorResponse(409, { code: 'revision_conflict', currentRevision: 7 }),
    )
    // The conflict makes the sender compare against the server's text.
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        files: [
          {
            id: 'id-a',
            name: 'a.jianpu',
            content: 'theirs',
            revision: 9,
            trashedAt: null,
          },
        ],
      }),
    )

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({
      kind: 'conflict',
      currentRevision: 7,
    })
    expect(looper.snapshot().summary.hasUnsynced).toBe(true)
  })

  it('401 classifies as an auth error and keeps the message', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockResolvedValueOnce(
      apiErrorResponse(401, {
        code: 'unauthorized',
        reason: 'expired',
        failedAt: 0,
        attempts: 3,
      }),
    )

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('error')
    expect(backend.lastError()).toEqual({ kind: 'auth' })
    expect(looper.snapshot().summary.hasUnsynced).toBe(true)
  })

  it('classifies an unexpected non-2xx response as unknown', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockResolvedValueOnce(jsonResponse(500, { message: 'boom' }))

    await backend.saveContent(seededState())
    await looper.whenIdle()

    expect(backend.status()).toBe('offline')
    expect(backend.lastError()).toEqual({ kind: 'network' })
  })
})
