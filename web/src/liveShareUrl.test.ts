import { describe, expect, it } from 'vitest'
import { deriveLiveIdentity, parseLiveShareFromHash } from './liveShareUrl'

const SECRET_A = 'secret-a'
const SECRET_B = 'secret-b'

describe('liveShareUrl', () => {
  it('round-trips a derived room id through the #live= hash format', async () => {
    const { roomId } = await deriveLiveIdentity(SECRET_A, 'some-file-id')
    expect(parseLiveShareFromHash(`#live=${roomId}`)).toEqual({ roomId })
  })

  it('rejects a missing #live= prefix', () => {
    expect(parseLiveShareFromHash('#share=abc123')).toBeNull()
    expect(parseLiveShareFromHash('')).toBeNull()
  })

  it('rejects a malformed room id', () => {
    expect(parseLiveShareFromHash('#live=not@valid@id')).toBeNull()
    expect(parseLiveShareFromHash('#live=')).toBeNull()
  })

  it('derives the same identity for the same secret and file id', async () => {
    const a = await deriveLiveIdentity(SECRET_A, 'file-a')
    const b = await deriveLiveIdentity(SECRET_A, 'file-a')
    expect(a).toEqual(b)
  })

  it('derives distinct identities for distinct file ids', async () => {
    const a = await deriveLiveIdentity(SECRET_A, 'file-a')
    const b = await deriveLiveIdentity(SECRET_A, 'file-b')
    expect(a.roomId).not.toEqual(b.roomId)
    expect(a.ownerToken).not.toEqual(b.ownerToken)
  })

  it('derives distinct identities for distinct device secrets', async () => {
    const a = await deriveLiveIdentity(SECRET_A, 'file-a')
    const b = await deriveLiveIdentity(SECRET_B, 'file-a')
    expect(a.roomId).not.toEqual(b.roomId)
    expect(a.ownerToken).not.toEqual(b.ownerToken)
  })

  it('derives a url-safe owner token', async () => {
    const { ownerToken } = await deriveLiveIdentity(SECRET_A, 'file-a')
    expect(ownerToken).toMatch(/^[A-Za-z0-9_-]+$/)
  })

  it('parses a --filename suffix after the fixed-length room id, appending .jianpu', async () => {
    const { roomId } = await deriveLiveIdentity(SECRET_A, 'some-file-id')
    expect(parseLiveShareFromHash(`#live=${roomId}--My Song`)).toEqual({
      roomId,
      filename: 'My Song.jianpu',
    })
  })

  it('leaves CJK and other non-ASCII filenames, and any character, unescaped', async () => {
    const { roomId } = await deriveLiveIdentity(SECRET_A, 'some-file-id')
    const hash = `#live=${roomId}--快樂天堂 100% A & B -- more`
    expect(parseLiveShareFromHash(hash)).toEqual({
      roomId,
      filename: '快樂天堂 100% A & B -- more.jianpu',
    })
  })

  it('omits filename from the payload when no --suffix is present', async () => {
    const { roomId } = await deriveLiveIdentity(SECRET_A, 'some-file-id')
    expect(parseLiveShareFromHash(`#live=${roomId}`)).toEqual({ roomId })
  })

  it('rejects trailing content that is not a --filename suffix', async () => {
    const { roomId } = await deriveLiveIdentity(SECRET_A, 'some-file-id')
    expect(parseLiveShareFromHash(`#live=${roomId}foo`)).toBeNull()
  })

  it('still rejects a malformed room id even with a --filename suffix', () => {
    expect(parseLiveShareFromHash('#live=not@valid@id--foo')).toBeNull()
  })
})
