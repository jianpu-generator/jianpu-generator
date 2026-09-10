//! Doc transitions and the write-guard for a Synced Share, ported from the
//! old TS `live-share-worker/src/doc.ts`. Retargeted from the KV
//! `StoredDoc` (doc + bearer `ownerToken`) to a `docs` table row keyed by
//! an internal `owner_user_id` -- see `crate::resolve_role`.

use crate::protocol::SyncedDoc;
use crate::protocol::SyncedWriteRequest;
use crate::resolve_role::{resolve_role, SyncedRole};

/// Mirrors a `docs` row (see `live-share-worker/migrations/0001_init.sql`).
/// Unlike the old KV `StoredDoc`, `owner_user_id` is an internal,
/// provider-agnostic `users.id`, never sent back to a client -- see
/// `to_public_doc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDoc {
    pub share_id: String,
    pub owner_user_id: String,
    pub filename: String,
    pub content: String,
    pub revision: i64,
    pub ended: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A share id with no `docs` row yet reads the same as one that was shared
/// and then stopped -- both are "nothing to show a viewer" (mirrors the old
/// `EMPTY_DOC` constant).
pub fn empty_doc() -> SyncedDoc {
    SyncedDoc {
        ended: true,
        filename: String::new(),
        content: String::new(),
        revision: 0,
    }
}

/// Strips everything but the public shape before a doc goes back over the
/// wire -- `owner_user_id`, `share_id`, and the timestamps never leave the
/// server (mirrors the old `toPublicDoc`).
pub fn to_public_doc(stored: Option<&StoredDoc>) -> SyncedDoc {
    match stored {
        None => empty_doc(),
        Some(doc) => SyncedDoc {
            ended: doc.ended,
            filename: doc.filename.clone(),
            content: doc.content.clone(),
            revision: doc.revision,
        },
    }
}

/// A rejected write. The only rejection reason at this layer is a resolved
/// identity that doesn't match the share's `owner_user_id` -- per
/// `TODO-synced-share-rust-d1-migration.md` §0: "any mismatch or
/// verification failure ... is rejected outright, no other cases exist".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    Forbidden,
}

/// Applies an owner's write to a share's existing `docs` row. A share must
/// already exist for this to be called (see the "create share" endpoint in
/// `crate::handlers`) -- unlike the old KV `applyWrite`, there is no more
/// "first write claims ownership" state, since ownership is fixed at
/// creation time (TODO §0).
///
/// Pure and D1-free so the write-guard and doc transitions are
/// unit-testable without a real D1 database: `resolved_user_id` stands in
/// for whatever the caller's `crate::identity::IdentityProvider` resolved,
/// and `now` stands in for the current time.
pub fn apply_write(
    existing: &StoredDoc,
    resolved_user_id: Option<&str>,
    request: &SyncedWriteRequest,
    now: i64,
) -> Result<StoredDoc, WriteError> {
    if resolve_role(&existing.owner_user_id, resolved_user_id) != SyncedRole::Owner {
        return Err(WriteError::Forbidden);
    }

    Ok(match request {
        SyncedWriteRequest::Update {
            filename,
            content,
            revision,
            ..
        } => StoredDoc {
            filename: filename.clone(),
            content: content.clone(),
            revision: *revision,
            ended: false,
            updated_at: now,
            ..existing.clone()
        },
        SyncedWriteRequest::Stop { .. } => StoredDoc {
            ended: true,
            updated_at: now,
            ..existing.clone()
        },
    })
}
