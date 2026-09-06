export type LiveRole = 'owner' | 'viewer'

/**
 * The entire write-guard for the room, extracted as a pure function so the
 * security-critical branch is unit-testable without a live Durable Object.
 * A connection with no token is always a viewer. The first connection to
 * present a token claims ownership (persisted to `ctx.storage`); later
 * reconnects with that same token are also owners, which is what lets a
 * reloaded owner tab resume its session instead of orphaning viewers.
 */
export function resolveRole(
  storedOwnerToken: string | null,
  incomingToken: string | null,
): LiveRole {
  if (!incomingToken) return 'viewer'
  if (storedOwnerToken === null || incomingToken === storedOwnerToken) {
    return 'owner'
  }
  return 'viewer'
}
