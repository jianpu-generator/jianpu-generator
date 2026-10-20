//! Typed events for the `app_log` table (migration `0006_app_log.sql`): an
//! append-only log in D1 that agents and the owner query with SQL (the saved
//! queries in `queries/observability/`) so production behaviour, e.g. why a
//! user got signed out, can be explained from data instead of guessed.
//!
//! The table is generic (a `flow` column, not a table per flow) and only sees
//! text; the type safety lives here. `LogEvent` is the only way to describe a
//! row: `LogEvent::to_row` splits it into the `flow`/`kind`/`outcome`/
//! `subject`/`detail` columns, deriving `outcome` and `subject` from the
//! variant so they can never contradict it.
//!
//! D1/JsValue-free and `pub` so `tests/app_log.rs` can unit-test it; the D1
//! writes live in the crate-private `db::app_log`.

use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// How long a row is kept before the daily scheduled handler prunes it.
pub const APP_LOG_RETENTION_MILLIS: i64 = 30 * 24 * 60 * 60 * 1000;

/// Rows deleted per `prune_app_log.sql` statement, so one prune never turns
/// into a single huge D1 write.
pub const APP_LOG_PRUNE_BATCH_SIZE: u32 = 5000;

/// Upper bound on prune batches per scheduled run; whatever is left over is
/// picked up by the next day's run.
pub const APP_LOG_PRUNE_MAX_BATCHES: u32 = 10;

/// The git SHA the worker was built from (see `build.rs`), stored in every
/// row's `worker_version` so rows map straight to the deployed commit.
pub const WORKER_VERSION: &str = env!("WORKER_GIT_SHA");

const TOKEN_TAG_LENGTH: usize = 12;

/// Identifies a token without storing it: the first 12 hex chars of its
/// SHA-256 hash (`identity::hash_token`'s output).
pub fn token_tag(token_hash: &str) -> String {
    token_hash.chars().take(TOKEN_TAG_LENGTH).collect()
}

/// Rows with `at` before this are pruned.
pub fn prune_cutoff_millis(now_millis: i64) -> i64 {
    now_millis - APP_LOG_RETENTION_MILLIS
}

/// One `app_log` row's worth of event.
///
/// Schema-evolution rule: rows are kept for 30 days and must keep decoding
/// the whole time, so only ever *add* fields to an existing variant's
/// payload, and give every added field `#[serde(default)]`. Never rename or
/// remove a field or variant -- a changed shape is a new variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "flow", content = "event", rename_all = "snake_case")]
pub enum LogEvent {
    Auth(AuthEvent),
}

/// The GitHub sign-in token's life cycle: issued, verified (only on a cache
/// miss, never per request), rejected by GitHub, or revoked on sign-out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum AuthEvent {
    TokenIssued(TokenIssued),
    TokenExchangeFailed(TokenExchangeFailed),
    TokenVerified(TokenVerified),
    TokenRejected(TokenRejected),
    VerificationUnavailable(VerificationUnavailable),
    TokenRevoked(TokenRevoked),
    TokenRevokeFailed(TokenRevokeFailed),
}

/// `POST /auth/github/callback` exchanged a code for a new token.
/// `provider_user_id`/`login` come from a best-effort `GET /user` with the
/// new token, so they are `None` when that call failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenIssued {
    pub token_tag: String,
    #[serde(default)]
    pub provider_user_id: Option<String>,
    #[serde(default)]
    pub login: Option<String>,
}

/// GitHub's token endpoint did not hand out a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenExchangeFailed {
    pub reason: String,
}

/// A token not in the verification cache was verified against GitHub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenVerified {
    pub token_tag: String,
    pub provider_user_id: String,
}

/// GitHub's `GET /user` answered 401 for a token -- the worker then answers
/// `unauthorized` and the web app discards the token. `reason` is GitHub's
/// status and body, verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRejected {
    pub token_tag: String,
    pub reason: String,
    pub attempts: u32,
}

/// A token could not be verified for a reason that says nothing about the
/// token (GitHub 5xx or rate limit, network, D1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationUnavailable {
    pub token_tag: String,
    pub reason: String,
    pub attempts: u32,
}

