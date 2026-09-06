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
}

// Owner -> server. Whole-content, not a diff/delta format — correct-by-
// construction with exactly one writer.
export interface SyncedUpdateRequest {
  type: 'update'
  ownerToken: string
  filename: string
  content: string
  revision: number
}

// Owner -> server. Marks the share ended (see `SyncedDoc.ended`) without
// discarding the stored doc/ownerToken, so the share — and therefore the
// link — survives to be reused by a later "Sync" click on the same file.
export interface SyncedStopRequest {
  type: 'stop'
  ownerToken: string
}

// Body of `POST /shares/:shareId` — only the owner is ever allowed to send
// these (enforced server-side by `resolveRole`, keyed on `ownerToken`).
export type SyncedWriteRequest = SyncedUpdateRequest | SyncedStopRequest
