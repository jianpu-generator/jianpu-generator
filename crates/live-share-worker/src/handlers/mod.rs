//! HTTP routing for the Synced Share worker, ported from the old TS
//! `live-share-worker/src/index.ts`. CORS and the `OPTIONS` preflight are
//! handled once in `lib.rs` around whatever this router returns; this
//! module owns the Synced Share routes (`GET /shares/{share_id}` and
//! `POST /files/{id}/share{,/stop,/status}`, `POST /folders/share{,/stop,/status}`
//! and `POST /shares/overview`; `shares` submodule), the
//! `/files` routes (the cloud storage backend, see `crate::files`; `files`
//! submodule), plus the account routes `/auth/sign-up`, `/auth/sign-in`,
//! `/auth/sign-out` and `/auth/change-password` (`auth` submodule).
//!
//! Every write (including starting/stopping a share) resolves its caller's
//! identity via `identity::resolve_verified_user_id` -- a session lookup,
//! never the client's own claim. A token that names no live session fails
//! the request closed with `ApiError::Unauthorized` (401), carrying a
//! `auth::session::VerificationFailure`: reason, timestamp, attempt count,
//! and deliberately nothing else -- the token/hash must never appear in
//! this response. Every `/files/*` route requires a resolved identity too.
//! The only anonymous read of a file's content is `GET /shares/{share_id}`,
//! and only while that file's share is live and the file isn't in the bin.
//!
//! `routes()` is the one list of routes: each entry registers into both the
//! worker `Router` and the OpenAPI spec (see `routes`), so the web client
//! generated from that spec can't drift from what's actually served. Every
//! failure is an `ApiError` body.
//!
//! Helpers below (`is_unique_constraint_violation`, `resolve_files_caller`)
//! are private to this module but visible to every submodule per Rust's
//! ancestor-visibility rule -- no `pub(crate)` needed. `D1_BINDING` is
//! `pub(crate)` since `lib.rs`'s scheduled handler writes through it too.

mod auth;
mod e2e_reset;
mod files;
pub(crate) mod routes;
mod shares;

use worker::D1Database;

use crate::api_error::ApiError;
use crate::app_log::RequestMeta;
use crate::identity::resolve_verified_user_id;
use routes::{HandlerResult, Routes};

/// D1 binding name this worker expects in `deploy.json`/`cloudflare.config.ts`. Wiring the
/// actual binding is task 5 (`TODO-synced-share-rust-d1-migration.md` §4's
/// "Update `deploy.json`/`cloudflare.config.ts`" bullet) -- out of scope here.
pub(crate) const D1_BINDING: &str = "DB";

/// Every route, for both serving and the spec. `e2e_reset_enabled` (see
/// `crate::e2e_reset`) decides only whether `POST /e2e/reset` is served;
/// the spec always documents it, so the e2e setup calls it through the
/// generated client.
pub(crate) fn routes(e2e_reset_enabled: bool) -> worker::Result<Routes> {
    Routes::new()
        .get("/shares/{share_id}", shares::get_share)?
        .post("/auth/sign-up", auth::sign_up)?
        .post("/auth/sign-in", auth::sign_in)?
        .post("/auth/sign-out", auth::sign_out)?
        .post("/auth/change-password", auth::change_password)?
        .post("/files/list", files::list_files)?
        .post("/files", files::create_file)?
        .post("/files/{id}/content", files::update_file_content)?
        .post("/files/rename-batch", files::rename_files_batch)?
        .post("/files/{id}/rename", files::rename_file)?
        .post("/files/{id}/delete", files::delete_file)?
        .post("/files/{id}/restore", files::restore_file)?
        .post("/files/{id}/share", shares::start_share)?
        .post("/files/{id}/share/stop", shares::stop_share)?
        .post("/files/{id}/share/status", shares::share_status)?
        .post("/folders/share", shares::start_folder_share)?
        .post("/folders/share/stop", shares::stop_folder_share)?
        .post("/folders/share/status", shares::folder_share_status)?
        .post("/shares/overview", shares::share_overview)?
        .post_if(e2e_reset_enabled, "/e2e/reset", e2e_reset::reset)
}

/// D1 surfaces a SQLite `UNIQUE constraint failed` violation as a plain
/// stringly-typed `worker::Error`, not a structured error code -- so this
/// checks the message text.
fn is_unique_constraint_violation(error: &worker::Error) -> bool {
    error.to_string().contains("UNIQUE constraint failed")
}

/// Resolves an authenticated request's caller identity -- see this
/// module's doc comment. A token that names no live session becomes
/// `ApiError::Unauthorized`, whose body is exactly `VerificationFailure`'s
/// fields, which never include the token or its hash.
async fn resolve_files_caller(
    d1: &D1Database,
    identity_token: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<String> {
    resolve_verified_user_id(d1, identity_token, request_meta)
        .await?
        .map_err(ApiError::Unauthorized)
}
