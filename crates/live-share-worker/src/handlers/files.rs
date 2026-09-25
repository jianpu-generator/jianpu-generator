//! `/files/*` route handlers (the cloud storage backend) -- see the parent
//! module's doc comment.

use worker::RouteContext;

use crate::api_error::ApiError;
use crate::db;
use crate::files::{self, ContentWriteAttempt, ContentWriteOutcome, PublicFile};
use crate::protocol::{
    CreateFileRequest, DeleteFileRequest, FileIdPath, ListFilesRequest, ListFilesResponse,
    RenameFileRequest, RestoreFileRequest, UpdateFileContentRequest, UpdateFileContentResponse,
};

use super::routes::{HandlerResult, Json, NoContent, NoPathParams};
use super::{is_unique_constraint_violation, resolve_files_caller, D1_BINDING};

/// `POST /files/list` -- every file (active and trashed alike) owned by the
/// resolved caller; the client partitions the result by `trashedAt` (see
/// `crate::protocol::ListFilesResponse`'s doc comment).
pub(super) async fn list_files(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: ListFilesRequest,
) -> HandlerResult<Json<ListFilesResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let stored = db::list_files_by_owner(&d1, &owner_user_id).await?;
    let files = stored.iter().map(files::to_public_file).collect();
    Ok(Json(ListFilesResponse { files }))
}

/// `POST /files` -- creates a new file row (new file / duplicate / import).
/// See `crate::protocol::CreateFileRequest`'s doc comment for the response
/// shapes.
pub(super) async fn create_file(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: CreateFileRequest,
) -> HandlerResult<Json<PublicFile>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let now = worker::Date::now().as_millis() as i64;
    let created = files::StoredFile {
        id: body.id,
        owner_user_id,
        name: body.name,
        content: body.content,
        revision: 0,
        trashed_at: None,
        created_at: now,
        updated_at: now,
    };
    match db::insert_file(&d1, &created).await {
        Ok(()) => Ok(Json(files::to_public_file(&created))),
        Err(error) if is_unique_constraint_violation(&error) => Err(ApiError::NameTaken),
        Err(error) => Err(error.into()),
    }
}

/// `POST /files/{id}/content` -- the atomic CAS content save. `{revision}`
/// on success, `ApiError::RevisionConflict` when the row moved on since the
/// caller last saw it (or was trashed out from under the write),
/// `ApiError::NotFound` when the file doesn't exist or isn't the caller's --
/// see `crate::files::classify_content_write`.
pub(super) async fn update_file_content(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: UpdateFileContentRequest,
) -> HandlerResult<Json<UpdateFileContentResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let now = worker::Date::now().as_millis() as i64;
    let attempt = db::update_file_content(
        &d1,
        &path.id,
        &owner_user_id,
        &body.content,
        body.expected_revision,
        now,
    )
    .await?;
    let current = match attempt {
        ContentWriteAttempt::Applied => None,
        ContentWriteAttempt::RowsAffectedZero => {
            db::get_file_by_id(&d1, &path.id, &owner_user_id).await?
        }
    };

    match files::classify_content_write(attempt, body.expected_revision, current.as_ref()) {
        ContentWriteOutcome::Applied { new_revision } => Ok(Json(UpdateFileContentResponse {
            revision: new_revision,
        })),
        ContentWriteOutcome::Conflict { current_revision } => {
            Err(ApiError::RevisionConflict { current_revision })
        }
        ContentWriteOutcome::NotFound => Err(ApiError::NotFound),
    }
}

/// `POST /files/{id}/rename` -- atomic rename, no revision gate (matches the
/// old GitHub backend's actual behavior, decision #3). `NotFound` on zero
/// rows affected; a name collision maps to `ApiError::NameTaken`.
pub(super) async fn rename_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: RenameFileRequest,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let now = worker::Date::now().as_millis() as i64;
    match db::rename_file(&d1, &path.id, &owner_user_id, &body.name, now).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(ApiError::NotFound),
        Err(error) if is_unique_constraint_violation(&error) => Err(ApiError::NameTaken),
        Err(error) => Err(error.into()),
    }
}

/// `POST /files/{id}/delete` -- atomic move to the bin. `NotFound` on zero
/// rows affected (already trashed, wrong owner, or never existed).
pub(super) async fn delete_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: DeleteFileRequest,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let now = worker::Date::now().as_millis() as i64;
    if db::trash_file(&d1, &path.id, &owner_user_id, now).await? {
        Ok(NoContent)
    } else {
        Err(ApiError::NotFound)
    }
}

/// `POST /files/{id}/restore` -- atomic restore out of the bin. `NotFound`
/// on zero rows affected (no such trashed file for this caller); a name
/// collision (against the caller's other files) maps to
/// `ApiError::NameTaken`, same as `create_file`/`rename_file`.
pub(super) async fn restore_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: RestoreFileRequest,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &ctx, &body.identity_token).await?;

    let now = worker::Date::now().as_millis() as i64;
    match db::restore_file(&d1, &path.id, &owner_user_id, &body.name, now).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(ApiError::NotFound),
        Err(error) if is_unique_constraint_violation(&error) => Err(ApiError::NameTaken),
        Err(error) => Err(error.into()),
    }
}
