//! Wire types for the Synced Share worker's HTTP API. Ported from the old
//! TS `live-share-worker/src/protocol.ts` (mirrored, not shared via a
//! workspace package, with `web/src/syncedShare/protocol.ts` -- keep the
//! two in sync by hand when either changes, same constraint as the file
//! this was ported from).
//!
//! JSON field names use `camelCase` on the wire (matching the old TS
//! convention, e.g. `identityToken`), even though Rust field names stay
//! `snake_case`.

use serde::{Deserialize, Serialize};

/// What `GET /shares/:share_id` returns. `filename`/`content`/`revision`
/// are the pointed-at `files` row's current `name`/`content`/`revision` --
/// a share holds no copy of its own (see `crate::share`). `ended` is true
/// when the owner pressed "Stop Sync" or the file is in the bin; the
/// `shares` row keeps existing either way so a later start reproduces the
/// same link, and while ended `filename`/`content` are empty so a stopped
/// link can't leak the live file. Never carries `owner_user_id`,
/// `share_id`, the file id, any token/hash, or any other internal id -- see
/// `crate::share::to_public_doc`. `owner_login` is the one deliberate
/// exception: `user_identities.login` (a cached, public GitHub display
/// name, not an internal id) is exposed here specifically so the
/// viewer-facing `SyncedShareBanner` can show "Shared by @login"; it's
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

/// Body of `POST /files/:id/share`, `POST /files/:id/share/stop` and `POST
/// /files/:id/share/status` -- all three only need the caller's identity;
/// the file is named by the path, and ownership is checked server-side
/// against the resolved identity (never the client's own claim).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileShareRequest {
    pub identity_token: String,
}

/// Response of `POST /files/:id/share` -- the file's one share id, the same
/// one every time the file is shared (`shares.file_id` is unique).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileShareResponse {
    pub share_id: String,
}

/// Response of `POST /files/:id/share/status` -- `share` is `None` when the
/// caller has never shared this file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareStatusResponse {
    pub share: Option<ShareStatus>,
}

/// A file's share as its owner sees it: the link's id and whether it's been
/// stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareStatus {
    pub share_id: String,
    pub ended: bool,
}

// -- `/files/*` (cloud storage backend) -------------------------------------
//
// Unlike `SyncedDoc` above (a wire shape distinct from the pure
// `crate::share::ShareView`), the public wire shape for a file is
// `crate::files::PublicFile` itself -- it already derives `Serialize` +
// `Deserialize` with `camelCase` renaming, so these request/response types
// reuse it directly instead of duplicating an identical `PublicFileWire`
// struct here.
//
// Every `/files/*` route requires a resolved identity (see
// `crate::handlers`'s module doc comment) -- `identity_token` appears in
// every request body below, same convention as `FileShareRequest` above.

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
