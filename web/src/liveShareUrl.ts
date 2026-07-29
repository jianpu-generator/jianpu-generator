const LIVE_HASH_PREFIX = '#live='

const FILENAME_EXTENSION = '.jianpu'

// Base64url encoding of 8 random/derived bytes (64 bits) is *always* exactly
// this many characters — base64 maps fixed-size byte groups to fixed-size
// char groups, so length depends only on byte count, never byte values.
// PartyKit only needs an opaque room name, not a UUID shape — 64 bits keeps
// accidental collisions negligible (rooms are never deleted, see
// server.ts) while staying short, since every char here lands directly in
// a copy-pasted URL.
const ROOM_ID_LENGTH = 11
const ROOM_ID_PATTERN = new RegExp(`^[0-9A-Za-z_-]{${ROOM_ID_LENGTH}}$`)

// Purely cosmetic — separates the room id from the filename so a copied
// link reads clearly. Parsing doesn't need it to find the boundary (the
// room id's length is fixed, see above), so the filename after it is never
// escaped and may contain any character, including further dashes.
const FILENAME_SEPARATOR = '--'

const DEVICE_SECRET_KEY = 'jianpu:live-device-secret:v1'

export interface LiveSharePayload {
  roomId: string
  /** Filename at share-time, carried in the URL purely so the link reads as
   * something a human can recognize (in chat, browser history, etc). Never
   * authoritative — the room's live `sync`/`update` messages are the source
   * of truth, so a rename after sharing doesn't invalidate the link, it just
   * makes this cosmetic copy stale. */
  filename?: string
}

export interface LiveIdentity {
  roomId: string
  ownerToken: string
}

function toBase64Url(bytes: Uint8Array): string {
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

/** Random per-browser secret that never leaves the device — mixed with a
 * file's stable `fileId` to derive that file's room id and owner token, so
 * the same file always reproduces the same live link without persisting a
 * roomId/ownerToken pair per file. Kept separate from `deriveLiveIdentity`
 * so that function stays a pure, unit-testable derivation. */
export function getOrCreateDeviceSecret(): string {
  const existing = localStorage.getItem(DEVICE_SECRET_KEY)
  if (existing) return existing
  const secret = toBase64Url(crypto.getRandomValues(new Uint8Array(32)))
  localStorage.setItem(DEVICE_SECRET_KEY, secret)
  return secret
}

async function hmacSha256(
  secret: string,
  message: string,
): Promise<Uint8Array> {
  const key = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  )
  const signature = await crypto.subtle.sign(
    'HMAC',
    key,
    new TextEncoder().encode(message),
  )
  return new Uint8Array(signature)
}

/** Derives a stable `{roomId, ownerToken}` pair for a file from a device
 * secret (see `getOrCreateDeviceSecret`) and the file's `fileId`.
 * Deterministic: the same file + secret always reproduces the same live
 * link, so stopping and re-starting a live session doesn't hand out a new
 * one. The ownerToken can't be derived from the (publicly shared) roomId
 * alone — both are independent HMAC outputs keyed by the same secret, and
 * the secret itself is never sent anywhere. */
export async function deriveLiveIdentity(
  secret: string,
  fileId: string,
): Promise<LiveIdentity> {
  const [roomBytes, ownerBytes] = await Promise.all([
    hmacSha256(secret, `room:${fileId}`),
    hmacSha256(secret, `owner:${fileId}`),
  ])
  return {
    roomId: toBase64Url(roomBytes.slice(0, 8)),
    ownerToken: toBase64Url(ownerBytes),
  }
}

export function buildLiveShareUrl(roomId: string, filename?: string): string {
  const base = new URL(import.meta.env.BASE_URL, window.location.origin)
  const bareName = filename?.endsWith(FILENAME_EXTENSION)
    ? filename.slice(0, -FILENAME_EXTENSION.length)
    : filename
  const namePart = bareName ? `${FILENAME_SEPARATOR}${bareName}` : ''
  return `${base.href}${LIVE_HASH_PREFIX}${roomId}${namePart}`
}

export function parseLiveShareFromHash(
  hash: string = window.location.hash,
): LiveSharePayload | null {
  if (!hash.startsWith(LIVE_HASH_PREFIX)) return null
  const body = hash.slice(LIVE_HASH_PREFIX.length)
  const roomId = body.slice(0, ROOM_ID_LENGTH)
  if (!ROOM_ID_PATTERN.test(roomId)) return null
  const rest = body.slice(ROOM_ID_LENGTH)
  if (rest === '') return { roomId }
  if (!rest.startsWith(FILENAME_SEPARATOR)) return null
  const filename = `${rest.slice(FILENAME_SEPARATOR.length)}${FILENAME_EXTENSION}`
  return { roomId, filename }
}

export function clearLiveShareHash(): void {
  const url = new URL(window.location.href)
  url.hash = ''
  history.replaceState(null, '', `${url.pathname}${url.search}`)
}
