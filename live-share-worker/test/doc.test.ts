import { describe, expect, it } from 'vitest'
import { applyWrite, EMPTY_DOC, type StoredDoc, toPublicDoc } from '../src/doc'

describe('toPublicDoc', () => {
  it('reads a share with no KV entry yet the same as one that was shared then stopped', () => {
    expect(toPublicDoc(null)).toEqual(EMPTY_DOC)
  })

  it('never leaks ownerToken to the caller', () => {
    const stored: StoredDoc = {
      ownerToken: 'secret',
      ended: false,
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
    }
    expect(toPublicDoc(stored)).toEqual({
      ended: false,
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
    })
  })
})

describe('applyWrite', () => {
  it("lets the first update on a share claim ownership and store the doc", () => {
    const result = applyWrite(null, {
      type: 'update',
      ownerToken: 'token-a',
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
    })
    expect(result).toEqual({
      ownerToken: 'token-a',
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
      ended: false,
    })
  })

  it('rejects an update from a token that does not match the stored owner', () => {
    const existing: StoredDoc = {
      ownerToken: 'token-a',
      ended: false,
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
    }
    const result = applyWrite(existing, {
      type: 'update',
      ownerToken: 'token-b',
      filename: 'tampered.jianpu',
      content: '[M] 2',
      revision: 2,
    })
    expect(result).toBe('forbidden')
  })

  it('overwrites content on a later update from the same owner', () => {
    const existing: StoredDoc = {
      ownerToken: 'token-a',
      ended: false,
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
    }
    const result = applyWrite(existing, {
      type: 'update',
      ownerToken: 'token-a',
      filename: 'song.jianpu',
      content: '[M] 2',
      revision: 2,
    })
    expect(result).toEqual({
      ownerToken: 'token-a',
      filename: 'song.jianpu',
      content: '[M] 2',
      revision: 2,
      ended: false,
    })
  })

  it('marks the share ended on stop, keeping the doc so a later update reproduces it', () => {
    const existing: StoredDoc = {
      ownerToken: 'token-a',
      ended: false,
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
    }
    const result = applyWrite(existing, { type: 'stop', ownerToken: 'token-a' })
    expect(result).toEqual({ ...existing, ended: true })
  })

  it('rejects a stop from a non-owner token', () => {
    const existing: StoredDoc = {
      ownerToken: 'token-a',
      ended: false,
      filename: 'song.jianpu',
      content: '[M] 1',
      revision: 1,
    }
    const result = applyWrite(existing, { type: 'stop', ownerToken: 'token-b' })
    expect(result).toBe('forbidden')
  })
})
