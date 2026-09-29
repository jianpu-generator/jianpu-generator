import shareIdFormat from './generated/live-share-worker/shareIdFormat.json'
import { SHARE_QUERY_PARAM } from './shareQueryParam'

const SYNCED_HASH_PREFIX = '#synced='

const FILENAME_EXTENSION = '.jianpu'

// Generated from `crates/live-share-worker/src/share_id.rs`
// (`share_id_format`) by `build:worker-types` -- the worker mints every
// share id in exactly this shape, and this is its only definition in TS.
const SHARE_ID_LENGTH = shareIdFormat.length
const SHARE_ID_PATTERN = new RegExp(shareIdFormat.pattern)

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
  const payload = `${shareId}${namePart}`
  base.searchParams.set(SHARE_QUERY_PARAM, payload)
  return `${base.href}${SYNCED_HASH_PREFIX}${payload}`
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
  url.searchParams.delete(SHARE_QUERY_PARAM)
  const search = url.searchParams.toString()
  history.replaceState(null, '', `${url.pathname}${search ? `?${search}` : ''}`)
}

/** Removes the `?s=` param (added purely for a link-preview crawler's
 * pre-JS request, see `shareQueryParam.ts`) from the address bar once
 * client JS has parsed it, leaving the `#synced=` hash untouched -- unlike
 * `clearSyncedShareHash`, the viewer is still using the hash for its
 * session. Keeps the URL a viewer sees tidy without affecting anything the
 * server already handled. */
export function stripShareQueryParam(): void {
  const url = new URL(window.location.href)
  if (!url.searchParams.has(SHARE_QUERY_PARAM)) return
  url.searchParams.delete(SHARE_QUERY_PARAM)
  const search = url.searchParams.toString()
  history.replaceState(
    null,
    '',
    `${url.pathname}${search ? `?${search}` : ''}${url.hash}`,
  )
}
