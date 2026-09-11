//! Unit tests for `resolve_role`, ported from the old
//! `live-share-worker/test/resolveRole.test.ts` and retargeted to the
//! resolved-identity-vs-`owner_user_id` comparison (see
//! `src/resolve_role.rs`'s doc comment for why the shape changed).
#![allow(clippy::disallowed_macros)]

use live_share_worker::resolve_role::{resolve_role, SyncedRole};

#[test]
fn treats_a_request_with_no_resolved_identity_as_a_viewer() {
    assert_eq!(resolve_role("owner-id", None), SyncedRole::Viewer);
}

#[test]
fn treats_a_resolved_identity_matching_the_owner_as_the_owner() {
    assert_eq!(
        resolve_role("owner-id", Some("owner-id")),
        SyncedRole::Owner
    );
}

#[test]
fn treats_a_resolved_identity_that_does_not_match_the_owner_as_a_viewer() {
    assert_eq!(
        resolve_role("owner-id", Some("someone-elses-id")),
        SyncedRole::Viewer
    );
}
