//! `/files/*` route handlers (the cloud storage backend) -- see the parent
//! module's doc comment.

use worker::{Request, RouteContext};
use worker::{Response, Result};

use crate::db;
use crate::files::{self, ContentWriteAttempt, ContentWriteOutcome};
use crate::protocol::{
    ConflictResponse, CreateFileRequest, DeleteFileRequest, ListFilesRequest, ListFilesResponse,
    RenameFileRequest, RestoreFileRequest, UpdateFileContentRequest, UpdateFileContentResponse,
};

use super::{
    is_unique_constraint_violation, name_taken_response, resolve_files_caller,
    verification_failure_response, D1_BINDING,
};

/// `POST /files/list` -- every file (active and trashed alike) owned by the
/// resolved caller; the client partitions the result by `trashedAt` (see
/// `crate::protocol::ListFilesResponse`'s doc comment).
pub(super) async fn list_files(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Ok(body) = req.json::<ListFilesRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let stored = db::list_files_by_owner(&d1, &owner_user_id).await?;
    let files: Vec<_> = stored.iter().map(files::to_public_file).collect();
    Response::from_json(&ListFilesResponse { files })
}

/// `POST /files` -- creates a new file row (new file / duplicate / import).
/// See `crate::protocol::CreateFileRequest`'s doc comment for the response
/// shapes.
pub(super) async fn create_file(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Ok(body) = req.json::<CreateFileRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

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
        Ok(()) => Response::from_json(&files::to_public_file(&created)),
        Err(error) if is_unique_constraint_violation(&error) => name_taken_response(),
        Err(error) => Err(error),
    }
}

/// `POST /files/:id/content` -- the atomic CAS content save. `200
/// {revision}` on success, `409 {currentRevision}` when the row moved on
/// since the caller last saw it (or was trashed out from under the write),
/// `404` when the file doesn't exist or isn't the caller's -- see
/// `crate::files::classify_content_write`.
pub(super) async fn update_file_content(
    mut req: Request,
    ctx: RouteContext<()>,
) -> Result<Response> {
    let Some(id) = ctx.param("id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<UpdateFileContentRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let now = worker::Date::now().as_millis() as i64;
    let attempt = db::update_file_content(
        &d1,
        &id,
        &owner_user_id,
        &body.content,
        body.expected_revision,
        now,
    )
    .await?;
    let current = match attempt {
        ContentWriteAttempt::Applied => None,
        ContentWriteAttempt::RowsAffectedZero => {
            db::get_file_by_id(&d1, &id, &owner_user_id).await?
        }
    };

    match files::classify_content_write(attempt, body.expected_revision, current.as_ref()) {
        ContentWriteOutcome::Applied { new_revision } => {
            Response::from_json(&UpdateFileContentResponse {
                revision: new_revision,
            })
        }
        ContentWriteOutcome::Conflict { current_revision } => {
            Ok(Response::from_json(&ConflictResponse { current_revision })?.with_status(409))
        }
        ContentWriteOutcome::NotFound => Response::error("Not Found", 404),
    }
}

/// `POST /files/:id/rename` -- atomic rename, no revision gate (matches the
/// old GitHub backend's actual behavior, decision #3). `404` on zero rows
/// affected; a name collision maps to `409 {code: "name_taken"}`.
pub(super) async fn rename_file(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = ctx.param("id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<RenameFileRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let now = worker::Date::now().as_millis() as i64;
    match db::rename_file(&d1, &id, &owner_user_id, &body.name, now).await {
        Ok(true) => Ok(Response::empty()?.with_status(204)),
        Ok(false) => Response::error("Not Found", 404),
        Err(error) if is_unique_constraint_violation(&error) => name_taken_response(),
        Err(error) => Err(error),
    }
}

/// `POST /files/:id/delete` -- atomic move to the bin. `404` on zero rows
/// affected (already trashed, wrong owner, or never existed).
pub(super) async fn delete_file(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = ctx.param("id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<DeleteFileRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let now = worker::Date::now().as_millis() as i64;
    if db::trash_file(&d1, &id, &owner_user_id, now).await? {
        Ok(Response::empty()?.with_status(204))
    } else {
        Response::error("Not Found", 404)
    }
}

/// `POST /files/:id/restore` -- atomic restore out of the bin. `404` on
/// zero rows affected (no such trashed file for this caller); a name
/// collision (against the caller's other files) maps to `409 {code:
/// "name_taken"}`, same as `create_file`/`rename_file`.
pub(super) async fn restore_file(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(id) = ctx.param("id").cloned() else {
        return Response::error("Not Found", 404);
    };
    let Ok(body) = req.json::<RestoreFileRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = match resolve_files_caller(&d1, &ctx, &body.identity_token).await? {
        Ok(user_id) => user_id,
        Err(failure) => return verification_failure_response(&failure),
    };

    let now = worker::Date::now().as_millis() as i64;
    match db::restore_file(&d1, &id, &owner_user_id, &body.name, now).await {
        Ok(true) => Ok(Response::empty()?.with_status(204)),
        Ok(false) => Response::error("Not Found", 404),
        Err(error) if is_unique_constraint_violation(&error) => name_taken_response(),
        Err(error) => Err(error),
    }
}
