//! The test-only `POST /e2e/reset` route handler -- see `crate::e2e_reset`
//! for why it exists and how it's gated.

use worker::RouteContext;

use crate::api_error::ApiError;
use crate::db;
use crate::e2e_reset::{emails_to_reset, parse_reset_emails, EMAILS_VAR};
use crate::protocol::{E2eResetRequest, E2eResetResponse};

use super::routes::{HandlerResult, Json, NoPathParams};
use super::D1_BINDING;

/// `POST /e2e/reset` -- deletes the accounts of the emails listed in
/// `EMAILS_VAR` (or of the requested subset of them), with everything they
/// own. Only registered when that var is set (see `super::routes`), so
/// reading it here can only fail on a malformed value -- which fails the
/// request rather than silently resetting nothing.
pub(super) async fn reset(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: E2eResetRequest,
) -> HandlerResult<Json<E2eResetResponse>> {
    let value = ctx.var(EMAILS_VAR)?.to_string();
    let allowed = parse_reset_emails(&value).map_err(|error| ApiError::Internal {
        message: format!("{EMAILS_VAR} is not a JSON array of emails: {error}"),
    })?;
    let emails =
        emails_to_reset(&allowed, body.emails.as_deref()).map_err(|_| ApiError::BadRequest)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let deleted = db::delete_accounts(&d1, &emails).await?;
    Ok(Json(E2eResetResponse {
        deleted_accounts: deleted.accounts,
        deleted_shares: deleted.shares,
        deleted_files: deleted.files,
    }))
}
