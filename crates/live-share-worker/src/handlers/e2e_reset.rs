//! The test-only `POST /e2e/reset` route handler -- see `crate::e2e_reset`
//! for why it exists and how it's gated.

use worker::RouteContext;

use crate::api_error::ApiError;
use crate::db;
use crate::e2e_reset::{parse_github_user_ids, GITHUB_USER_IDS_VAR};
use crate::identity::github::GITHUB_PROVIDER;
use crate::protocol::{E2eResetRequest, E2eResetResponse};

use super::routes::{HandlerResult, Json, NoPathParams};
use super::D1_BINDING;

/// `POST /e2e/reset` -- hard-deletes every file and share owned by the
/// GitHub accounts listed in `GITHUB_USER_IDS_VAR`. Only registered when
/// that var is set (see `super::routes`), so reading it here can only fail
/// on a malformed value -- which fails the request rather than silently
/// resetting nothing.
pub(super) async fn reset(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    _body: E2eResetRequest,
) -> HandlerResult<Json<E2eResetResponse>> {
    let value = ctx.var(GITHUB_USER_IDS_VAR)?.to_string();
    let github_user_ids = parse_github_user_ids(&value).map_err(|error| ApiError::Internal {
        message: format!("{GITHUB_USER_IDS_VAR} is not a JSON array of GitHub user ids: {error}"),
    })?;
    let d1 = ctx.d1(D1_BINDING)?;
    let deleted = db::delete_rows_of_provider_users(&d1, GITHUB_PROVIDER, &github_user_ids).await?;
    Ok(Json(E2eResetResponse {
        deleted_shares: deleted.shares,
        deleted_files: deleted.files,
    }))
}
