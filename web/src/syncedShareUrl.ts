const SYNCED_HASH_PREFIX = '#synced='

const FILENAME_EXTENSION = '.jianpu'

// Base64url encoding of 8 random bytes (64 bits) is *always* exactly this
// many characters — base64 maps fixed-size byte groups to fixed-size char
// groups, so length depends only on byte count, never byte values. The
// worker only needs an opaque share name, not a UUID shape — 64 bits keeps
// accidental collisions negligible (shares are never deleted, see
// `crates/live-share-worker/src/share_id.rs`) while staying short, since
// every char here lands directly in a copy-pasted URL. Must match
// `SHARE_ID_LENGTH` in that Rust file — do not change one without the
// other.
const SHARE_ID_LENGTH = 11
const SHARE_ID_PATTERN = new RegExp(`^[0-9A-Za-z_-]{${SHARE_ID_LENGTH}}$`)

// Purely cosmetic — separates the share id from the filename so a copied
// link reads clearly. Parsing doesn't need it to find the boundary (the
// share id's length is fixed, see above), so the filename after it is never
// escaped and may contain any character, including further dashes.
const FILENAME_SEPARATOR = '--'

export interface SyncedSharePayload {
  shareId: string
  /** Filename at share-time, carried in the URL purely so the link reads as
   * something a human can recognize (in chat, browser history, etc). Never
   * authoritative — the share's stored doc is the source of truth, so a
   * rename after sharing doesn't invalidate the link, it just makes this
   * cosmetic copy stale. */
  filename?: string
}

export function buildSyncedShareUrl(
  shareId: string,
  filename?: string,
): string {
  const base = new URL(import.meta.env.BASE_URL, window.location.origin)
  const bareName = filename?.endsWith(FILENAME_EXTENSION)
    ? filename.slice(0, -FILENAME_EXTENSION.length)
    : filename
  const namePart = bareName ? `${FILENAME_SEPARATOR}${bareName}` : ''
  return `${base.href}${SYNCED_HASH_PREFIX}${shareId}${namePart}`
}

export function parseSyncedShareFromHash(
  hash: string = window.location.hash,
): SyncedSharePayload | null {
  if (!hash.startsWith(SYNCED_HASH_PREFIX)) return null
  const body = hash.slice(SYNCED_HASH_PREFIX.length)
  const shareId = body.slice(0, SHARE_ID_LENGTH)
  if (!SHARE_ID_PATTERN.test(shareId)) return null
  const rest = body.slice(SHARE_ID_LENGTH)
  if (rest === '') return { shareId }
  if (!rest.startsWith(FILENAME_SEPARATOR)) return null
  const filename = `${rest.slice(FILENAME_SEPARATOR.length)}${FILENAME_EXTENSION}`
  return { shareId, filename }
}

export function clearSyncedShareHash(): void {
  const url = new URL(window.location.href)
  url.hash = ''
  history.replaceState(null, '', `${url.pathname}${url.search}`)
}