/// `POST /auth/github/revoke` revoked a token on sign-out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRevoked {
    pub token_tag: String,
}

/// `POST /auth/github/revoke` failed to revoke a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRevokeFailed {
    pub token_tag: String,
    pub reason: String,
}

/// The `outcome` column, derived from the event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Ok,
    Rejected,
    Unavailable,
    Error,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::Rejected => "rejected",
            Outcome::Unavailable => "unavailable",
            Outcome::Error => "error",
        }
    }
}

/// Where a row's request came from: captured on every request-driven row
/// (sign-in, verification, revocation); `None` on rows with no request,
/// such as the scheduled prune.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestMeta {
    pub origin: Option<String>,
    pub user_agent: Option<String>,
}

/// The event-derived columns of an `app_log` row (the rest -- `at`,
/// `user_id`, `worker_version`, `origin`, `user_agent` -- are filled in when
/// the row is written).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRow {
    pub flow: String,
    pub kind: String,
    pub outcome: Outcome,
    pub subject: Option<String>,
    pub detail_json: String,
}

impl AuthEvent {
    fn outcome(&self) -> Outcome {
        match self {
            AuthEvent::TokenIssued(_)
            | AuthEvent::TokenVerified(_)
            | AuthEvent::TokenRevoked(_) => Outcome::Ok,
            AuthEvent::TokenRejected(_) => Outcome::Rejected,
            AuthEvent::VerificationUnavailable(_) => Outcome::Unavailable,
            AuthEvent::TokenExchangeFailed(_) | AuthEvent::TokenRevokeFailed(_) => Outcome::Error,
        }
    }

    fn token_tag(&self) -> Option<&str> {
        match self {
            AuthEvent::TokenIssued(event) => Some(&event.token_tag),
            AuthEvent::TokenExchangeFailed(_) => None,
            AuthEvent::TokenVerified(event) => Some(&event.token_tag),
            AuthEvent::TokenRejected(event) => Some(&event.token_tag),
            AuthEvent::VerificationUnavailable(event) => Some(&event.token_tag),
            AuthEvent::TokenRevoked(event) => Some(&event.token_tag),
            AuthEvent::TokenRevokeFailed(event) => Some(&event.token_tag),
        }
    }
}

impl LogEvent {
    pub fn outcome(&self) -> Outcome {
        match self {
            LogEvent::Auth(event) => event.outcome(),
        }
    }

    /// What the row is about, as a typed tag (`token:<token tag>`), so rows
    /// of one token can be joined across kinds.
    pub fn subject(&self) -> Option<String> {
        match self {
            LogEvent::Auth(event) => event.token_tag().map(|tag| format!("token:{tag}")),
        }
    }

    /// Splits the serialized `{"flow", "event": {"kind", "detail"}}` into
    /// columns.
    pub fn to_row(&self) -> serde_json::Result<LogRow> {
        let value = serde_json::to_value(self)?;
        let flow = string_field(&value, "flow")?;
        let event = value
            .get("event")
            .ok_or_else(|| serde_json::Error::missing_field("event"))?;
        let kind = string_field(event, "kind")?;
        let detail = event
            .get("detail")
            .ok_or_else(|| serde_json::Error::missing_field("detail"))?;
        Ok(LogRow {
            flow,
            kind,
            outcome: self.outcome(),
            subject: self.subject(),
            detail_json: detail.to_string(),
        })
    }
}

fn string_field(value: &Value, field: &'static str) -> serde_json::Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| serde_json::Error::missing_field(field))
}

impl TryFrom<&LogRow> for LogEvent {
    type Error = serde_json::Error;

    /// Reassembles a row read back from `app_log` into its event.
    fn try_from(row: &LogRow) -> serde_json::Result<LogEvent> {
        let detail: Value = serde_json::from_str(&row.detail_json)?;
        serde_json::from_value(serde_json::json!({
            "flow": row.flow,
            "event": { "kind": row.kind, "detail": detail },
        }))
    }
}
