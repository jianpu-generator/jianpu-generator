//! Synced Share route handlers -- see the parent module's doc comment.
//!
//! A share points at a cloud `files` row (see `crate::share`), so there is
//! no share-side write of content at all: the owner's ordinary autosave is
//! what a viewer sees. These routes only start, stop and report on the
//! pointer itself.

use worker::RouteContext;

use crate::api_error::ApiError;
use crate::db;
use crate::protocol::{
    FileIdPath, FileShareRequest, FileShareResponse, ShareIdPath, ShareStatusResponse, SyncedDoc,
};
use crate::share;
use crate::share_id;

use super::routes::{HandlerResult, Json, NoContent};
use super::{resolve_files_caller, D1_BINDING};

/// `GET /shares/{share_id}` -- fully anonymous: the unguessable `share_id`
/// is the sole credential needed to view (TODO §0). Reads the pointed-at
/// file directly; an ended share returns no content (see
/// `share::to_public_doc`). Never cached -- see `routes::respond`.
pub(super) async fn get_share(
    ctx: RouteContext<()>,
    path: ShareIdPath,
) -> HandlerResult<Json<SyncedDoc>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let view = db::get_share_view(&d1, &path.share_id).await?;
    Ok(Json(share::to_public_doc(view.as_ref())))
}

/// `POST /files/{id}/share` -- starts (or resumes) sharing one of the
/// caller's active files. Idempotent: a file that was ever shared keeps its
/// existing `share_id` (`queries/upsert_share.sql`'s `ON CONFLICT
/// (file_id)`), so re-sharing always reproduces the same link, from any
/// device.
pub(super) async fn start_share(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: FileShareRequest,
) -> HandlerResult<Json<FileShareResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let is_active_owned_file = db::get_file_by_id(&d1, &path.id, &owner_user_id)
        .await?
        .is_some_and(|file| file.trashed_at.is_none());
    if !is_active_owned_file {
        return Err(ApiError::NotFound);
    }

    let new_share_id = share_id::generate_unique_share_id(&d1).await?;
    let now = worker::Date::now().as_millis() as i64;
    let Some(share_id) = db::upsert_share(&d1, &new_share_id, &path.id, now).await? else {
        return Err(ApiError::Internal {
            message: "upserting the share returned no row".to_string(),
        });
    };
    Ok(Json(FileShareResponse { share_id }))
}

/// `POST /files/{id}/share/stop` -- ends the caller's share of this file.
/// The row is kept so a later start reproduces the same link. `NoContent`,
/// or `NotFound` when the caller has no share for this file.
pub(super) async fn stop_share(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: FileShareRequest,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let now = worker::Date::now().as_millis() as i64;
    if db::end_share(&d1, &path.id, &owner_user_id, now).await? {
        Ok(NoContent)
    } else {
        Err(ApiError::NotFound)
    }
}

/// `POST /files/{id}/share/status` -- whether the caller's file is shared,
/// and under which link. The server is the single source of truth, so every
/// device the owner signs in on sees the same live/stopped state.
pub(super) async fn share_status(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: FileShareRequest,
) -> HandlerResult<Json<ShareStatusResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let share = db::get_share_status_by_file(&d1, &path.id, &owner_user_id).await?;
    Ok(Json(ShareStatusResponse { share }))
}
