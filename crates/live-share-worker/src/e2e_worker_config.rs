//! What `web/playwright.config.ts` and the e2e mock GitHub server need to
//! know about the worker to run it against a mocked GitHub: the `[vars]`
//! names to override, and the client id placeholder in the revocation URL.
//! Written to `web/src/generated/live-share-worker/e2eWorkerConfig.json` by
//! `tests/export_openapi.rs`, so the web side imports them rather than
//! repeating them.

use serde::Serialize;

use crate::e2e_reset::GITHUB_USER_IDS_VAR;
use crate::identity::github::GITHUB_USER_URL_VAR;
use crate::identity::SESSION_TTL_MILLIS_VAR;
use crate::oauth::{
    CLIENT_ID_BINDING, CLIENT_ID_PLACEHOLDER, GITHUB_TOKEN_REVOCATION_URL_VAR, GITHUB_TOKEN_URL_VAR,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct E2eWorkerConfig {
    /// Enables the test-only `POST /e2e/reset` route; its value is a JSON
    /// array of GitHub user ids.
    pub reset_github_user_ids_var: String,
    /// The GitHub OAuth client id the worker signs in with.
    pub github_client_id_var: String,
    pub github_user_url_var: String,
    pub github_token_url_var: String,
    /// The revocation URL's value contains `client_id_placeholder`.
    pub github_token_revocation_url_var: String,
    pub client_id_placeholder: String,
    pub session_ttl_millis_var: String,
}

pub fn e2e_worker_config() -> E2eWorkerConfig {
    E2eWorkerConfig {
        reset_github_user_ids_var: GITHUB_USER_IDS_VAR.to_string(),
        github_client_id_var: CLIENT_ID_BINDING.to_string(),
        github_user_url_var: GITHUB_USER_URL_VAR.to_string(),
        github_token_url_var: GITHUB_TOKEN_URL_VAR.to_string(),
        github_token_revocation_url_var: GITHUB_TOKEN_REVOCATION_URL_VAR.to_string(),
        client_id_placeholder: CLIENT_ID_PLACEHOLDER.to_string(),
        session_ttl_millis_var: SESSION_TTL_MILLIS_VAR.to_string(),
    }
}
