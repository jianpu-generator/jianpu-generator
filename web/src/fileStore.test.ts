import { describe, expect, it } from 'vitest'
import { type FileStoreState, mergeBackendResult } from './fileStore'

function makeState(overrides: Partial<FileStoreState>): FileStoreState {
  return {
    active: 'a.jianpu',
    userFiles: {},
    bin: {},
    fileIds: {},
    ...overrides,
  }
}

describe('mergeBackendResult', () => {
  it('preserves a concurrent edit to an unrelated file made after base was read', () => {
    const base = makeState({
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'original a', 'b.jianpu': 'original b' },
      fileIds: { 'a.jianpu': 'id-a', 'b.jianpu': 'id-b' },
    })
    // createFile's async result: adds 'untitled.jianpu', computed from `base`.
    const next = makeState({
      active: 'untitled.jianpu',
      userFiles: {
        'a.jianpu': 'original a',
        'b.jianpu': 'original b',
        'untitled.jianpu': 'template',
      },
      fileIds: { 'a.jianpu': 'id-a', 'b.jianpu': 'id-b', 'untitled.jianpu': 'id-new' },
    })
    // While createFile was in flight, the user edited 'b.jianpu'.
    const prev = makeState({
      active: 'b.jianpu',
      userFiles: { 'a.jianpu': 'original a', 'b.jianpu': 'edited b' },
      fileIds: { 'a.jianpu': 'id-a', 'b.jianpu': 'id-b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.userFiles['b.jianpu']).toBe('edited b')
    expect(result.userFiles['untitled.jianpu']).toBe('template')
    expect(result.active).toBe('untitled.jianpu')
    expect(result.fileIds['untitled.jianpu']).toBe('id-new')
  })

  it('carries a live edit to the renamed file into the new name instead of the stale snapshot', () => {
    const base = makeState({
      active: 'old.jianpu',
      userFiles: { 'old.jianpu': 'original' },
      fileIds: { 'old.jianpu': 'id-1' },
    })
    const next = makeState({
      active: 'new.jianpu',
      userFiles: { 'new.jianpu': 'original' },
      fileIds: { 'new.jianpu': 'id-1' },
    })
    // User kept typing into 'old.jianpu' while the rename was in flight.
    const prev = makeState({
      active: 'old.jianpu',
      userFiles: { 'old.jianpu': 'original plus more' },
      fileIds: { 'old.jianpu': 'id-1' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.userFiles['new.jianpu']).toBe('original plus more')
    expect(result.userFiles['old.jianpu']).toBeUndefined()
    expect(result.active).toBe('new.jianpu')
  })

  it('does not force-switch active if the user already navigated away during the await', () => {
    const base = makeState({
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'content a' },
    })
    const next = makeState({
      active: 'untitled.jianpu',
      userFiles: { 'a.jianpu': 'content a', 'untitled.jianpu': 'template' },
      fileIds: { 'untitled.jianpu': 'id-new' },
    })
    const prev = makeState({
      active: 'b.jianpu',
      userFiles: { 'a.jianpu': 'content a', 'b.jianpu': 'content b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.active).toBe('untitled.jianpu')
    expect(result.userFiles['b.jianpu']).toBe('content b')
  })

  it('moves content from userFiles to bin on delete without losing unrelated edits', () => {
    const base = makeState({
      active: 'a.jianpu',
      userFiles: { 'a.jianpu': 'content a', 'b.jianpu': 'content b' },
    })
    const next = makeState({
      active: 'b.jianpu',
      userFiles: { 'b.jianpu': 'content b' },
      bin: { 'a.jianpu': 'content a' },
    })
    const prev = makeState({
      active: 'b.jianpu',
      userFiles: { 'a.jianpu': 'content a', 'b.jianpu': 'edited b' },
    })

    const result = mergeBackendResult(prev, base, next)

    expect(result.userFiles['a.jianpu']).toBeUndefined()
    expect(result.bin['a.jianpu']).toBe('content a')
    expect(result.userFiles['b.jianpu']).toBe('edited b')
  })
})
