//! Account identity resolution shared by both file storage (`/files/*`) and
//! the Synced Share owner routes (`/files/{id}/share*`), plus the hashed-token cache + backoff-wrapped
//! verification wiring that sits in front of it
//! (`TODO-synced-share-rust-d1-migration.md` §0/§6, task 7).
//!
//! `resolve_verified_user_id` is what every authenticated route in
//! `handlers/` calls, whether it's a Synced Share owner route (start, stop,
//! status) or a `/files/*` storage route: it never trusts a client-asserted identity
//! directly, always routing through an `IdentityProvider` --
//! `github::GithubIdentityProvider` in production -- fronted by a
//! hashed-token cache against `oauth_sessions` so a fresh verification
//! doesn't re-hit GitHub on every request.

pub(crate) mod github;

use worker::{D1Database, Delay, Result, RouteContext};

use sha2::{Digest, Sha256};

use crate::db;
use crate::verification::{
    retry_with_backoff, session_is_fresh, VerificationFailure, MAX_RETRIES, MAX_TOTAL_WAIT,
    SESSION_TTL_MILLIS,
};

/// What an `IdentityProvider` resolves a token to: the internal `user_id`
/// (create-on-first-sight `users`/`user_identities` row, per §6) plus the
/// `(provider, provider_user_id)` pair that gets cached in `oauth_sessions`
/// so a later request with the same token doesn't need to re-verify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedIdentity {
    pub user_id: String,
    pub provider: String,
    pub provider_user_id: String,
}

/// Resolves whatever identity a caller presents to an internal `users.id`,
/// doing create-on-first-sight `users`/`user_identities` rows. Never trust a
/// client-asserted identity directly -- that's exactly what this trait
/// exists to avoid; every write goes through `resolve_verified_user_id`,
/// which calls this only on a cache miss/stale entry.
pub(crate) trait IdentityProvider {
    async fn resolve_identity(
        &self,
        db: &D1Database,
        identity_token: &str,
    ) -> std::result::Result<ResolvedIdentity, IdentityError>;
}

/// Why an `IdentityProvider` could not resolve a token. The split matters to
/// the client: only `TokenRejected` means the stored token is dead and the
/// user must sign in again; `Unavailable` (GitHub 5xx/rate limit, network,
/// D1) is transient, so the token must be kept and the request retried.
#[derive(Debug)]
pub(crate) enum IdentityError {
    /// The provider itself said the token is invalid or revoked.
    TokenRejected(String),
    /// Verification could not be completed for a reason unrelated to the
    /// token's validity.
    Unavailable(worker::Error),
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityError::TokenRejected(reason) => formatter.write_str(reason),
            IdentityError::Unavailable(error) => error.fmt(formatter),
        }
    }
}

impl From<worker::Error> for IdentityError {
    fn from(error: worker::Error) -> Self {
        IdentityError::Unavailable(error)
    }
}

/// Optional `[vars]` override for how long a cached `oauth_sessions`
/// verification stays fresh, in milliseconds -- unset in production
/// (`SESSION_TTL_MILLIS`), set to `0` by e2e's local `cf dev` run so
/// every request re-verifies against the mock GitHub server. Without it, a
/// token revoked on GitHub's side keeps passing on its cached verification
/// for up to an hour, so e2e could never observe a revocation.
pub(crate) const SESSION_TTL_MILLIS_VAR: &str = "SYNCED_SHARE_SESSION_TTL_MILLIS";

/// Resolves the `oauth_sessions` freshness TTL: `SESSION_TTL_MILLIS_VAR` if
/// set to a valid integer, otherwise `SESSION_TTL_MILLIS`.
pub(crate) fn session_ttl_millis_from_env(ctx: &RouteContext<()>) -> i64 {
    ctx.var(SESSION_TTL_MILLIS_VAR)
        .ok()
        .and_then(|value| value.to_string().parse().ok())
        .unwrap_or(SESSION_TTL_MILLIS)
}

/// Resolves `identity_token` to an internal `user_id`, via the hashed-token
/// cache + backoff-wrapped verification policy from TODO §0/§6:
///
/// 1. Hash the token (SHA-256; the raw token is never persisted) and look
///    up `oauth_sessions` for a fresh (within `session_ttl_millis`) cached
///    verification. On a hit, resolve straight to `user_id` via the cached
///    `(provider, provider_user_id)` -- no call to `identity_provider`.
/// 2. On a cache miss or a stale entry, call `identity_provider`, retrying
///    per TODO §0's backoff policy (`MAX_RETRIES`, capped at
///    `MAX_TOTAL_WAIT`) before failing closed. On success, refresh/insert
///    the `oauth_sessions` row and return the resolved `user_id`.
///
/// A rejected token (after retries) is reported as the inner `Err`, never as
/// a fallback identity -- callers (`handlers.rs`) must reject the write. A
/// failure that says nothing about the token (GitHub down, D1 error) is the
/// outer `Err` instead, so it surfaces as a retryable server error rather
/// than an `unauthorized` that makes the client discard a valid token.
pub(crate) async fn resolve_verified_user_id(
    db: &D1Database,
    identity_provider: &impl IdentityProvider,
    identity_token: &str,
    session_ttl_millis: i64,
) -> Result<Result<String, VerificationFailure>> {
    let token_hash = hash_token(identity_token);
    let now = worker::Date::now().as_millis() as i64;

    if let Some(session) = db::get_oauth_session(db, &token_hash).await? {
        if session_is_fresh(session.verified_at, now, session_ttl_millis) {
            if let Some(user_id) =
                db::get_user_id_for_identity(db, &session.provider, &session.provider_user_id)
                    .await?
            {
                return Ok(Ok(user_id));
            }
            // The cache says "verified" but the identity row it points at is
            // gone -- fall through and re-verify rather than trusting a
            // dangling cache entry.
        }
    }

    let retry_result = retry_with_backoff(
        MAX_RETRIES,
        MAX_TOTAL_WAIT,
        || identity_provider.resolve_identity(db, identity_token),
        Delay::from,
    )
    .await;

    let resolved = match retry_result {
        Ok(resolved) => resolved,
        Err(failure) => match failure.last_error {
            IdentityError::TokenRejected(reason) => {
                return Ok(Err(VerificationFailure {
                    reason,
                    failed_at: now,
                    attempts: failure.attempts,
                }))
            }
            IdentityError::Unavailable(error) => return Err(error),
        },
    };

    db::upsert_oauth_session(
        db,
        &token_hash,
        &resolved.provider,
        &resolved.provider_user_id,
        now,
    )
    .await?;

    Ok(Ok(resolved.user_id))
}

/// Hashes an identity token for `oauth_sessions.token_hash` -- the raw
/// token itself must never be persisted (TODO §0).
fn hash_token(identity_token: &str) -> String {
    let digest = Sha256::digest(identity_token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
