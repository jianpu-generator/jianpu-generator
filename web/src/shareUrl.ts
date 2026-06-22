import LZString from 'lz-string'

export interface SharePayload {
  filename: string
  content: string
}

const SHARE_HASH_PREFIX = '#share='

function isSharePayload(value: unknown): value is SharePayload {
  if (typeof value !== 'object' || value === null) return false
  const payload = value as SharePayload
  return (
    typeof payload.filename === 'string' && typeof payload.content === 'string'
  )
}

function parseSharePayloadJson(raw: string): SharePayload | null {
  try {
    const parsed: unknown = JSON.parse(raw)
    return isSharePayload(parsed) ? parsed : null
  } catch {
    return null
  }
}

export function encodeShareHashSuffix(
  filename: string,
  content: string,
): string {
  const payload = JSON.stringify({ filename, content } satisfies SharePayload)
  return LZString.compressToEncodedURIComponent(payload)
}

export function decodeShareHashSuffix(encoded: string): SharePayload | null {
  const decompressed = LZString.decompressFromEncodedURIComponent(encoded)
  if (decompressed != null) {
    const parsed = parseSharePayloadJson(decompressed)
    if (parsed) return parsed
  }

  try {
    return parseSharePayloadJson(decodeURIComponent(encoded))
  } catch {
    return null
  }
}

export function buildShareUrl(filename: string, content: string): string {
  const base = new URL(import.meta.env.BASE_URL, window.location.origin)
  return `${base.href}${SHARE_HASH_PREFIX}${encodeShareHashSuffix(filename, content)}`
}

export function parseShareFromHash(
  hash: string = window.location.hash,
): SharePayload | null {
  if (!hash.startsWith(SHARE_HASH_PREFIX)) return null
  return decodeShareHashSuffix(hash.slice(SHARE_HASH_PREFIX.length))
}

export function clearShareHash(): void {
  const url = new URL(window.location.href)
  url.hash = ''
  history.replaceState(null, '', `${url.pathname}${url.search}`)
}
