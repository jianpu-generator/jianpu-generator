import { beforeAll, describe, expect, it } from 'vitest'
import {
  createFile,
  deleteFile,
  deleteFolder,
  duplicateFile,
  type FileStoreState,
  moveFolder,
  renameFile,
  restoreFile,
} from './fileStore'
import { deserializeFileStore } from './fileStorePersistence'
import { fileTreeRows } from './fileStoreQueries'
import { loadWasmForTests } from './testing/loadWasmForTests'

beforeAll(loadWasmForTests)

function makeState(overrides: Partial<FileStoreState>): FileStoreState {
  return {
    active: 'a.jianpu',
    userFiles: {},
    bin: {},
    fileIds: {},
    ...overrides,
  }
}

describe('file store operations delegated to Rust', () => {
  it('creates a named file inside a folder', () => {
    const state = makeState({
      userFiles: { 'Hymns/a.jianpu': 'x' },
      fileIds: { 'Hymns/a.jianpu': 'id-1' },
    })
    const next = createFile(state, 'Hymns', 'Psalm')
    expect(next.active).toBe('Hymns/Psalm.jianpu')
  })

  it('duplicates in the same folder', () => {
    const state = makeState({
      active: 'A/b.jianpu',
      userFiles: { 'A/b.jianpu': 'content' },
      fileIds: { 'A/b.jianpu': 'id-b' },
    })
    expect(duplicateFile(state).active).toBe('A/b 2.jianpu')
  })

  it('moves a file between folders keeping its id', () => {
    const state = makeState({
      userFiles: { 'a.jianpu': 'content' },
      fileIds: { 'a.jianpu': 'id-a' },
    })
    const next = renameFile(state, 'a.jianpu', 'X/Y/a')
    expect(next.userFiles).toEqual({ 'X/Y/a.jianpu': 'content' })
    expect(next.fileIds['X/Y/a.jianpu']).toBe('id-a')
  })

  it('returns the same state object when an operation is rejected', () => {
    const state = makeState({
      userFiles: { 'a.jianpu': '1', 'b.jianpu': '2' },
      fileIds: { 'a.jianpu': 'id-a', 'b.jianpu': 'id-b' },
    })
    expect(renameFile(state, 'a.jianpu', 'b.jianpu')).toBe(state)
  })

  it('moves a folder as a batch and reports every change', () => {
    const state = makeState({
      userFiles: { 'A/x.jianpu': '1', 'A/B/y.jianpu': '2' },
      fileIds: { 'A/x.jianpu': 'id-x', 'A/B/y.jianpu': 'id-y' },
      folders: ['A/Empty'],
    })
    const result = moveFolder(state, 'A', 'Z')
    expect(result.ok).toBe(true)
    if (!result.ok) return
    expect(Object.keys(result.state.userFiles).sort()).toEqual([
      'Z/B/y.jianpu',
      'Z/x.jianpu',
    ])
    expect(result.state.folders).toEqual(['Z/Empty'])
    expect(result.state.fileIds['Z/x.jianpu']).toBe('id-x')
    expect(result.changes).toHaveLength(3)
  })

  it('rejects a colliding folder move without changing anything', () => {
    const state = makeState({
      userFiles: { 'A/x.jianpu': '1', 'B/y.jianpu': '2' },
      fileIds: { 'A/x.jianpu': 'id-x', 'B/y.jianpu': 'id-y' },
    })
    const result = moveFolder(state, 'A', 'B')
    expect(result.ok).toBe(false)
    if (result.ok) return
    expect(result.failure.kind).toBe('collision')
  })

  it('deletes a folder to the bin and restores a file into the recreated folder', () => {
    const state = makeState({
      userFiles: { 'A/B/y.jianpu': '2' },
      fileIds: { 'A/B/y.jianpu': 'id-y' },
    })
    const deleted = deleteFolder(state, 'A')
    expect(deleted.userFiles).toEqual({})
    expect(deleted.bin).toEqual({ 'A/B/y.jianpu': '2' })
    const restored = restoreFile(deleted, 'A/B/y.jianpu')
    expect(restored.userFiles).toEqual({ 'A/B/y.jianpu': '2' })
    expect(restored.fileIds['A/B/y.jianpu']).toBe('id-y')
  })

  it('bins a single file under its full path', () => {
    const state = makeState({
      userFiles: { 'A/y.jianpu': '2' },
      fileIds: { 'A/y.jianpu': 'id-y' },
    })
    expect(deleteFile(state, 'A/y.jianpu').bin).toEqual({ 'A/y.jianpu': '2' })
  })

  it('lists the tree as sorted display rows', () => {
    const state = makeState({
      userFiles: { 'b.jianpu': '', 'A/z.jianpu': '' },
      folders: ['Empty'],
    })
    expect(
      fileTreeRows(state).map((row) => [row.path, row.depth, row.isFolder]),
    ).toEqual([
      ['A', 0, true],
      ['A/z.jianpu', 1, false],
      ['Empty', 0, true],
      ['b.jianpu', 0, false],
    ])
  })
})

describe('persistence of the optional folders field', () => {
  it('loads an old store without folders', () => {
    const state = deserializeFileStore(
      JSON.stringify({
        active: 'a.jianpu',
        userFiles: { 'a.jianpu': 'x' },
        bin: {},
        fileIds: { 'a.jianpu': 'id-a' },
      }),
    )
    expect(state.folders).toBeUndefined()
    expect(state.userFiles).toEqual({ 'a.jianpu': 'x' })
  })

  it('keeps folders and a nested active file when present', () => {
    const state = deserializeFileStore(
      JSON.stringify({
        active: 'A/a.jianpu',
        userFiles: { 'A/a.jianpu': 'x' },
        bin: {},
        fileIds: { 'A/a.jianpu': 'id-a' },
        folders: ['A/Empty', 3],
      }),
    )
    expect(state.folders).toEqual(['A/Empty'])
    expect(state.active).toBe('A/a.jianpu')
  })
})
