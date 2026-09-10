//! Real GitHub identity resolution: `GithubIdentityProvider`, the
//! production `IdentityProvider` for the dedicated, minimally-scoped
//! "sign in with GitHub" connection described in
//! `TODO-synced-share-rust-d1-migration.md` §0/§6 (task 6).
//!
//! Scope note: this module is plumbing/structure only, per task 6. It is
//! **not** wired into `handlers.rs` yet -- every write still goes through
//! `identity::stub::StubIdentityProvider` until task 7 replaces it. Task 7
//! also adds the pieces this module deliberately does not: hashed-token
//! caching against `oauth_sessions`, the `backoff`-wrapped
//! retry-once-then-fail-closed policy from §0, and populating
//! `user_identities.login` for "shared by @username" display. This module
//! only proves out the direct call to GitHub's `GET /user` plus the same
//! create-on-first-sight `users`/`user_identities` path
//! `identity::stub::StubIdentityProvider` already exercises.

// Task 6 is plumbing only: nothing calls this module yet (`handlers.rs`
// still resolves every write through `identity::stub::StubIdentityProvider`
// until task 7 wires this provider in). That leaves every item below
// genuinely unreferenced from this crate's own perspective, which the
// workspace's `dead_code = "deny"` lint would otherwise reject -- allowed
// here deliberately, not as a lint-dodge for actually-unused code.
#![allow(dead_code)]

use serde::Deserialize;
use worker::{D1Database, Error};
use worker::{Fetch, Headers, Method, Request, RequestInit, Result};

use crate::db;
use crate::identity::IdentityProvider;
use crate::share_id::{generate_id, USER_ID_LENGTH};

const GITHUB_PROVIDER: &str = "github";
const GITHUB_USER_ENDPOINT: &str = "https://api.github.com/user";

/// The subset of GitHub's `GET /user` response this provider needs. `id` is
/// GitHub's stable numeric user id (bound as `provider_user_id`, which is
/// `TEXT` in the schema since not every provider's id is numeric -- see §1);
/// `login` is the current username, kept only in memory here for now (not
/// yet persisted -- see the module doc comment on `login` caching being
/// task 7's job).
#[derive(Debug, Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
}

/// Resolves the Synced Share sign-in connection's token to an internal
/// `user_id` by calling GitHub's `GET /user` directly. See the module doc
/// comment for what is and isn't in scope yet.
pub(crate) struct GithubIdentityProvider;

impl IdentityProvider for GithubIdentityProvider {
    async fn resolve_user_id(&self, db: &D1Database, identity_token: &str) -> Result<String> {
        let github_user = fetch_github_user(identity_token).await?;
        let provider_user_id = github_user.id.to_string();

        if let Some(user_id) =
            db::get_user_id_for_identity(db, GITHUB_PROVIDER, &provider_user_id).await?
        {
            return Ok(user_id);
        }

        let user_id = generate_id(USER_ID_LENGTH);
        let now = worker::Date::now().as_millis() as i64;
        db::insert_user(db, &user_id, now).await?;
        db::insert_user_identity(db, GITHUB_PROVIDER, &provider_user_id, &user_id, now).await?;
        Ok(user_id)
    }
}

/// Calls `GET https://api.github.com/user` with `identity_token` as a
/// bearer token. No retry/backoff here (that's the §0 policy task 7 wraps
/// around the *cached* verification path) -- a single failed call here is
/// simply a failed `resolve_user_id` call.
async fn fetch_github_user(identity_token: &str) -> Result<GithubUser> {
    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {identity_token}"))?;
    headers.set("Accept", "application/vnd.github+json")?;
    // GitHub's API rejects requests with no `User-Agent`.
    headers.set("User-Agent", "jianpu-generator-live-share-worker")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);

    let request = Request::new_with_init(GITHUB_USER_ENDPOINT, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() != 200 {
        let body = response.text().await.unwrap_or_default();
        return Err(Error::RustError(format!(
            "GitHub GET /user failed: status={}, body={body}",
            response.status_code()
        )));
    }

    response.json::<GithubUser>().await
}
