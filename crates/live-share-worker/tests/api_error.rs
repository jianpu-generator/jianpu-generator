//! Wire-shape and status tests for `api_error::ApiError`, the one failure
//! body every route answers with.
#![allow(clippy::disallowed_macros)]

use live_share_worker::api_error::ApiError;
use live_share_worker::verification::VerificationFailure;

#[test]
fn serializes_a_unit_variant_as_just_its_code() -> Result<(), serde_json::Error> {
    assert_eq!(
        serde_json::to_string(&ApiError::NameTaken)?,
        r#"{"code":"name_taken"}"#
    );
    Ok(())
}

#[test]
fn serializes_a_revision_conflict_with_a_camel_case_revision() -> Result<(), serde_json::Error> {
    let error = ApiError::RevisionConflict {
        current_revision: 7,
    };

    assert_eq!(
        serde_json::to_string(&error)?,
        r#"{"code":"revision_conflict","currentRevision":7}"#
    );
    Ok(())
}

#[test]
fn flattens_a_verification_failure_next_to_its_code() -> Result<(), serde_json::Error> {
    let error = ApiError::Unauthorized(VerificationFailure {
        reason: "GitHub said no".to_string(),
        failed_at: 1,
        attempts: 3,
    });

    assert_eq!(
        serde_json::to_string(&error)?,
        r#"{"code":"unauthorized","reason":"GitHub said no","failedAt":1,"attempts":3}"#
    );
    Ok(())
}

struct StatusCase {
    error: ApiError,
    status: u16,
}

#[test]
fn derives_each_status_code_from_the_variant() {
    let failure = VerificationFailure {
        reason: "boom".to_string(),
        failed_at: 0,
        attempts: 1,
    };
    let cases = [
        StatusCase {
            error: ApiError::BadRequest,
            status: 400,
        },
        StatusCase {
            error: ApiError::Unauthorized(failure),
            status: 401,
        },
        StatusCase {
            error: ApiError::NotFound,
            status: 404,
        },
        StatusCase {
            error: ApiError::NameTaken,
            status: 409,
        },
        StatusCase {
            error: ApiError::RevisionConflict {
                current_revision: 1,
            },
            status: 409,
        },
        StatusCase {
            error: ApiError::Internal {
                message: "boom".to_string(),
            },
            status: 500,
        },
        StatusCase {
            error: ApiError::UpstreamFailed {
                message: "boom".to_string(),
            },
            status: 502,
        },
    ];

    for case in cases {
        assert_eq!(case.error.status_code(), case.status, "{:?}", case.error);
    }
}
