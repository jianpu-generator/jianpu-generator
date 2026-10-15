import { describe, expect, it } from 'vitest'
import type { FileStoreState } from '../fileStore'
import { createTestBackend, fetchMock } from './cloudBackendTestHelpers'

const state: FileStoreState = {
  active: 'Scratch.jianpu',
  userFiles: {
    'Hymns/Advent/O Come.jianpu': 'a',
    'Hymns/Amazing.jianpu': 'b',
    'Scratch.jianpu': 'c',
  },
  bin: {},
  fileIds: {
    'Hymns/Advent/O Come.jianpu': 'id-come',
    'Hymns/Amazing.jianpu': 'id-amazing',
    'Scratch.jianpu': 'id-scratch',
  },
}

function neverAnswer() {
  fetchMock.mockImplementation(() => new Promise(() => undefined))
}

function queued(
  looper: ReturnType<typeof createTestBackend>['looper'],
  fileId: string,
) {
  const lane = looper.snapshot().queue.lanes.find((l) => l.fileId === fileId)
  return lane?.messages.map(({ message }) => message) ?? []
}

describe('createCloudBackend: folder operations are per-file messages', () => {
  it('moveFolder renames every file inside on its own', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.moveFolder(state, 'Hymns', 'Songs')

    expect(Object.keys(next.userFiles).sort()).toEqual([
      'Scratch.jianpu',
      'Songs/Advent/O Come.jianpu',
      'Songs/Amazing.jianpu',
    ])
    expect(queued(looper, 'id-come')).toEqual([
      { tag: 'rename-file', val: { to: 'Songs/Advent/O Come.jianpu' } },
    ])
    expect(queued(looper, 'id-amazing')).toEqual([
      { tag: 'rename-file', val: { to: 'Songs/Amazing.jianpu' } },
    ])
    expect(queued(looper, 'id-scratch')).toEqual([])
  })

  it('moveFolder onto an existing path changes and queues nothing', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.moveFolder(state, 'Hymns', 'Scratch.jianpu')

    expect(next).toBe(state)
    expect(queued(looper, 'id-come')).toEqual([])
  })

  it('deleteFolder trashes every file inside on its own', async () => {
    const { backend, looper } = createTestBackend()
    neverAnswer()

    const next = await backend.deleteFolder(state, 'Hymns')

    expect(Object.keys(next.bin).sort()).toEqual([
      'Hymns/Advent/O Come.jianpu',
      'Hymns/Amazing.jianpu',
    ])
    expect(queued(looper, 'id-come')).toEqual([{ tag: 'trash-file' }])
    expect(queued(looper, 'id-amazing')).toEqual([{ tag: 'trash-file' }])
    expect(queued(looper, 'id-scratch')).toEqual([])
  })
})
