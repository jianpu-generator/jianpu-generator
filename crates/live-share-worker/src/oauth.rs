//! Worker-side leg of the account's dedicated "sign in with GitHub"
//! connection -- shared by both file storage (`/files/*`) and Synced Share
//! (`/shares/*`), not specific to either
//! (`TODO-synced-share-rust-d1-migration.md` §0/§6, task 6): the
//! authorization-code + PKCE token exchange, which needs the OAuth App's
//! client secret and so cannot run in the browser. The client secret is
//! read from a Worker secret binding (`SYNCED_SHARE_GITHUB_CLIENT_SECRET`,
//! see `CLIENT_SECRET_BINDING` below) -- set via `wrangler secret put`,
//! never committed to this repo.
//!
//! Scope note: this module only performs the token exchange itself (plus,
//! as of task 8, a best-effort `GET /user` call to hand the client a
//! `login` to display -- see `GithubOauthCallbackResponse::login`). It does
//! not verify or cache the resulting token, nor resolve it to an internal
//! `user_id` -- that is `identity::github::GithubIdentityProvider` plus the
//! hashed-token caching and write-path wiring from task 7, which every
//! write (storage or sharing) still goes through regardless of what this
//! route returns. It also does not persist anything: the exchanged access
//! token is simply returned to the caller (the browser), which owns storing
//! it and sending it as `identity_token` on writes (task 8).
//!
//! `oauth4webapi` (the library TODO §0 names for this connection) is a
//! browser/Fetch-API-oriented JS library with no Rust equivalent runnable
//! in a `wasm32-unknown-unknown` Worker; the client-side half of this
//! connection (`web/src/storage/syncedShareGithubAuth.ts`) uses it to build
//! the PKCE authorization request, but this Worker-side leg makes the same
//! plain `POST` to GitHub's token endpoint that `oauth4webapi`'s token-grant
//! functions would, using this crate's existing `worker::Fetch`-based HTTP
//! calling convention (see `identity::github::fetch_github_user` for the
//! same pattern against `GET /user`).
//!
//! Also home to `POST /auth/github/revoke` (`github_revoke` below): GitHub's
//! real `/authorize` endpoint has no "force fresh consent" parameter, so the
//! only genuine way to make a *later* sign-in re-show the consent screen is
//! revoking this app's authorization grant now, via GitHub's
//! `DELETE /applications/{client_id}/grant`. Called from logout
//! (`disconnectGithub` in `useSyncedShareOwner.ts`), not from sign-in
//! itself -- see `github_revoke`'s own doc comment. This disconnects the
//! account as a whole (both storage and sharing), since it is one shared
//! sign-in.

use base64::Engine;
use serde::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::{Error, Fetch, Headers, Method, Request, RequestInit, Response, Result};
use worker::{RouteContext, Var};

const DEFAULT_GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// Optional `[vars]` override for the token-exchange endpoint, mirroring
/// `identity::github`'s `GITHUB_USER_URL_VAR` -- unset in production, set by
/// e2e's local `wrangler dev` run to a mock GitHub server so no Playwright
/// run ever makes a real GitHub API call (task 11).
const GITHUB_TOKEN_URL_VAR: &str = "SYNCED_SHARE_GITHUB_TOKEN_URL";

/// Optional `[vars]` override for the grant-revocation endpoint, same
/// mocking purpose as `GITHUB_TOKEN_URL_VAR` above. `{client_id}` in the
/// resolved value is substituted with the real client id (see
/// `grant_url_from_env`) since GitHub's revoke endpoint embeds it in the
/// path, not the query string or body.
const GITHUB_GRANT_URL_VAR: &str = "SYNCED_SHARE_GITHUB_GRANT_URL";
const DEFAULT_GITHUB_GRANT_URL_TEMPLATE: &str =
    "https://api.github.com/applications/{client_id}/grant";

