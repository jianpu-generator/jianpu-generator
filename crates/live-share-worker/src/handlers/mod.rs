//! HTTP routing for the Synced Share worker, ported from the old TS
//! `live-share-worker/src/index.ts`. CORS and the `OPTIONS` preflight are
//! handled once in `lib.rs` around whatever this router returns; this
//! module owns the `/shares` routes (`shares` submodule), the `/files`
//! routes (the cloud storage backend, see `crate::files`; `files`
//! submodule), plus the `/auth/github/callback` and `/auth/github/revoke`
//! routes (see `crate::oauth`) added for the dedicated Synced Share sign-in
//! connection.
//!
//! Every write (and the create-share endpoint) resolves its caller's
//! identity via `identity::resolve_verified_user_id`, fronted by
//! `identity::github::GithubIdentityProvider` -- never the client's own
//! claim -- per `TODO-synced-share-rust-d1-migration.md` §0/§6 (task 7). A
//! verification failure (after the backoff-wrapped retries) fails the
//! request closed with a structured `verification::VerificationFailure`
//! body (401): reason, timestamp, attempt count, and deliberately nothing
//! else -- the token/hash must never appear in this response. Every
//! `/files/*` route requires a resolved identity too -- unlike `GET
//! /shares/:share_id`, there is no anonymous read path for a file.
//!
//! Helpers below (`D1_BINDING`, `verification_failure_response`,
//! `is_unique_constraint_violation`, `resolve_files_caller`,
//! `name_taken_response`) are private to this module but visible to both
//! submodules per Rust's ancestor-visibility rule -- no `pub(crate)`
//! needed.

mod files;
mod shares;

use worker::{D1Database, RouteContext, Router};
use worker::{Response, Result};

use crate::identity::github::GithubIdentityProvider;
use crate::identity::resolve_verified_user_id;
use crate::oauth;
use crate::verification::VerificationFailure;

/// D1 binding name this worker expects in `wrangler.toml`. Wiring the
/// actual binding is task 5 (`TODO-synced-share-rust-d1-migration.md` §4's
/// "Update `wrangler.toml`" bullet) -- out of scope here.
const D1_BINDING: &str = "DB";

pub(crate) fn router() -> Router<'static, ()> {
    Router::new()
        .get_async("/shares/:share_id", shares::get_share)
        .post_async("/shares/:share_id", shares::post_share)
        .post_async("/shares", shares::create_share)
        .post_async("/auth/github/callback", oauth::github_oauth_callback)
        .post_async("/auth/github/revoke", oauth::github_revoke)
        .post_async("/files/list", files::list_files)
        .post_async("/files", files::create_file)
        .post_async("/files/:id/content", files::update_file_content)
        .post_async("/files/:id/rename", files::rename_file)
        .post_async("/files/:id/delete", files::delete_file)
        .post_async("/files/:id/restore", files::restore_file)
}

/// Turns a failed verification (GitHub call failed even after the
/// backoff-wrapped retries) into the write's HTTP response: `401`, body is
/// exactly `VerificationFailure`'s fields (reason, timestamp, attempt
/// count) and nothing else -- per TODO §0, the token/hash must never appear
/// here, which `VerificationFailure` enforces by construction (it has no
/// such field to leak).
fn verification_failure_response(failure: &VerificationFailure) -> Result<Response> {
    Ok(Response::from_json(failure)?.with_status(401))
}

/// D1 surfaces a SQLite `UNIQUE constraint failed` violation as a plain
/// stringly-typed `worker::Error`, not a structured error code -- so this
/// checks the message text. Exact matching here is an implementation
/// detail of this defense-in-depth path, not part of `share_creation`'s
/// design: `resolve_share_id` only needs to know "created" vs "conflict",
/// never which real error this maps from.
fn is_unique_constraint_violation(error: &worker::Error) -> bool {
    error.to_string().contains("UNIQUE constraint failed")
}

/// Resolves a `/files/*` request's caller identity, same path every Synced
/// Share write already goes through -- see this module's doc comment.
/// Factored out since every `/files/*` handler needs exactly this.
async fn resolve_files_caller(
    d1: &D1Database,
    ctx: &RouteContext<()>,
    identity_token: &str,
) -> Result<Result<String, VerificationFailure>> {
    let identity_provider = GithubIdentityProvider::from_env(ctx);
    resolve_verified_user_id(d1, &identity_provider, identity_token).await
}

/// Maps a `idx_files_owner_name` unique-constraint violation (a genuine
/// name-collision race on create/rename/restore, see
/// `migrations/0003_files.sql`) to a `409` distinct in shape from
/// `ConflictResponse` (the revision-conflict shape used by the content-save
/// route) -- matches `crate::protocol::CreateFileRequest`'s doc comment.
fn name_taken_response() -> Result<Response> {
    Ok(Response::from_json(&serde_json::json!({ "code": "name_taken" }))?.with_status(409))
}
