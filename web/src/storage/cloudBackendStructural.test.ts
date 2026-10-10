import { describe, expect, it } from 'vitest'
import { DEMO_FILE_NAMES, type FileStoreState } from '../fileStore'
import {
  apiErrorResponse,
  callAt,
  createTestBackend,
  emptyResponse,
  fetchMock,
  jsonResponse,
} from './cloudBackendTestHelpers'

const emptyState: FileStoreState = {
  active: DEMO_FILE_NAMES[0] ?? '',
  userFiles: {},
  bin: {},
  fileIds: {},
}

const stateWithFile: FileStoreState = {
  active: 'a.jianpu',
  userFiles: { 'a.jianpu': 'content' },
  bin: { 'binned.jianpu': 'old' },
  fileIds: { 'a.jianpu': 'id-a', 'binned.jianpu': 'id-binned' },
}

const createdFile = (name: string, revision = 0) =>
  jsonResponse(200, { id: 'x', name, content: '', revision, trashedAt: null })

function neverAnswer() {
  fetchMock.mockImplementation(() => new Promise(() => undefined))
}

function queuedTags(
  looper: ReturnType<typeof createTestBackend>['looper'],
  fileId: string,
): string[] {
  const lane = looper.snapshot().queue.lanes.find((l) => l.fileId === fileId)
  return lane?.messages.map(({ message }) => message.tag) ?? []
}

describe('createCloudBackend: structural operations are applied locally at once', () => {
  it('createFile returns the new file and queues a create-file message', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.createFile(emptyState)

    expect(next.active).toBe('untitled.jianpu')
    const id = next.fileIds['untitled.jianpu'] ?? ''
    expect(queuedTags(looper, id)).toEqual(['create-file'])
  })

  it('renameFile returns the renamed state and queues a rename-file message', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.renameFile(stateWithFile, 'a.jianpu', 'b.jianpu')

    expect(next.userFiles).toHaveProperty('b.jianpu')
    expect(queuedTags(looper, 'id-a')).toEqual(['rename-file'])
  })

  it('deleteFile moves the file to the bin and queues a trash-file message', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.deleteFile(stateWithFile, 'a.jianpu')

    expect(next.bin).toHaveProperty('a.jianpu')
    expect(queuedTags(looper, 'id-a')).toEqual(['trash-file'])
  })

  it('restoreFile brings the file back and queues a restore-file message', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.restoreFile(stateWithFile, 'binned.jianpu')

    expect(next.userFiles).toHaveProperty('binned.jianpu')
    expect(queuedTags(looper, 'id-binned')).toEqual(['restore-file'])
  })
})

describe('createCloudBackend: ordering within a file', () => {
  it('keeps a save behind the offline creation of the same file', async () => {
    const { backend, looper } = createTestBackend()
    fetchMock.mockRejectedValue(new TypeError('Failed to fetch'))

    const created = await backend.createFile(emptyState)
    await backend.saveContent({
      ...created,
      userFiles: { ...created.userFiles, 'untitled.jianpu': 'edited' },
    })
    await looper.whenIdle()

    const id = created.fileIds['untitled.jianpu'] ?? ''
    expect(queuedTags(looper, id)[0]).toBe('create-file')
    expect(backend.status()).toBe('offline')
  })

  it('sends the create before the save once the network is back', async () => {
    const { backend, looper, wake, advance } = createTestBackend()
    fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'))

    const created = await backend.createFile(emptyState)
    await backend.saveContent({
      ...created,
      userFiles: { ...created.userFiles, 'untitled.jianpu': 'edited' },
    })
    await looper.whenIdle()

    fetchMock
      .mockResolvedValueOnce(createdFile('untitled.jianpu', 1))
      .mockResolvedValueOnce(jsonResponse(200, { revision: 2 }))
    advance(120_000)
    wake()
    await looper.whenIdle()

    expect((await callAt(-2)).url).toBe('http://localhost:8787/files')
    expect((await callAt(-1)).url).toMatch(/\/files\/.+\/content$/)
    expect((await callAt(-1)).body).toMatchObject({
      content: 'edited',
      expectedRevision: 1,
    })
    expect(backend.status()).toBe('idle')
  })
})

describe('createCloudBackend: a name collision found at delivery time', () => {
  it('renames the local file to the name the outbox picked', async () => {
    const { backend, looper } = createTestBackend()
    const renames: Array<{ from: string; to: string }> = []
    backend.onLocalRename(({ from, to }) => renames.push({ from, to }))
    fetchMock
      .mockResolvedValueOnce(apiErrorResponse(409, { code: 'name_taken' }))
      .mockResolvedValueOnce(createdFile('untitled 2.jianpu'))

    await backend.createFile(emptyState)
    await looper.whenIdle()

    expect((await callAt(0)).body.name).toBe('untitled.jianpu')
    expect((await callAt(1)).body.name).toBe('untitled 2.jianpu')
    expect(renames).toEqual([
      { from: 'untitled.jianpu', to: 'untitled 2.jianpu' },
    ])
    expect(backend.status()).toBe('idle')
  })

  it('renames the local file after a rename collides', async () => {
    const { backend, looper } = createTestBackend()
    const renames: Array<{ from: string; to: string }> = []
    backend.onLocalRename(({ from, to }) => renames.push({ from, to }))
    fetchMock
      .mockResolvedValueOnce(apiErrorResponse(409, { code: 'name_taken' }))
      .mockResolvedValueOnce(emptyResponse(204))

    await backend.renameFile(stateWithFile, 'a.jianpu', 'b.jianpu')
    await looper.whenIdle()

    expect((await callAt(1)).body.name).toBe('b 2.jianpu')
    expect(renames).toEqual([{ from: 'b.jianpu', to: 'b 2.jianpu' }])
  })
})
