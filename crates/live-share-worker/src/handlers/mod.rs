//! HTTP routing for the Synced Share worker, ported from the old TS
//! `live-share-worker/src/index.ts`. CORS and the `OPTIONS` preflight are
//! handled once in `lib.rs` around whatever this router returns; this
//! module owns the Synced Share routes (`GET /shares/{share_id}` and
//! `POST /files/{id}/share{,/stop,/status}`; `shares` submodule), the
//! `/files` routes (the cloud storage backend, see `crate::files`; `files`
//! submodule), plus the `/auth/github/callback` and `/auth/github/revoke`
//! routes (see `crate::oauth`) added for the dedicated Synced Share sign-in
//! connection.
//!
//! Every write (including starting/stopping a share) resolves its caller's
//! identity via `identity::resolve_verified_user_id`, fronted by
//! `identity::github::GithubIdentityProvider` -- never the client's own
//! claim -- per `TODO-synced-share-rust-d1-migration.md` §0/§6 (task 7). A
//! verification failure (after the backoff-wrapped retries) fails the
//! request closed with `ApiError::Unauthorized` (401), carrying a
//! `verification::VerificationFailure`: reason, timestamp, attempt count,
//! and deliberately nothing else -- the token/hash must never appear in
//! this response. Every
//! `/files/*` route requires a resolved identity too. The only anonymous
//! read of a file's content is `GET /shares/{share_id}`, and only while that
//! file's share is live and the file isn't in the bin.
//!
//! `routes()` is the one list of routes: each entry registers into both the
//! worker `Router` and the OpenAPI spec (see `routes`), so the web client
//! generated from that spec can't drift from what's actually served. Every
//! failure is an `ApiError` body.
//!
//! Helpers below (`D1_BINDING`, `is_unique_constraint_violation`,
//! `resolve_files_caller`) are private to this module but visible to both
//! submodules per Rust's ancestor-visibility rule -- no `pub(crate)`
//! needed.

mod files;
pub(crate) mod routes;
mod shares;

use worker::{D1Database, RouteContext};

use crate::api_error::ApiError;
use crate::identity::github::GithubIdentityProvider;
use crate::identity::resolve_verified_user_id;
use crate::oauth;
use routes::{HandlerResult, Routes};

/// D1 binding name this worker expects in `wrangler.toml`. Wiring the
/// actual binding is task 5 (`TODO-synced-share-rust-d1-migration.md` §4's
/// "Update `wrangler.toml`" bullet) -- out of scope here.
const D1_BINDING: &str = "DB";

pub(crate) fn routes() -> worker::Result<Routes> {
    Routes::new()
        .get("/shares/{share_id}", shares::get_share)?
        .post("/auth/github/callback", oauth::github_oauth_callback)?
        .post("/auth/github/revoke", oauth::github_revoke)?
        .post("/files/list", files::list_files)?
        .post("/files", files::create_file)?
        .post("/files/{id}/content", files::update_file_content)?
        .post("/files/{id}/rename", files::rename_file)?
        .post("/files/{id}/delete", files::delete_file)?
        .post("/files/{id}/restore", files::restore_file)?
        .post("/files/{id}/share", shares::start_share)?
        .post("/files/{id}/share/stop", shares::stop_share)?
        .post("/files/{id}/share/status", shares::share_status)
}

/// D1 surfaces a SQLite `UNIQUE constraint failed` violation as a plain
/// stringly-typed `worker::Error`, not a structured error code -- so this
/// checks the message text.
fn is_unique_constraint_violation(error: &worker::Error) -> bool {
    error.to_string().contains("UNIQUE constraint failed")
}

/// Resolves a `/files/*` request's caller identity, same path every Synced
/// Share write already goes through -- see this module's doc comment. A
/// failed verification (GitHub call failed even after the backoff-wrapped
/// retries) becomes `ApiError::Unauthorized`, whose body is exactly
/// `VerificationFailure`'s fields -- per TODO §0, the token/hash must never
/// appear there, which `VerificationFailure` enforces by construction.
async fn resolve_files_caller(
    d1: &D1Database,
    ctx: &RouteContext<()>,
    identity_token: &str,
) -> HandlerResult<String> {
    let identity_provider = GithubIdentityProvider::from_env(ctx);
    resolve_verified_user_id(d1, &identity_provider, identity_token)
        .await?
        .map_err(ApiError::Unauthorized)
}
