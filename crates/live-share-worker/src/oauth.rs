//! Worker-side leg of the dedicated Synced Share "sign in with GitHub"
//! connection (`TODO-synced-share-rust-d1-migration.md` §0/§6, task 6): the
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
//! write still goes through regardless of what this route returns. It also
//! does not persist anything: the exchanged access token is simply returned
//! to the caller (the browser), which owns storing it and sending it as
//! `identity_token` on writes (task 8).
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
/// the client's "Synced as @username" identity chip (task 8) -- fetched via
/// one extra `GET /user` call with the freshly issued token; `None` if that
/// call fails, since a missing display name shouldn't fail a successful
/// sign-in.
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
