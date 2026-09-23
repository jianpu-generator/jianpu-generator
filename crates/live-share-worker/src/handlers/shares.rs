//! Synced Share route handlers -- see the parent module's doc comment.
//!
//! A share points at a cloud `files` row (see `crate::share`), so there is
//! no share-side write of content at all: the owner's ordinary autosave is
//! what a viewer sees. These routes only start, stop and report on the
//! pointer itself.

use worker::{Request, RouteContext};
use worker::{Response, Result};

use crate::db;
use crate::protocol::{FileShareRequest, FileShareResponse, ShareStatusResponse};
use crate::share;
use crate::share_id;

use super::{resolve_files_caller, verification_failure_response, D1_BINDING};

/// `GET /shares/:share_id` -- fully anonymous: the unguessable `share_id` is
/// the sole credential needed to view (TODO §0). Reads the pointed-at file
/// directly; an ended share returns no content (see
/// `share::to_public_doc`).
pub(super) async fn get_share(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(share_id) = ctx.param("share_id").cloned() else {
        return Response::error("Not Found", 404);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let view = db::get_share_view(&d1, &share_id).await?;
    let mut response = Response::from_json(&share::to_public_doc(view.as_ref()))?;
    // A viewer's page reload is the only way they ever see an owner's later
    // edit (see `useSyncedShareViewer.ts`'s doc comment) -- letting the
    // browser cache this response would mean that reload sometimes doesn't
    // actually re-fetch, forcing repeated reloads before the update shows.
    response.headers_mut().set("Cache-Control", "no-store")?;
    Ok(response)
}

/// `POST /files/:id/share` -- starts (or resumes) sharing one of the
/// caller's active files. Idempotent: a file that was ever shared keeps its
/// existing `share_id` (`queries/upsert_share.sql`'s `ON CONFLICT
/// (file_id)`), so re-sharing always reproduces the same link, from any
/// device.
pub(super) async fn start_share(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(file_id) = ctx.param("id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<FileShareRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let is_active_owned_file = db::get_file_by_id(&d1, &file_id, &owner_user_id)
        .await?
        .is_some_and(|file| file.trashed_at.is_none());
    if !is_active_owned_file {
        return Response::error("Not Found", 404);
    }

    let new_share_id = share_id::generate_unique_share_id(&d1).await?;
    let now = worker::Date::now().as_millis() as i64;
    let Some(share_id) = db::upsert_share(&d1, &new_share_id, &file_id, now).await? else {
        return Response::error("Internal Server Error", 500);
    };
    Response::from_json(&FileShareResponse { share_id })
}

/// `POST /files/:id/share/stop` -- ends the caller's share of this file.
/// The row is kept so a later start reproduces the same link. `204`, or
/// `404` when the caller has no share for this file.
pub(super) async fn stop_share(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(file_id) = ctx.param("id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<FileShareRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let now = worker::Date::now().as_millis() as i64;
    if db::end_share(&d1, &file_id, &owner_user_id, now).await? {
        Ok(Response::empty()?.with_status(204))
    } else {
        Response::error("Not Found", 404)
    }
}

/// `POST /files/:id/share/status` -- whether the caller's file is shared,
/// and under which link. The server is the single source of truth, so every
/// device the owner signs in on sees the same live/stopped state.
pub(super) async fn share_status(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(file_id) = ctx.param("id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<FileShareRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let share = db::get_share_status_by_file(&d1, &file_id, &owner_user_id).await?;
    Response::from_json(&ShareStatusResponse { share })
}
