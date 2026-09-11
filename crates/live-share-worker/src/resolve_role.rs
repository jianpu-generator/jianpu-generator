//! The write-guard for a share, ported from the old TS
//! `live-share-worker/src/resolveRole.ts`. Retargeted from comparing two
//! opaque `ownerToken` strings to comparing a resolved identity's
//! `user_id` against `docs.owner_user_id` (see
//! `TODO-synced-share-rust-d1-migration.md` §0/§6: "owner = resolved
//! `user_id` matches `docs.owner_user_id` exactly").

/// A share's role for the request currently being handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncedRole {
    Owner,
    Viewer,
}

/// A share's ownership is fixed at creation time now (no more "unpinned"
/// state a first write can claim, unlike the old KV model -- see TODO §0),
/// so this is a straight comparison: the resolved identity is the owner
/// only if it matches `owner_user_id` exactly. `resolved_user_id` is `None`
/// when identity resolution failed or wasn't attempted -- always a viewer
/// in that case, the same outcome as the old "no token" case.
pub fn resolve_role(owner_user_id: &str, resolved_user_id: Option<&str>) -> SyncedRole {
    match resolved_user_id {
        Some(user_id) if user_id == owner_user_id => SyncedRole::Owner,
        _ => SyncedRole::Viewer,
    }
}
