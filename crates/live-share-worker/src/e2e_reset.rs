//! The test-only `POST /e2e/reset` route's configuration (the route itself
//! is `crate::handlers::e2e_reset`). Local e2e runs reuse one Miniflare D1
//! state dir across invocations, so rows the synthetic e2e GitHub accounts
//! create pile up between runs -- e.g. every run reserves one more
//! "source N" default name -- and this route wipes those accounts' files
//! and shares before each run (`web/e2e/global-setup.ts`).
//!
//! Gated on `GITHUB_USER_IDS_VAR`: the route is only registered into the
//! router when that var is set, so without it the worker answers
//! `/e2e/reset` exactly as it does any unknown path. The real
//! `wrangler.toml` never sets it; only e2e's local `wrangler dev` passes it
//! (via `--var`, see `web/playwright.config.ts`). Its value is also the
//! allowlist: a JSON array of the GitHub user ids whose rows the route may
//! delete, so the route can never touch any other account's rows.
//!
//! D1-free, so `tests/e2e_reset.rs` can test the parsing directly.

use serde::Serialize;

/// `[vars]` name that both enables the route and lists the GitHub user ids
/// it resets. Published to the web through `e2e_reset_config` (see there),
/// never copied.
pub const GITHUB_USER_IDS_VAR: &str = "SYNCED_SHARE_E2E_RESET_GITHUB_USER_IDS";

/// What `web/playwright.config.ts` needs to enable the route, written to
/// `web/src/generated/live-share-worker/e2eResetConfig.json` by
/// `tests/export_openapi.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct E2eResetConfig {
    /// The `--var` name; its value is a JSON array of GitHub user ids.
    pub github_user_ids_var: String,
}

pub fn e2e_reset_config() -> E2eResetConfig {
    E2eResetConfig {
        github_user_ids_var: GITHUB_USER_IDS_VAR.to_string(),
    }
}

/// Parses `GITHUB_USER_IDS_VAR`'s value (a JSON array of GitHub's numeric
/// user ids) into `user_identities.provider_user_id` values -- stored as
/// text, the same `to_string()` form `crate::identity::github` writes.
pub fn parse_github_user_ids(value: &str) -> Result<Vec<String>, serde_json::Error> {
    let ids: Vec<i64> = serde_json::from_str(value)?;
    Ok(ids.iter().map(i64::to_string).collect())
}
