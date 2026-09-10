//! HTTP routing for the Synced Share worker, ported from the old TS
//! `live-share-worker/src/index.ts`. CORS and the `OPTIONS` preflight are
//! handled once in `lib.rs` around whatever this router returns; this
//! module owns the `/shares` routes plus the `/auth/github/callback` route
//! (task 6, see `crate::oauth`) added for the dedicated Synced Share
//! sign-in connection.
//!
//! Every write (and the create-share endpoint) resolves its caller's
//! identity via `identity::resolve_verified_user_id`, fronted by
//! `identity::github::GithubIdentityProvider` -- never the client's own
//! claim -- per `TODO-synced-share-rust-d1-migration.md` §0/§6 (task 7). A
//! verification failure (after the backoff-wrapped retries) fails the
//! request closed with a structured `verification::VerificationFailure`
//! body (401): reason, timestamp, attempt count, and deliberately nothing
//! else -- the token/hash must never appear in this response.

use worker::{Request, RouteContext, Router};
use worker::{Response, Result};

use crate::db;
use crate::doc;
use crate::identity::github::GithubIdentityProvider;
use crate::identity::resolve_verified_user_id;
use crate::oauth;
use crate::protocol::{CreateShareRequest, CreateShareResponse, SyncedWriteRequest};
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

/// `POST /shares` -- the new "create share" endpoint (TODO §1): generates a
/// server-side `share_id` and creates its `docs` row bound to the resolved
/// identity. Gated on identity resolution succeeding, same as every write --
/// see the module doc comment for the verification path.
async fn create_share(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Ok(body) = req.json::<CreateShareRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id =
        match resolve_verified_user_id(&d1, &GithubIdentityProvider, &body.identity_token).await? {
            Ok(user_id) => user_id,
            Err(failure) => return verification_failure_response(&failure),
        };

    let share_id = share_id::generate_unique_share_id(&d1).await?;
    let now = worker::Date::now().as_millis() as i64;
    let created = doc::StoredDoc {
        share_id: share_id.clone(),
        owner_user_id,
        filename: String::new(),
        content: String::new(),
        revision: 0,
        ended: false,
        created_at: now,
        updated_at: now,
    };
    db::insert_doc(&d1, &created).await?;

    Response::from_json(&CreateShareResponse { share_id })
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
    let resolved_user_id = match resolve_verified_user_id(
        &d1,
        &GithubIdentityProvider,
        body.identity_token(),
    )
    .await?
    {
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
