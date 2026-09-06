export type SyncedRole = 'owner' | 'viewer'

/**
 * The entire write-guard for a share, extracted as a pure function so the
 * security-critical branch is unit-testable without a real KV namespace.
 * A request with no token is always a viewer. The first write to present a
 * token claims ownership (persisted alongside the share's stored doc, see
 * `applyWrite`); a later write with that same token is also the owner,
 * which is what lets a reloaded owner tab resume its session instead of
 * orphaning viewers.
 */
export function resolveRole(
  storedOwnerToken: string | null,
  incomingToken: string | null,
): SyncedRole {
  if (!incomingToken) return 'viewer'
  if (storedOwnerToken === null || incomingToken === storedOwnerToken) {
    return 'owner'
  }
  return 'viewer'
}
