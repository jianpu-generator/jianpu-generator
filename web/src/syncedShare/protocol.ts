// Mirrored (not shared via a workspace package) from
// `crates/live-share-worker/src/protocol.rs` — keep the two in sync by hand
// when changing either.
//
// A synced share is a pointer to a cloud `files` row, not a copy of its
// content: a viewer fetches the file itself, and the owner's ordinary cloud
// autosave is the only write path. The owner-side routes below only start,
// stop and report on that pointer.

// What a `GET /shares/:shareId` returns. `filename`/`content`/`revision`
// are the shared file's current saved state. `ended` is true when the owner
// pressed "Stop Sync" or the file is in the bin — the share keeps existing
// (same id) so a later "Sync" click reproduces the same link, and while
// ended `filename`/`content` come back empty.
export interface SyncedDoc {
  ended: boolean
  filename: string
  content: string
  revision: number
  // Mirrors the Rust worker's `protocol::SyncedDoc::owner_login` (task 10):
  // the owning user's cached `user_identities.login`, exposed deliberately
  // for the viewer-facing "Shared by @username" attribution
  // (`SyncedShareBanner`). `null`/absent when the owner has no cached
  // login. No other internal id, token, or hash is ever present on this
  // response -- see `crate::share::to_public_doc`.
  ownerLogin?: string | null
}

// Body of `POST /files/:id/share`, `POST /files/:id/share/stop` and
// `POST /files/:id/share/status`. `identityToken` is the Synced Share GitHub
// sign-in token (`accountAuthCallback.ts`); the worker never trusts it
// directly, always resolving it through GitHub verification (task 7), and
// only lets the file's owner act on its share.
export interface FileShareRequest {
  identityToken: string
}

// Response of `POST /files/:id/share` — the file's one share id, the same
// every time the file is shared.
export interface FileShareResponse {
  shareId: string
}

// A file's share as its owner sees it.
interface ShareStatus {
  shareId: string
  ended: boolean
}

// Response of `POST /files/:id/share/status` — `share` is `null` when the
// file has never been shared.
export interface ShareStatusResponse {
  share: ShareStatus | null
}
