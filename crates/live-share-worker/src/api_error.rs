//! The one error body every route answers a failure with. A tagged union
//! (`code`) rather than ad-hoc status/body pairs, so the web client's
//! generated types (see `crate::handlers::routes`) let it `switch` on
//! `code` exhaustively instead of re-matching raw HTTP statuses or
//! hand-typed `{code: "..."}` literals. The HTTP status is derived from the
//! variant (`status_code`), never chosen separately at a call site.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::verification::VerificationFailure;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ApiError {
    /// The request body or a path parameter didn't parse.
    BadRequest,
    /// No such row for the caller (or not the caller's).
    NotFound,
    /// A create/rename/restore collided with another of the caller's file
    /// names (`idx_files_owner_name`).
    NameTaken,
    /// A content save's `expectedRevision` lost the race -- feeds the
    /// "Overwrite mine"/"Discard mine" UI with the row's actual revision.
    RevisionConflict {
        #[serde(rename = "currentRevision")]
        current_revision: i64,
    },
    /// GitHub verification of the caller's identity token failed, after
    /// retries. Carries exactly `VerificationFailure`'s fields, which by
    /// construction never include the token or its hash.
    Unauthorized(VerificationFailure),
    /// A call this worker makes to GitHub itself failed.
    UpstreamFailed { message: String },
    /// Anything else (a D1 error, ...).
    Internal { message: String },
}

impl ApiError {
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest => 400,
            ApiError::Unauthorized(_) => 401,
            ApiError::NotFound => 404,
            ApiError::NameTaken | ApiError::RevisionConflict { .. } => 409,
            ApiError::Internal { .. } => 500,
            ApiError::UpstreamFailed { .. } => 502,
        }
    }
}

impl From<worker::Error> for ApiError {
    fn from(error: worker::Error) -> Self {
        ApiError::Internal {
            message: error.to_string(),
        }
    }
}
