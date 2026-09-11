// Mirrored (not shared via a workspace package) from
// `crates/live-share-worker/src/protocol.rs` — keep the two in sync by hand
// when changing either.
//
// Plain request/response now (D1-backed, no persistent connection) — there
// is no more "message" being pushed to anyone, just a doc a viewer fetches
// on load and an owner overwrites on save.

// What a `GET /shares/:shareId` returns. `ended` mirrors the owner having
// pressed "Stop Sync" — the `docs` row keeps existing (same id, same owner)
// so a later "Sync" click reproduces the same link, but a viewer must not
// treat `content`/`filename` as current once this is true.
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
  // response -- see `crate::doc::to_public_doc`.
  ownerLogin?: string | null
}

// Owner -> server. Whole-content, not a diff/delta format — correct-by-
// construction with exactly one writer. `identityToken` is the Synced Share
// GitHub sign-in token (`syncedShareGithubAuth.ts`); the worker never trusts
// it directly, always resolving it through GitHub verification (task 7).
// There is no more `ownerToken`/device-secret path (task 11) -- GitHub
// sign-in is the only way to own a share.
export interface SyncedUpdateRequest {
  type: 'update'
  identityToken: string
  filename: string
  content: string
  revision: number
}

// Owner -> server. Marks the share ended (see `SyncedDoc.ended`) without
// discarding the stored doc, so the share — and therefore the link —
// survives to be reused by a later "Sync" click on the same file.
export interface SyncedStopRequest {
  type: 'stop'
  identityToken: string
}

// Body of `POST /shares/:shareId` — only the owner is ever allowed to send
// these (enforced server-side by `resolve_role`, keyed on the identity
// resolved from `identityToken`).
export type SyncedWriteRequest = SyncedUpdateRequest | SyncedStopRequest

// Body of `POST /shares` — creates a brand-new, server-generated share
// (TODO §1: `shareId` generation moved server-side, no more client-derived
// id). Requires a resolved GitHub identity.
export interface CreateShareRequest {
  identityToken: string
}

// Response of `POST /shares`.
export interface CreateShareResponse {
  shareId: string
}
