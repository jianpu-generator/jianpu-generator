/**
 * Pure helpers and low-level utilities used by `githubBackend.ts` — path
 * joining, base64 encode/decode for the Contents API, error classification
 * primitives, and the `localStorage`-backed name -> file-ID map. Split out so
 * `githubBackend.ts` (the `StorageBackend` implementation itself) stays
 * focused on the GitHub API calls and `StorageBackend` methods.
 */
import type { FileStoreState } from '../fileStore'

/** `localStorage` key prefix for the per-repo name -> file-ID map (see
 * `readStoredFileIds`/`writeStoredFileIds`). Scoped by `owner/repo` since a
 * user could point the backend at different repos over time. */
const FILE_IDS_STORAGE_PREFIX = 'jianpu:github-file-ids:v1:'

function fileIdsStorageKey(owner: string, repo: string): string {
  return `${FILE_IDS_STORAGE_PREFIX}${owner}/${repo}`
}

/** Reads the previously assigned name -> file-ID map for this repo, if any.
 * Falls back to an empty map on missing/corrupt storage so callers can
 * treat every name as new. */
export function readStoredFileIds(
  owner: string,
  repo: string,
): Record<string, string> {
  try {
    const raw = localStorage.getItem(fileIdsStorageKey(owner, repo))
    if (!raw) return {}
    const parsed = JSON.parse(raw) as unknown
    return parsed && typeof parsed === 'object'
      ? (parsed as Record<string, string>)
      : {}
  } catch {
    return {}
  }
}

export function writeStoredFileIds(
  owner: string,
  repo: string,
  fileIds: Record<string, string>,
): void {
  try {
    localStorage.setItem(
      fileIdsStorageKey(owner, repo),
      JSON.stringify(fileIds),
    )
  } catch {
    // Ignore write failures (e.g. private-browsing storage quotas); IDs
    // simply won't survive a reload, which is a safe degradation.
  }
}

export function joinPath(...segments: (string | undefined)[]): string {
  return segments
    .filter((segment): segment is string => !!segment && segment.length > 0)
    .map((segment) => segment.replace(/^\/+|\/+$/g, ''))
    .join('/')
}

export function encodeBase64(text: string): string {
  const bytes = new TextEncoder().encode(text)
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary)
}

export function decodeBase64(base64: string): string {
  const binary = atob(base64.replace(/\n/g, ''))
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0))
  return new TextDecoder().decode(bytes)
}

export function statusOf(error: unknown): number | undefined {
  if (typeof error === 'object' && error !== null && 'status' in error) {
    const status = (error as { status: unknown }).status
    return typeof status === 'number' ? status : undefined
  }
  return undefined
}

/** A raw `fetch` failure (offline, DNS failure, aborted request) surfaces
 * differently depending on what layer throws it. If something outside
 * Octokit's own request path throws, it's a plain `TypeError` with no
 * `.status`. But Octokit's `fetchWrapper` always catches the underlying
 * `fetch` rejection itself and rethrows a `RequestError` with `status: 500`
 * and no `.response` (unlike a real HTTP error response, which always
 * carries one) — the original `TypeError` survives on `.cause`. Both shapes
 * are checked here since either can reach a caller depending on where in
 * `githubBackend.ts` the failure originates. */
export function isNetworkError(error: unknown): boolean {
  if (statusOf(error) === undefined && error instanceof TypeError) return true
  if (typeof error !== 'object' || error === null) return false
  const { response, cause } = error as { response?: unknown; cause?: unknown }
  return response == null && cause instanceof TypeError
}

/** The single name added to `userFiles` between two `FileStoreState`s, used
 * to recover which file a pure `fileStore.ts` transform just created,
 * renamed to, or restored — rather than re-deriving that naming logic
 * (uniqueness/sanitization) a second time in this module. */
export function addedFileName(
  before: FileStoreState,
  after: FileStoreState,
): string | undefined {
  return Object.keys(after.userFiles).find(
    (name) => !(name in before.userFiles),
  )
}