/// Resolves the grant-revocation endpoint to call, substituting `client_id`
/// into the `{client_id}` placeholder -- GitHub's real revoke endpoint
/// embeds it in the URL path itself.
fn grant_url_from_env(ctx: &RouteContext<()>, client_id: &str) -> String {
    let template = ctx
        .var(GITHUB_GRANT_URL_VAR)
        .map(|v| v.to_string())
        .unwrap_or_else(|_| DEFAULT_GITHUB_GRANT_URL_TEMPLATE.to_string());
    template.replace("{client_id}", client_id)
}

/// Worker binding names this route expects. `CLIENT_ID` is not secret --
/// GitHub OAuth client ids are public, the browser already sends the same
/// one to build the authorization URL -- but is read from a binding
/// alongside the secret since both are needed together for the token
/// exchange. `CLIENT_SECRET` MUST be set via `wrangler secret put`, never
/// committed to `wrangler.toml`.
const CLIENT_ID_BINDING: &str = "SYNCED_SHARE_GITHUB_CLIENT_ID";
const CLIENT_SECRET_BINDING: &str = "SYNCED_SHARE_GITHUB_CLIENT_SECRET";

/// Body of `POST /auth/github/callback`: what the browser sends once
/// GitHub's redirect lands back on the app with `?code=...&state=...` --
/// the PKCE `code_verifier` only the browser ever held (see
/// `syncedShareGithubAuth.ts`), plus the exact `redirect_uri` used to start
/// the authorization request (GitHub requires it to match on the token
/// exchange too).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubOauthCallbackRequest {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

/// Response of `POST /auth/github/callback` on success. `access_token` is
/// the only field the client actually needs to act as this identity
/// (resolving it to a `user_id` is a separate, later concern -- task 7's
/// `identity::resolve_verified_user_id`, which every write still goes
/// through, not this route's job). `login` is a best-effort convenience for
/// the client's signed-in-as-@username identity chip (task 8, shown by both
/// the storage and Synced Share UIs) -- fetched via one extra `GET /user`
/// call with the freshly issued token; `None` if that call fails, since a
/// missing display name shouldn't fail a successful sign-in.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubOauthCallbackResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
}

/// Raw shape of GitHub's token-endpoint JSON response. GitHub can respond
/// `200 OK` even on failure, with an `error`/`error_description` pair
/// instead of `access_token` -- both are optional here to cover both cases.
#[derive(Debug, Deserialize)]
struct GithubTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// `POST /auth/github/callback` -- exchanges an authorization `code` (plus
/// its PKCE `code_verifier`) for an access token, using this Worker's own
/// client secret. See the module doc comment for what this route does and
/// does not do.
pub(crate) async fn github_oauth_callback(
    mut req: Request,
    ctx: RouteContext<()>,
) -> Result<Response> {
    let Ok(body) = req.json::<GithubOauthCallbackRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let client_id: Var = ctx.var(CLIENT_ID_BINDING)?;
    let client_secret = ctx.secret(CLIENT_SECRET_BINDING)?;
    let token_url = ctx
        .var(GITHUB_TOKEN_URL_VAR)
        .map(|value| value.to_string())
        .unwrap_or_else(|_| DEFAULT_GITHUB_TOKEN_URL.to_string());
    let user_endpoint = crate::identity::github::user_endpoint_from_env(&ctx);

    let token_response = exchange_code_for_token(
        &token_url,
        &client_id.to_string(),
        &client_secret.to_string(),
        &body,
    )
    .await?;

    match token_response.access_token {
        Some(access_token) => {
            // Best-effort only: a failed `GET /user` here must not fail an
            // otherwise-successful sign-in, so `login` degrades to `None`
            // rather than propagating the error.
            let login = crate::identity::github::fetch_github_login(&user_endpoint, &access_token)
                .await
                .ok();
            Response::from_json(&GithubOauthCallbackResponse {
                access_token,
                login,
            })
        }
        None => {
            let reason = token_response
                .error_description
                .or(token_response.error)
                .unwrap_or_else(|| "GitHub did not return an access token".to_string());
            Response::error(format!("GitHub token exchange failed: {reason}"), 502)
        }
    }
}

