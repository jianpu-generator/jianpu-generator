// Mirrored (not shared via a workspace package) at `web/src/live/protocol.ts`
// — keep the two in sync by hand when changing either.
//
// Plain request/response now (KV-backed, no persistent connection) — there
// is no more "message" being pushed to anyone, just a doc a viewer fetches
// on load and an owner overwrites on save.

// What a `GET /rooms/:roomId` returns. `ended` mirrors the owner having
// pressed "Stop Live" — the KV entry keeps existing (same id, same owner
// token) so a later "Go Live" reproduces the same link, but a viewer must
// not treat `content`/`filename` as current once this is true.
export interface LiveDoc {
  ended: boolean
  filename: string
  content: string
  revision: number
}

// Owner -> server. Whole-content, not a diff/delta format — correct-by-
// construction with exactly one writer.
export interface LiveUpdateRequest {
  type: 'update'
  ownerToken: string
  filename: string
  content: string
  revision: number
}

// Owner -> server. Marks the room ended (see `LiveDoc.ended`) without
// discarding the stored doc/ownerToken, so the room — and therefore the
// link — survives to be reused by a later "Go Live" on the same file.
export interface LiveStopRequest {
  type: 'stop'
  ownerToken: string
}

// Body of `POST /rooms/:roomId` — only the owner is ever allowed to send
// these (enforced server-side by `resolveRole`, keyed on `ownerToken`).
export type LiveWriteRequest = LiveUpdateRequest | LiveStopRequest
