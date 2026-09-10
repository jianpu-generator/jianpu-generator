//! Identity resolution seam for Synced Share ownership.
//!
//! `github::GithubIdentityProvider` (task 6) resolves a real GitHub identity
//! via `GET /user`, but is not wired into any write path yet -- hashed-token
//! caching against `oauth_sessions` and the retry/backoff policy that wraps
//! it are task 7. Every write still goes through `stub::StubIdentityProvider`
//! until then, which is explicitly **not secure** -- see its doc comment.

pub(crate) mod github;
pub(crate) mod stub;

use worker::{D1Database, Result};

/// Resolves whatever identity a caller presents to an internal `users.id`,
/// doing create-on-first-sight `users`/`user_identities` rows the same way
/// the real GitHub-backed provider will (TODO §6: "look up (or
/// create-on-first-sight) the matching `user_identities` row to get
/// `user_id`"). Never trust a client-asserted identity directly -- that's
/// exactly what this trait exists to avoid; every write goes through it.
pub(crate) trait IdentityProvider {
    async fn resolve_user_id(&self, db: &D1Database, identity_token: &str) -> Result<String>;
}
