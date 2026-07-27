// Mirrored (not shared via a workspace package) at `web/src/live/protocol.ts`
// — keep the two in sync by hand when changing either.

export type LiveRole = 'owner' | 'viewer'

// Sent right after connect, and again on any full broadcast. Covers both
// "late joiner needs current state" and, later, phase-2's "initial doc"
// need, so the shape doesn't have to change.
export interface LiveSyncMessage {
  type: 'sync'
  role: LiveRole
  // True once the owner has pressed "Stop Live" — the room keeps existing
  // (same id, same token) so a later "Go Live" reproduces the same link,
  // but viewers must not see `content`/`filename` until it flips back.
  ended: boolean
  filename: string
  content: string
  revision: number
}

// Owner -> server -> other clients. Whole-content, not a diff/delta format
// — correct-by-construction with exactly one writer, and deliberately
// leaves the "delta" protocol space empty for Yjs's own update messages in
// phase 2 rather than inventing a rival one that gets thrown away.
export interface LiveUpdateMessage {
  type: 'update'
  filename: string
  content: string
  revision: number
}

export interface LivePresenceMessage {
  type: 'presence'
  connectionCount: number
}

// Owner -> server -> viewers. Broadcast the moment the owner stops, so
// already-connected viewers drop the score immediately instead of waiting
// for their socket to close on its own.
export interface LiveEndedMessage {
  type: 'ended'
}

// Owner -> server. Marks the room ended (see `LiveSyncMessage.ended`)
// without deleting `ownerToken`/`doc`, so the room — and therefore the
// link — survives to be reused by a later "Go Live" on the same file.
export interface LiveStopMessage {
  type: 'stop'
}

export type LiveServerMessage =
  | LiveSyncMessage
  | LiveUpdateMessage
  | LivePresenceMessage
  | LiveEndedMessage

export type LiveClientMessage = LiveUpdateMessage | LiveStopMessage // only owner sends these
