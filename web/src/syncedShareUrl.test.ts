import { describe, expect, it } from 'vitest'
import { parseSyncedShareFromHash } from './syncedShareUrl'

// A well-formed, 11-char share id (matches `SHARE_ID_LENGTH`/`SHARE_ID_PATTERN`
// in `syncedShareUrl.ts`) -- shareIds are generated server-side now (task
// 11), so tests use a fixed sample instead of deriving one.
const SAMPLE_SHARE_ID = 'abcDEF123_-'

describe('syncedShareUrl', () => {
  it('round-trips a share id through the #synced= hash format', () => {
    expect(parseSyncedShareFromHash(`#synced=${SAMPLE_SHARE_ID}`)).toEqual({
      shareId: SAMPLE_SHARE_ID,
    })
  })

  it('rejects a missing #synced= prefix', () => {
    expect(parseSyncedShareFromHash('#share=abc123')).toBeNull()
    expect(parseSyncedShareFromHash('')).toBeNull()
  })

  it('rejects a malformed share id', () => {
    expect(parseSyncedShareFromHash('#synced=not@valid@id')).toBeNull()
    expect(parseSyncedShareFromHash('#synced=')).toBeNull()
  })

  it('parses a --filename suffix after the fixed-length share id, appending .jianpu', () => {
    expect(
      parseSyncedShareFromHash(`#synced=${SAMPLE_SHARE_ID}--My Song`),
    ).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: 'My Song.jianpu',
    })
  })

  it('leaves CJK and other non-ASCII filenames, and any character, unescaped', () => {
    const hash = `#synced=${SAMPLE_SHARE_ID}--快樂天堂 100% A & B -- more`
    expect(parseSyncedShareFromHash(hash)).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: '快樂天堂 100% A & B -- more.jianpu',
    })
  })

  it('omits filename from the payload when no --suffix is present', () => {
    expect(parseSyncedShareFromHash(`#synced=${SAMPLE_SHARE_ID}`)).toEqual({
      shareId: SAMPLE_SHARE_ID,
    })
  })

  it('rejects trailing content that is not a --filename suffix', () => {
    expect(parseSyncedShareFromHash(`#synced=${SAMPLE_SHARE_ID}foo`)).toBeNull()
  })

  it('still rejects a malformed share id even with a --filename suffix', () => {
    expect(parseSyncedShareFromHash('#synced=not@valid@id--foo')).toBeNull()
  })
})
