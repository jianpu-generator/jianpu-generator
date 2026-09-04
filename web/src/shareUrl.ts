import LZString from 'lz-string'
import { compress_share_payload, decompress_share_payload } from './jianpuWasm'
import { ensureWasmInit } from './wasmInit'

export interface SharePayload {
  filename: string
  content: string
}

const SHARE_HASH_PREFIX = '#share='

function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

function base64UrlToBytes(value: string): Uint8Array | null {
  if (!/^[A-Za-z0-9_-]*$/.test(value)) return null
  const base64 = value.replace(/-/g, '+').replace(/_/g, '/')
  const padded = base64 + '='.repeat((4 - (base64.length % 4)) % 4)
  try {
    const binary = atob(padded)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    return bytes
  } catch {
    return null
  }
}

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

export async function encodeShareHashSuffix(
  filename: string,
  content: string,
): Promise<string> {
  await ensureWasmInit()
  const payload = JSON.stringify({ filename, content } satisfies SharePayload)
  const compressed = compress_share_payload(payload)
  return bytesToBase64Url(compressed)
}

async function tryDecodeBrotli(encoded: string): Promise<SharePayload | null> {
  const bytes = base64UrlToBytes(encoded)
  if (!bytes) return null
  await ensureWasmInit()
  const decompressed = decompress_share_payload(bytes)
  if (decompressed === undefined) return null
  return parseSharePayloadJson(decompressed)
}

export async function decodeShareHashSuffix(
  encoded: string,
): Promise<SharePayload | null> {
  const brotliDecoded = await tryDecodeBrotli(encoded)
  if (brotliDecoded) return brotliDecoded

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

export async function buildShareUrl(
  filename: string,
  content: string,
): Promise<string> {
  const base = new URL(import.meta.env.BASE_URL, window.location.origin)
  return `${base.href}${SHARE_HASH_PREFIX}${await encodeShareHashSuffix(filename, content)}`
}

export async function parseShareFromHash(
  hash: string = window.location.hash,
): Promise<SharePayload | null> {
  if (!hash.startsWith(SHARE_HASH_PREFIX)) return null
  return decodeShareHashSuffix(hash.slice(SHARE_HASH_PREFIX.length))
}

export function clearShareHash(): void {
  const url = new URL(window.location.href)
  url.hash = ''
  history.replaceState(null, '', `${url.pathname}${url.search}`)
}
