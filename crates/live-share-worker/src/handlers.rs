//! HTTP routing for the Synced Share worker, ported from the old TS
//! `live-share-worker/src/index.ts`. CORS and the `OPTIONS` preflight are
//! handled once in `lib.rs` around whatever this router returns; this
//! module only owns the `/shares` routes themselves.

use worker::{Request, RouteContext, Router};
use worker::{Response, Result};

use crate::db;
use crate::doc;
use crate::identity::stub::StubIdentityProvider;
use crate::identity::IdentityProvider;
use crate::protocol::{CreateShareRequest, CreateShareResponse, SyncedWriteRequest};
use crate::share_id;

/// D1 binding name this worker expects in `wrangler.toml`. Wiring the
/// actual binding is task 5 (`TODO-synced-share-rust-d1-migration.md` §4's
/// "Update `wrangler.toml`" bullet) -- out of scope here.
const D1_BINDING: &str = "DB";

pub(crate) fn router() -> Router<'static, ()> {
    Router::new()
        .get_async("/shares/:share_id", get_share)
        .post_async("/shares/:share_id", post_share)
        .post_async("/shares", create_share)
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
    Response::from_json(&doc::to_public_doc(existing.as_ref()))
}

/// `POST /shares` -- the new "create share" endpoint (TODO §1): generates a
/// server-side `share_id` and creates its `docs` row bound to the resolved
/// identity. Gated on identity resolution succeeding, same as every write;
/// today that resolution is the stub, not real GitHub verification (task
/// 6/7).
async fn create_share(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Ok(body) = req.json::<CreateShareRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = StubIdentityProvider
        .resolve_user_id(&d1, &body.identity_token)
        .await?;

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
    // resolve it, even though today's resolution is the insecure stub.
    let resolved_user_id = StubIdentityProvider
        .resolve_user_id(&d1, body.identity_token())
        .await?;
    let now = worker::Date::now().as_millis() as i64;

    match doc::apply_write(&existing, Some(&resolved_user_id), &body, now) {
        Ok(updated) => {
            db::update_doc(&d1, &updated).await?;
            Ok(Response::empty()?.with_status(204))
        }
        Err(doc::WriteError::Forbidden) => Response::error("Forbidden", 403),
    }
}
