//! Identity resolution seam for Synced Share ownership, and the
//! hashed-token cache + backoff-wrapped verification wiring that sits in
//! front of it (`TODO-synced-share-rust-d1-migration.md` §0/§6, task 7).
//!
//! `resolve_verified_user_id` is what every write (and the create-share
//! endpoint) calls in `handlers.rs`: it never trusts a client-asserted
//! identity directly, always routing through an `IdentityProvider` --
//! `github::GithubIdentityProvider` in production -- fronted by a
//! hashed-token cache against `oauth_sessions` so a fresh verification
//! doesn't re-hit GitHub on every request.

pub(crate) mod github;

use worker::{D1Database, Delay, Result};

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
    ) -> Result<ResolvedIdentity>;
}

/// Resolves `identity_token` to an internal `user_id`, via the hashed-token
/// cache + backoff-wrapped verification policy from TODO §0/§6:
///
/// 1. Hash the token (SHA-256; the raw token is never persisted) and look
///    up `oauth_sessions` for a fresh (within `SESSION_TTL_MILLIS`) cached
///    verification. On a hit, resolve straight to `user_id` via the cached
///    `(provider, provider_user_id)` -- no call to `identity_provider`.
/// 2. On a cache miss or a stale entry, call `identity_provider`, retrying
///    per TODO §0's backoff policy (`MAX_RETRIES`, capped at
///    `MAX_TOTAL_WAIT`) before failing closed. On success, refresh/insert
///    the `oauth_sessions` row and return the resolved `user_id`.
///
/// A verification failure (after retries) is reported as `Err`, never as a
/// fallback identity -- callers (`handlers.rs`) must reject the write.
pub(crate) async fn resolve_verified_user_id(
    db: &D1Database,
    identity_provider: &impl IdentityProvider,
    identity_token: &str,
) -> Result<Result<String, VerificationFailure>> {
    let token_hash = hash_token(identity_token);
    let now = worker::Date::now().as_millis() as i64;

    if let Some(session) = db::get_oauth_session(db, &token_hash).await? {
        if session_is_fresh(session.verified_at, now, SESSION_TTL_MILLIS) {
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
        Err(failure) => {
            return Ok(Err(VerificationFailure {
                reason: failure.last_error.to_string(),
                failed_at: now,
                attempts: failure.attempts,
            }))
        }
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
