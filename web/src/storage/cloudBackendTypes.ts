import type { FileStoreState } from '../fileStore'
import type { StorageBackend } from './types'

export interface CloudBackendConfig {
  /** The unified account sign-in token (`accountAuth.ts`) — the same
   * token already used to prove
   * ownership of a Synced Share link, sent as `identityToken` on every
   * `/files/*` request body (never as a header/query param — matches this
   * worker's `FileShareRequest` convention). */
  token: string
  /** Bare host (no scheme), same shape as `useSyncedShareOwner.ts`'s
   * `VITE_SYNCED_SHARE_HOST` — turned into an origin via
   * `syncedShareWorkerOrigin`. */
  workerHost: string
}

/**
 * Typed detail behind a `'error'`/`'offline'` `SaveStatus`, mirroring
 * `GithubBackendError`'s role for the old GitHub backend. `'conflict'`
 * carries the server's current revision (for `forceOverwrite`/"Discard
 * mine"); `'auth'` is new here (401 -- the account sign-in token was
 * revoked/expired), replacing GitHub's `'rate-limited'` (this worker has no
 * rate limiting of its own, see `TODO-synced-share-rust-d1-migration.md`).
 */
export type CloudBackendError =
  | { kind: 'conflict'; currentRevision: number }
  | { kind: 'auth' }
  | { kind: 'network' }
  | { kind: 'unknown'; message: string }

export interface CloudBackend extends StorageBackend {
  readonly kind: 'cloud'
  /** Detail behind the most recent `'error'`/`'offline'` status, if any. */
  lastError(): CloudBackendError | null
  /**
   * Used by `storageSettingsModalHelpers.ts`'s "Overwrite mine": realigns
   * the tracked revision for `state.active` to the server's last-reported
   * current revision (from a `'conflict'` `lastError`), then retries the
   * save with that revision -- so the CAS write that previously lost the
   * race now succeeds and re-pushes the caller's in-memory edit.
   */
  forceOverwrite(state: FileStoreState): Promise<void>
}
