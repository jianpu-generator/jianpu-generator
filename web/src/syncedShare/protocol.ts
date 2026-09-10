// Mirrored (not shared via a workspace package) from
// `live-share-worker/src/protocol.ts` — keep the two in sync by hand when changing
// either.
//
// Plain request/response now (KV-backed, no persistent connection) — there
// is no more "message" being pushed to anyone, just a doc a viewer fetches
// on load and an owner overwrites on save.

// What a `GET /shares/:shareId` returns. `ended` mirrors the owner having
// pressed "Stop Sync" — the KV entry keeps existing (same id, same owner
// token) so a later "Sync" click reproduces the same link, but a viewer must
// not treat `content`/`filename` as current once this is true.
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

// `identityToken` mirrors the Rust worker's `SyncedWriteRequest::identity_token`
// (`crates/live-share-worker/src/protocol.rs`) -- the Synced Share GitHub
// sign-in token (`syncedShareGithubAuth.ts`), sent alongside the still-live
// `ownerToken` field per task 8: the old device-secret/`ownerToken` path is
// untouched here (its removal is task 11), this is purely additive. Present
// only once the Synced Share GitHub connection exists; omitted otherwise
// (the deployed worker at that point is still the old KV one, which ignores
// unknown fields).
export interface SyncedIdentityFields {
  identityToken?: string
}

// Owner -> server. Whole-content, not a diff/delta format — correct-by-
// construction with exactly one writer.
export interface SyncedUpdateRequest extends SyncedIdentityFields {
  type: 'update'
  ownerToken: string
  filename: string
  content: string
  revision: number
}

// Owner -> server. Marks the share ended (see `SyncedDoc.ended`) without
// discarding the stored doc/ownerToken, so the share — and therefore the
// link — survives to be reused by a later "Sync" click on the same file.
export interface SyncedStopRequest extends SyncedIdentityFields {
  type: 'stop'
  ownerToken: string
}

// Body of `POST /shares/:shareId` — only the owner is ever allowed to send
// these (enforced server-side by `resolveRole`, keyed on `ownerToken`).
export type SyncedWriteRequest = SyncedUpdateRequest | SyncedStopRequest
