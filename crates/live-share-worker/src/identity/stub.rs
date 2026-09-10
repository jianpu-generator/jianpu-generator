//! **NOT SECURE.** Placeholder `IdentityProvider` for task 4 only --
//! anyone can claim any identity by sending any string as `identity_token`.
//!
//! Real GitHub OAuth verification (`GithubIdentityProvider`) is task 6/7 of
//! `TODO-synced-share-rust-d1-migration.md` §6, not this one. Until then,
//! this stub treats the incoming string directly as a stable pseudo
//! `provider_user_id` under a `"stub"` provider: re-sending the same string
//! round-trips to the same `user_id`. Its only purpose is to exercise the
//! real create-on-first-sight `users`/`user_identities` path (the same
//! path the real provider will use) end to end, without any actual
//! verification behind it. Do not try to make this "safe" -- replacing it
//! with real verification is explicitly out of scope for task 4.

use worker::{D1Database, Result};

use crate::db;
use crate::identity::IdentityProvider;
use crate::share_id::{generate_id, USER_ID_LENGTH};

const STUB_PROVIDER: &str = "stub";

/// **NOT SECURE.** See module doc comment.
pub(crate) struct StubIdentityProvider;

impl IdentityProvider for StubIdentityProvider {
    async fn resolve_user_id(&self, db: &D1Database, identity_token: &str) -> Result<String> {
        if let Some(user_id) =
            db::get_user_id_for_identity(db, STUB_PROVIDER, identity_token).await?
        {
            return Ok(user_id);
        }

        let user_id = generate_id(USER_ID_LENGTH);
        let now = worker::Date::now().as_millis() as i64;
        db::insert_user(db, &user_id, now).await?;
        db::insert_user_identity(db, STUB_PROVIDER, identity_token, &user_id, now).await?;
        Ok(user_id)
    }
}
