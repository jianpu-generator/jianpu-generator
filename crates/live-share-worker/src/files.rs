//! Pure, D1-free logic for the cloud (D1) storage backend's `files` table --
//! the replacement for the old GitHub Contents API storage backend, unified
//! under the same account identity `crate::identity` already resolves for
//! Synced Share. See `live-share-worker/migrations/0003_files.sql` for the
//! schema this mirrors.
//!
//! There is no single "apply a write" entry point --
//! `crate::handlers`'s `/files/*` routes each perform their own atomic D1
//! `UPDATE`/`INSERT` (see `crate::db`), because rename/delete/restore are
//! unconditional (no revision gate, per the old GitHub backend's actual
//! behavior) while a content save is gated on `expected_revision` as an
//! optimistic-concurrency guard. What stays pure and unit-testable here is
//! *classifying* the outcome of a content-save attempt -- the one op with
//! real conflict semantics to get right.

use serde::{Deserialize, Serialize};

/// Mirrors a `files` row (see `live-share-worker/migrations/0003_files.sql`).
/// `owner_user_id` is an internal, provider-agnostic `users.id`, never sent back to a client -- see
/// `to_public_file`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFile {
    pub id: String,
    pub owner_user_id: String,
    pub name: String,
    pub content: String,
    pub revision: i64,
    pub trashed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// The public shape of a `files` row -- strips `owner_user_id` and the
/// timestamps before a file goes back over the wire, same reasoning as
/// `crate::share::to_public_doc`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicFile {
    pub id: String,
    pub name: String,
    pub content: String,
    pub revision: i64,
    pub trashed_at: Option<i64>,
}

pub fn to_public_file(file: &StoredFile) -> PublicFile {
    PublicFile {
        id: file.id.clone(),
        name: file.name.clone(),
        content: file.content.clone(),
        revision: file.revision,
        trashed_at: file.trashed_at,
    }
}

/// What a content-write `UPDATE` attempt reported back -- see
/// `queries/update_file_content.sql`'s `WHERE id=? AND owner_user_id=? AND
/// revision=? AND trashed_at IS NULL` guard. `crate::handlers` is
/// responsible for turning a real D1 `D1Result` into one of these (its
/// `meta().changes`, or a follow-up read if that isn't available -- see
/// that module's doc comment) so this classification stays D1-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentWriteAttempt {
    Applied,
    RowsAffectedZero,
}

/// The outcome `crate::handlers`'s content-save route turns into an HTTP
/// response: `Applied` -> 200, `Conflict` -> 409 (existing "Overwrite mine" /
/// "Discard mine" UI), `NotFound` -> 404.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentWriteOutcome {
    Applied { new_revision: i64 },
    Conflict { current_revision: i64 },
    NotFound,
}

/// Classifies a content-write attempt against the row's state *after* the
/// attempt (`current`, from `queries/get_file_by_id.sql` -- `None` if the
/// row doesn't exist or the caller isn't its owner, since that query is
/// scoped by `id` + `owner_user_id`). Pure so the CAS logic is
/// unit-testable without a real D1 database.
///
/// - `Applied`: the `UPDATE` matched a row (it was at `expected_revision`,
///   active, and owned by the caller) -- `new_revision` is
///   `expected_revision + 1`, matching `update_file_content.sql`'s
///   `revision=revision+1`.
/// - `RowsAffectedZero` + a `current` row still present: the row moved on
///   (its `revision` no longer matches `expected_revision`) or was trashed
///   out from under the write -- either way, the client's edit conflicts
///   with the server's current state, so this reports `Conflict` with
///   whatever revision the row is actually at now, for the "Overwrite
///   mine"/"Discard mine" UI.
/// - `RowsAffectedZero` + no `current` row: the file doesn't exist, or
///   isn't owned by the caller -- `NotFound`, not a conflict (there's
///   nothing to reconcile against).
pub fn classify_content_write(
    attempt: ContentWriteAttempt,
    expected_revision: i64,
    current: Option<&StoredFile>,
) -> ContentWriteOutcome {
    match attempt {
        ContentWriteAttempt::Applied => ContentWriteOutcome::Applied {
            new_revision: expected_revision + 1,
        },
        ContentWriteAttempt::RowsAffectedZero => match current {
            Some(file) => ContentWriteOutcome::Conflict {
                current_revision: file.revision,
            },
            None => ContentWriteOutcome::NotFound,
        },
    }
}
