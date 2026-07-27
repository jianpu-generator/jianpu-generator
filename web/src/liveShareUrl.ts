const LIVE_HASH_PREFIX = '#live='

// A UUID-shaped room id, deterministically derived (see deriveLiveIdentity)
// so the same file always reproduces the same link — the only thing
// embedded in the copyable viewer link.
const ROOM_ID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

const DEVICE_SECRET_KEY = 'jianpu:live-device-secret:v1'

export interface LiveSharePayload {
  roomId: string
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

function toHex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('')
}

/** Formats 16 bytes as a UUID-shaped string (not a real v4 UUID — no
 * version/variant bits are forced) so a derived room id still satisfies
 * `ROOM_ID_PATTERN` and slots into the existing `#live=` hash format. */
function formatAsRoomId(bytes: Uint8Array): string {
  const hex = toHex(bytes.slice(0, 16))
  return [
    hex.slice(0, 8),
    hex.slice(8, 12),
    hex.slice(12, 16),
    hex.slice(16, 20),
    hex.slice(20, 32),
  ].join('-')
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
    roomId: formatAsRoomId(roomBytes),
    ownerToken: toBase64Url(ownerBytes),
  }
}

export function buildLiveShareUrl(roomId: string): string {
  const base = new URL(import.meta.env.BASE_URL, window.location.origin)
  return `${base.href}${LIVE_HASH_PREFIX}${roomId}`
}

export function parseLiveShareFromHash(
  hash: string = window.location.hash,
): LiveSharePayload | null {
  if (!hash.startsWith(LIVE_HASH_PREFIX)) return null
  const roomId = hash.slice(LIVE_HASH_PREFIX.length)
  return ROOM_ID_PATTERN.test(roomId) ? { roomId } : null
}

export function clearLiveShareHash(): void {
  const url = new URL(window.location.href)
  url.hash = ''
  history.replaceState(null, '', `${url.pathname}${url.search}`)
}