async fn exchange_code_for_token(
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    body: &GithubOauthCallbackRequest,
) -> Result<GithubTokenResponse> {
    let payload = serde_json::json!({
        "client_id": client_id,
        "client_secret": client_secret,
        "code": body.code,
        "code_verifier": body.code_verifier,
        "redirect_uri": body.redirect_uri,
    });

    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Accept", "application/json")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(&payload.to_string())));

    let request = Request::new_with_init(token_url, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() != 200 {
        let text = response.text().await.unwrap_or_default();
        return Err(Error::RustError(format!(
            "GitHub token exchange request failed: status={}, body={text}",
            response.status_code()
        )));
    }

    response.json::<GithubTokenResponse>().await
}

/// Body of `POST /auth/github/revoke`: the token being disconnected, so its
/// underlying GitHub grant can be revoked server-side. Sent from
/// `disconnectGithub` (`useSyncedShareOwner.ts`) at logout time -- see the
/// module doc comment for why that's the only place a token to revoke ever
/// exists.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubRevokeRequest {
    pub identity_token: String,
}

/// `POST /auth/github/revoke` -- GitHub OAuth Apps have no "force consent"
/// request parameter (confirmed against GitHub's docs: `/login/oauth/authorize`
/// only supports `client_id`, `redirect_uri`, `login`, `scope`, `state`,
/// `allow_signup`, plus PKCE fields). The only real mechanism for forcing a
/// fresh consent screen on a *later* sign-in is deleting this app's
/// authorization grant now, via `DELETE /applications/{client_id}/grant` --
/// so this route is called from logout (`disconnectGithub`), not from
/// sign-in itself, where no token to revoke exists yet.
///
/// This is a genuine (non-swallowed) error response on failure; the client
/// treats the call as best-effort (never awaited, never blocks the local
/// logout), but this route's own contract stays honest/debuggable, matching
/// `github_oauth_callback`'s error shape.
pub(crate) async fn github_revoke(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Ok(body) = req.json::<GithubRevokeRequest>().await else {
        return Response::error("Bad Request", 400);
    };

    let client_id: Var = ctx.var(CLIENT_ID_BINDING)?;
    let client_secret = ctx.secret(CLIENT_SECRET_BINDING)?;
    let grant_url = grant_url_from_env(&ctx, &client_id.to_string());

    match revoke_grant(
        &grant_url,
        &client_id.to_string(),
        &client_secret.to_string(),
        &body.identity_token,
    )
    .await
    {
        Ok(()) => Response::empty(),
        Err(error) => Response::error(format!("GitHub grant revocation failed: {error}"), 502),
    }
}

/// Calls `DELETE {grant_url}` with HTTP Basic auth (`client_id:client_secret`)
/// and a JSON body of `{"access_token": identity_token}`, matching GitHub's
/// real revoke-grant API. GitHub responds `204 No Content` on success; `200`
/// is also accepted defensively. Same `Headers`/`RequestInit`/`Fetch::Request`
/// pattern as `identity::github::fetch_github_user`.
async fn revoke_grant(
    grant_url: &str,
    client_id: &str,
    client_secret: &str,
    identity_token: &str,
) -> Result<()> {
    let credentials =
        base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"));
    let payload = serde_json::json!({ "access_token": identity_token });

    let headers = Headers::new();
    headers.set("Authorization", &format!("Basic {credentials}"))?;
    headers.set("Content-Type", "application/json")?;
    headers.set("Accept", "application/vnd.github+json")?;
    headers.set("User-Agent", "jianpu-generator-live-share-worker")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Delete)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(&payload.to_string())));

    let request = Request::new_with_init(grant_url, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    match response.status_code() {
        204 | 200 => Ok(()),
        status => {
            let text = response.text().await.unwrap_or_default();
            Err(Error::RustError(format!(
                "GitHub grant revocation request failed: status={status}, body={text}"
            )))
        }
    }
}
