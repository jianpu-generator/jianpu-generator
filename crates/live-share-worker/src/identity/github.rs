//! Real GitHub identity resolution: `GithubIdentityProvider`, the
//! production `IdentityProvider` for the dedicated, minimally-scoped
//! "sign in with GitHub" connection described in
//! `TODO-synced-share-rust-d1-migration.md` §0/§6 (tasks 6-7). Wired into
//! the write path via `crate::identity::resolve_verified_user_id`, which
//! adds the hashed-token cache against `oauth_sessions` and the
//! `backoff`-wrapped retry-then-fail-closed policy from §0 in front of the
//! `resolve_identity` call this module implements.

use serde::Deserialize;
use worker::{D1Database, Error};
use worker::{Fetch, Headers, Method, Request, RequestInit, Result};

use crate::db;
use crate::identity::{IdentityProvider, ResolvedIdentity};
use crate::share_id::{generate_id, USER_ID_LENGTH};

const GITHUB_PROVIDER: &str = "github";
const GITHUB_USER_ENDPOINT: &str = "https://api.github.com/user";

/// The subset of GitHub's `GET /user` response this provider needs. `id` is
/// GitHub's stable numeric user id (bound as `provider_user_id`, which is
/// `TEXT` in the schema since not every provider's id is numeric -- see §1);
/// `login` is the current username, cached into `user_identities.login` on
/// create-on-first-sight for "shared by @username" display (task 10 wires
/// it into the actual viewer UI -- this only persists it).
#[derive(Debug, Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
}

/// Resolves the Synced Share sign-in connection's token to an internal
/// `user_id` by calling GitHub's `GET /user` directly. See the module doc
/// comment for the cache/retry policy wrapped around this by
/// `crate::identity::resolve_verified_user_id`.
pub(crate) struct GithubIdentityProvider;

impl IdentityProvider for GithubIdentityProvider {
    async fn resolve_identity(
        &self,
        db: &D1Database,
        identity_token: &str,
    ) -> Result<ResolvedIdentity> {
        let github_user = fetch_github_user(identity_token).await?;
        let provider_user_id = github_user.id.to_string();

        if let Some(user_id) =
            db::get_user_id_for_identity(db, GITHUB_PROVIDER, &provider_user_id).await?
        {
            return Ok(ResolvedIdentity {
                user_id,
                provider: GITHUB_PROVIDER.to_string(),
                provider_user_id,
            });
        }

        let user_id = generate_id(USER_ID_LENGTH);
        let now = worker::Date::now().as_millis() as i64;
        db::insert_user(db, &user_id, now).await?;
        db::insert_user_identity(
            db,
            GITHUB_PROVIDER,
            &provider_user_id,
            &user_id,
            Some(&github_user.login),
            now,
        )
        .await?;
        Ok(ResolvedIdentity {
            user_id,
            provider: GITHUB_PROVIDER.to_string(),
            provider_user_id,
        })
    }
}

/// Calls `GET /user` with `identity_token` and returns just the `login`,
/// for `oauth::github_oauth_callback`'s best-effort identity-chip response
/// (task 8) -- a thin, crate-visible wrapper since `GithubUser` and
/// `fetch_github_user` itself stay private to this module.
pub(crate) async fn fetch_github_login(identity_token: &str) -> Result<String> {
    fetch_github_user(identity_token)
        .await
        .map(|user| user.login)
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
