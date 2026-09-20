//! HTTP routing for the Synced Share worker, ported from the old TS
//! `live-share-worker/src/index.ts`. CORS and the `OPTIONS` preflight are
//! handled once in `lib.rs` around whatever this router returns; this
//! module owns the `/shares` routes plus the `/auth/github/callback` and
//! `/auth/github/revoke` routes (see `crate::oauth`) added for the
//! dedicated Synced Share sign-in connection.
//!
//! Every write (and the create-share endpoint) resolves its caller's
//! identity via `identity::resolve_verified_user_id`, fronted by
//! `identity::github::GithubIdentityProvider` -- never the client's own
//! claim -- per `TODO-synced-share-rust-d1-migration.md` §0/§6 (task 7). A
//! verification failure (after the backoff-wrapped retries) fails the
//! request closed with a structured `verification::VerificationFailure`
//! body (401): reason, timestamp, attempt count, and deliberately nothing
//! else -- the token/hash must never appear in this response.

use worker::{D1Database, Request, RouteContext, Router};
use worker::{Response, Result};

use crate::db;
use crate::doc;
use crate::identity::github::GithubIdentityProvider;
use crate::identity::resolve_verified_user_id;
use crate::oauth;
use crate::protocol::{CreateShareRequest, CreateShareResponse, SyncedWriteRequest};
use crate::share_creation::{self, CreateAttempt, ResolveShareIdError};
use crate::share_id;
use crate::verification::VerificationFailure;

/// D1 binding name this worker expects in `wrangler.toml`. Wiring the
/// actual binding is task 5 (`TODO-synced-share-rust-d1-migration.md` §4's
/// "Update `wrangler.toml`" bullet) -- out of scope here.
const D1_BINDING: &str = "DB";

pub(crate) fn router() -> Router<'static, ()> {
    Router::new()
        .get_async("/shares/:share_id", get_share)
        .post_async("/shares/:share_id", post_share)
        .post_async("/shares", create_share)
        .post_async("/auth/github/callback", oauth::github_oauth_callback)
        .post_async("/auth/github/revoke", oauth::github_revoke)
}

/// `GET /shares/:share_id` -- fully anonymous, matching the old behavior:
/// the unguessable `share_id` is the sole credential needed to view (TODO
/// §0).
async fn get_share(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(share_id) = ctx.param("share_id").cloned() else {
        return Response::error("Not Found", 404);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let existing = db::get_doc_by_share_id(&d1, &share_id).await?;
    let owner_login = match existing.as_ref() {
        Some(doc) => db::get_owner_login(&d1, &doc.owner_user_id).await?,
        None => None,
    };
    Response::from_json(&doc::to_public_doc(existing.as_ref(), owner_login))
}

/// `POST /shares` -- the "create share" endpoint (TODO §1): generates a
/// server-side `share_id` and creates its `docs` row bound to the resolved
/// identity. Gated on identity resolution succeeding, same as every write --
/// see the module doc comment for the verification path.
///
/// Idempotent per `(owner_user_id, external_file_id)` when the request
/// carries an `external_file_id` (a GitHub-backed file, per migration
/// 0002): re-sharing the same file as the same owner reproduces the same
/// `share_id` instead of minting a second one -- see
/// `crate::share_creation::resolve_share_id`. A request with no
/// `external_file_id` (a local-only file) always mints a fresh share,
/// unchanged from before.
async fn create_share(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Ok(body) = req.json::<CreateShareRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let identity_provider = GithubIdentityProvider::from_env(&ctx);
    let owner_user_id =
        match resolve_verified_user_id(&d1, &identity_provider, &body.identity_token).await? {
            Ok(user_id) => user_id,
            Err(failure) => return verification_failure_response(&failure),
        };

    let external_file_id = body.external_file_id.as_deref();
    let share_id = match share_creation::resolve_share_id(
        external_file_id,
        || {
            db::get_doc_by_owner_and_external_file(
                &d1,
                &owner_user_id,
                external_file_id.unwrap_or_default(),
            )
        },
        || create_doc(&d1, &owner_user_id, external_file_id),
    )
    .await
    {
        Ok(share_id) => share_id,
        Err(ResolveShareIdError::Upstream(error)) => return Err(error),
        // Both remaining variants are states `resolve_share_id` doesn't
        // assume away just because they're not supposed to happen -- see
        // its doc comment. Neither is a client mistake, so this reports a
        // plain 500 rather than trying to guess a more specific status.
        Err(ResolveShareIdError::LocalCreateConflicted)
        | Err(ResolveShareIdError::ConflictWithNoWinner) => {
            return Response::error("Internal Server Error", 500)
        }
    };

    Response::from_json(&CreateShareResponse { share_id })
}

/// Mints a fresh `share_id` and attempts to insert its `docs` row, reporting
/// `CreateAttempt::Conflict` instead of propagating a D1 error when the
/// insert loses the race against a concurrent create for the same
/// `(owner_user_id, external_file_id)` -- see
/// `migrations/0002_docs_external_file_id.sql`'s partial unique index and
/// `share_creation::resolve_share_id`'s doc comment for how the caller
/// reacts to a `Conflict`.
async fn create_doc(
    d1: &D1Database,
    owner_user_id: &str,
    external_file_id: Option<&str>,
) -> Result<CreateAttempt> {
    let share_id = share_id::generate_unique_share_id(d1).await?;
    let now = worker::Date::now().as_millis() as i64;
    let created = doc::StoredDoc {
        share_id: share_id.clone(),
        owner_user_id: owner_user_id.to_string(),
        filename: String::new(),
        content: String::new(),
        revision: 0,
        ended: false,
        created_at: now,
        updated_at: now,
        external_file_id: external_file_id.map(str::to_string),
    };
    match db::insert_doc(d1, &created).await {
        Ok(()) => Ok(CreateAttempt::Created(share_id)),
        Err(error) if is_unique_constraint_violation(&error) => Ok(CreateAttempt::Conflict),
        Err(error) => Err(error),
    }
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

/// `POST /shares/:share_id` -- applies an update or a stop. The share must
/// already exist (created via `create_share` above); unlike the old TS
/// `index.ts`, a write no longer implicitly creates the share.
async fn post_share(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(share_id) = ctx.param("share_id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<SyncedWriteRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let Some(existing) = db::get_doc_by_share_id(&d1, &share_id).await? else {
        return Response::error("Not Found", 404);
    };

    // Never trust a client-asserted identity directly (TODO §6) -- always
    // resolve it through the verified, cached path.
    let identity_provider = GithubIdentityProvider::from_env(&ctx);
    let resolved_user_id =
        match resolve_verified_user_id(&d1, &identity_provider, body.identity_token()).await? {
            Ok(user_id) => user_id,
            Err(failure) => return verification_failure_response(&failure),
        };
    let now = worker::Date::now().as_millis() as i64;

    match doc::apply_write(&existing, Some(&resolved_user_id), &body, now) {
        Ok(updated) => {
            db::update_doc(&d1, &updated).await?;
            Ok(Response::empty()?.with_status(204))
        }
        Err(doc::WriteError::Forbidden) => Response::error("Forbidden", 403),
    }
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
