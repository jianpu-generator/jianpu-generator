import { resolveRole } from './resolveRole'
import type { LiveDoc, LiveWriteRequest } from './protocol'

/** Storage shape in KV — `LiveDoc` plus the `ownerToken` that must never be
 * sent back to a client (see `toPublicDoc`). */
export interface StoredDoc extends LiveDoc {
  ownerToken: string
}

export const EMPTY_DOC: LiveDoc = {
  ended: true,
  filename: '',
  content: '',
  revision: 0,
}

/** Strips `ownerToken` before a doc goes back over the wire. A room that
 * was never shared (no KV entry yet) reads the same as one that was shared
 * and then stopped — both are just "nothing to show a viewer". */
export function toPublicDoc(stored: StoredDoc | null): LiveDoc {
  if (!stored) return EMPTY_DOC
  const { ended, filename, content, revision } = stored
  return { ended, filename, content, revision }
}

/**
 * Applies an owner's write to the room's existing stored doc (`null` for a
 * room that's never been written to), returning the new doc to persist or
 * `'forbidden'` if the request's token isn't the room's owner. Pure and
 * KV-free so the write-guard and doc transitions are unit-testable without
 * a real KV namespace, mirroring `resolveRole`'s own extraction.
 */
export function applyWrite(
  existing: StoredDoc | null,
  request: LiveWriteRequest,
): StoredDoc | 'forbidden' {
  const role = resolveRole(existing?.ownerToken ?? null, request.ownerToken)
  if (role !== 'owner') return 'forbidden'
  // The first write from a token claims ownership; a reconnect/resend with
  // the same token stays the owner (see `resolveRole`) — either way the
  // room's `ownerToken` is whichever was already stored, or this request's
  // if there wasn't one yet.
  const ownerToken = existing?.ownerToken ?? request.ownerToken

  if (request.type === 'stop') {
    return { ...(existing ?? { ...EMPTY_DOC, ownerToken }), ownerToken, ended: true }
  }

  return {
    ownerToken,
    filename: request.filename,
    content: request.content,
    revision: request.revision,
    ended: false,
  }
}
