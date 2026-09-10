//! Identity resolution seam for Synced Share ownership.
//!
//! Real GitHub OAuth verification (`GithubIdentityProvider`, hashed-token
//! caching against `oauth_sessions`, retry/backoff) lands in task 6/7 of
//! `TODO-synced-share-rust-d1-migration.md` §6. This trait is the seam that
//! work will plug into; `stub` provides the only implementation for now,
//! and it is explicitly **not secure** -- see its doc comment.

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
