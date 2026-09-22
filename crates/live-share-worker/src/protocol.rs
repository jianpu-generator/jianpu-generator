//! Wire types for the Synced Share worker's HTTP API. Ported from the old
//! TS `live-share-worker/src/protocol.ts` (mirrored, not shared via a
//! workspace package, with `web/src/syncedShare/protocol.ts` -- keep the
//! two in sync by hand when either changes, same constraint as the file
//! this was ported from).
//!
//! JSON field names use `camelCase` on the wire (matching the old TS
//! convention, e.g. `ownerToken`), even though Rust field names stay
//! `snake_case`.

use serde::{Deserialize, Serialize};

/// What `GET /shares/:share_id` returns. `ended` mirrors the owner having
/// pressed "Stop Sync" -- the `docs` row keeps existing (same `share_id`,
/// same `owner_user_id`) so a later "Sync" click reproduces the same link,
/// but a viewer must not treat `content`/`filename` as current once this is
/// true. Never carries `owner_user_id`, `share_id`, any token/hash, or any
/// other internal id -- see `crate::doc::to_public_doc`. `owner_login` is
/// the one deliberate exception: `user_identities.login` (a cached, public
/// GitHub display name, not an internal id) is exposed here specifically so
/// the viewer-facing `SyncedShareBanner` can show "Shared by @login"; it's
/// `None` when the owner has no cached login (task 10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncedDoc {
    pub ended: bool,
    pub filename: String,
    pub content: String,
    pub revision: i64,
    pub owner_login: Option<String>,
}

/// Body of `POST /shares/:share_id` -- only the share's owner is ever
/// allowed to have a write applied (enforced server-side by
/// `crate::resolve_role::resolve_role`, keyed on the identity resolved from
/// `identity_token`, see `crate::identity`).
///
/// `identity_token` is opaque at this layer: for now it's whatever
/// `crate::identity::stub::StubIdentityProvider` accepts (not a real OAuth
/// token -- see that module's doc comment). Struct-style enum variants
/// (never tuple variants, per this repo's no-tuple-in-new-data-structures
/// convention) mirror the old TS `SyncedUpdateRequest` / `SyncedStopRequest`
/// union members, tagged the same way (`type: "update" | "stop"`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SyncedWriteRequest {
    Update {
        identity_token: String,
        filename: String,
        content: String,
        revision: i64,
    },
    Stop {
        identity_token: String,
    },
}

impl SyncedWriteRequest {
    /// The opaque identity token every write carries, regardless of variant.
    pub fn identity_token(&self) -> &str {
        match self {
            SyncedWriteRequest::Update { identity_token, .. }
            | SyncedWriteRequest::Stop { identity_token } => identity_token,
        }
    }
}

/// Body of `POST /shares` -- creates a brand-new, server-generated share
/// (see `TODO-synced-share-rust-d1-migration.md` §1: `shareId` generation
/// moved server-side). Requires a resolved identity; unlike the old TS
/// `index.ts`, there is no more implicit "first POST to a client-chosen id
/// creates it" behavior.
///
/// `external_file_id` is `Some` only for a GitHub-backed file (the client's
/// `owner/repo/scores/name` Contents API path, computed in
/// `web/src/hooks/useScoreSource.ts`); `None`/absent for a local-only file.
/// When present, `crate::handlers::create_share` makes this call idempotent
/// per `(owner_user_id, external_file_id)` -- see
/// `crate::share_creation::resolve_share_id` -- instead of always minting a
/// fresh share.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShareRequest {
    pub identity_token: String,
    pub external_file_id: Option<String>,
}

/// Response of `POST /shares`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShareResponse {
    pub share_id: String,
}

// -- `/files/*` (cloud storage backend) -------------------------------------
//
// Unlike `SyncedDoc` above (a wire shape distinct from the pure
// `crate::doc::StoredDoc`), the public wire shape for a file is
// `crate::files::PublicFile` itself -- it already derives `Serialize` +
// `Deserialize` with `camelCase` renaming, so these request/response types
// reuse it directly instead of duplicating an identical `PublicFileWire`
// struct here.
//
// Every `/files/*` route requires a resolved identity (see
// `crate::handlers`'s module doc comment) -- `identity_token` appears in
// every request body below, same convention as `CreateShareRequest`/
// `SyncedWriteRequest` above.

/// Body of `POST /files/list`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFilesRequest {
    pub identity_token: String,
}

/// Response of `POST /files/list` -- every file (active and trashed alike)
/// owned by the resolved caller; the client partitions this by
/// `trashedAt` into its file list vs. bin (see `cloudBackend.ts`'s
/// `load()`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFilesResponse {
    pub files: Vec<crate::files::PublicFile>,
}

/// Body of `POST /files` -- creates a new file row (new file / duplicate /
/// import). `id` is client-generated (`fileStore.ts`'s
/// `generateFileId()`) -- safe because every write is additionally gated on
/// the resolved `owner_user_id` server-side. Response is the created
/// `crate::files::PublicFile` itself; a name collision against the
/// caller's other files (active or trashed) is instead reported as `409
/// {code: "name_taken"}`, distinct from this module's `ConflictResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileRequest {
    pub identity_token: String,
    pub id: String,
    pub name: String,
    pub content: String,
}

/// Body of `POST /files/:id/content` -- the atomic CAS content save. See
/// `crate::files::classify_content_write` for how `expected_revision` gates
/// the write.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileContentRequest {
    pub identity_token: String,
    pub content: String,
    pub expected_revision: i64,
}

/// `200` response of `POST /files/:id/content`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileContentResponse {
    pub revision: i64,
}

/// `409` response of `POST /files/:id/content` when the row moved on since
/// the caller last saw it -- feeds the existing "Overwrite mine"/"Discard
/// mine" UI. Distinct in shape from the `409 {code: "name_taken"}` name-
/// collision response (create/rename/restore).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictResponse {
    pub current_revision: i64,
}

/// Body of `POST /files/:id/rename`. No revision gate -- rename has no
/// conflict semantics (matches the old GitHub backend's actual behavior,
/// decision #3). `404` on zero rows affected; a name collision maps to
/// `409 {code: "name_taken"}`, same as `CreateFileRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileRequest {
    pub identity_token: String,
    pub name: String,
}

/// Body of `POST /files/:id/delete` -- moves the file to the bin
/// (`trashed_at`). No revision gate; `404` on zero rows affected.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileRequest {
    pub identity_token: String,
}

/// Body of `POST /files/:id/restore`. `name` is supplied by the caller
/// (already recomputed client-side to be unique, per `fileStore.ts`'s
/// `uniqueName`/`reservedNames`) -- can still race into a `409 {code:
/// "name_taken"}`, same as `CreateFileRequest`/`RenameFileRequest`. `404` on
/// zero rows affected (no such trashed file for this caller).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFileRequest {
    pub identity_token: String,
    pub name: String,
}
