import { describe, expect, it } from 'vitest'
import {
  deriveSyncedShareIdentity,
  parseSyncedShareFromHash,
} from './syncedShareUrl'

const SECRET_A = 'secret-a'
const SECRET_B = 'secret-b'

describe('syncedShareUrl', () => {
  it('round-trips a derived share id through the #synced= hash format', async () => {
    const { shareId } = await deriveSyncedShareIdentity(
      SECRET_A,
      'some-file-id',
    )
    expect(parseSyncedShareFromHash(`#synced=${shareId}`)).toEqual({ shareId })
  })

  it('rejects a missing #synced= prefix', () => {
    expect(parseSyncedShareFromHash('#share=abc123')).toBeNull()
    expect(parseSyncedShareFromHash('')).toBeNull()
  })

  it('rejects a malformed share id', () => {
    expect(parseSyncedShareFromHash('#synced=not@valid@id')).toBeNull()
    expect(parseSyncedShareFromHash('#synced=')).toBeNull()
  })

  it('derives the same identity for the same secret and file id', async () => {
    const a = await deriveSyncedShareIdentity(SECRET_A, 'file-a')
    const b = await deriveSyncedShareIdentity(SECRET_A, 'file-a')
    expect(a).toEqual(b)
  })

  it('derives distinct identities for distinct file ids', async () => {
    const a = await deriveSyncedShareIdentity(SECRET_A, 'file-a')
    const b = await deriveSyncedShareIdentity(SECRET_A, 'file-b')
    expect(a.shareId).not.toEqual(b.shareId)
    expect(a.ownerToken).not.toEqual(b.ownerToken)
  })

  it('derives distinct identities for distinct device secrets', async () => {
    const a = await deriveSyncedShareIdentity(SECRET_A, 'file-a')
    const b = await deriveSyncedShareIdentity(SECRET_B, 'file-a')
    expect(a.shareId).not.toEqual(b.shareId)
    expect(a.ownerToken).not.toEqual(b.ownerToken)
  })

  it('derives a url-safe owner token', async () => {
    const { ownerToken } = await deriveSyncedShareIdentity(SECRET_A, 'file-a')
    expect(ownerToken).toMatch(/^[A-Za-z0-9_-]+$/)
  })

  it('parses a --filename suffix after the fixed-length share id, appending .jianpu', async () => {
    const { shareId } = await deriveSyncedShareIdentity(
      SECRET_A,
      'some-file-id',
    )
    expect(parseSyncedShareFromHash(`#synced=${shareId}--My Song`)).toEqual({
      shareId,
      filename: 'My Song.jianpu',
    })
  })

  it('leaves CJK and other non-ASCII filenames, and any character, unescaped', async () => {
    const { shareId } = await deriveSyncedShareIdentity(
      SECRET_A,
      'some-file-id',
    )
    const hash = `#synced=${shareId}--快樂天堂 100% A & B -- more`
    expect(parseSyncedShareFromHash(hash)).toEqual({
      shareId,
      filename: '快樂天堂 100% A & B -- more.jianpu',
    })
  })

  it('omits filename from the payload when no --suffix is present', async () => {
    const { shareId } = await deriveSyncedShareIdentity(
      SECRET_A,
      'some-file-id',
    )
    expect(parseSyncedShareFromHash(`#synced=${shareId}`)).toEqual({ shareId })
  })

  it('rejects trailing content that is not a --filename suffix', async () => {
    const { shareId } = await deriveSyncedShareIdentity(
      SECRET_A,
      'some-file-id',
    )
    expect(parseSyncedShareFromHash(`#synced=${shareId}foo`)).toBeNull()
  })

  it('still rejects a malformed share id even with a --filename suffix', () => {
    expect(parseSyncedShareFromHash('#synced=not@valid@id--foo')).toBeNull()
  })
})
