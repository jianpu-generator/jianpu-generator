//! Queries against the `users` / `user_identities` / `oauth_sessions`
//! tables (account identity, shared by Synced Share and the cloud storage
//! backend -- see `crate::identity`).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

const GET_USER_IDENTITY: &str = include_str!("../../queries/get_user_identity.sql");
const INSERT_USER: &str = include_str!("../../queries/insert_user.sql");
const INSERT_USER_IDENTITY: &str = include_str!("../../queries/insert_user_identity.sql");
const GET_OAUTH_SESSION: &str = include_str!("../../queries/get_oauth_session.sql");
const UPSERT_OAUTH_SESSION: &str = include_str!("../../queries/upsert_oauth_session.sql");

pub(crate) async fn get_user_id_for_identity(
    db: &D1Database,
    provider: &str,
    provider_user_id: &str,
) -> Result<Option<String>> {
    db.prepare(GET_USER_IDENTITY)
        .bind(&[
            JsValue::from_str(provider),
            JsValue::from_str(provider_user_id),
        ])?
        .first(Some("user_id"))
        .await
}

pub(crate) async fn insert_user(db: &D1Database, user_id: &str, created_at: i64) -> Result<()> {
    db.prepare(INSERT_USER)
        .bind(&[
            JsValue::from_str(user_id),
            JsValue::from_f64(created_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

/// `login` is a best-effort cached display name (nullable -- a future
/// non-GitHub provider that can't offer one binds `NULL`).
pub(crate) async fn insert_user_identity(
    db: &D1Database,
    provider: &str,
    provider_user_id: &str,
    user_id: &str,
    login: Option<&str>,
    linked_at: i64,
) -> Result<()> {
    db.prepare(INSERT_USER_IDENTITY)
        .bind(&[
            JsValue::from_str(provider),
            JsValue::from_str(provider_user_id),
            JsValue::from_str(user_id),
            login.map(JsValue::from_str).unwrap_or(JsValue::NULL),
            JsValue::from_f64(linked_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

/// Raw row shape from `get_oauth_session.sql`.
#[derive(Deserialize)]
pub(crate) struct OauthSession {
    pub provider: String,
    pub provider_user_id: String,
    pub verified_at: i64,
}

/// Hashed-token cache lookup against `oauth_sessions` (TODO §0/§6, task 7).
/// `token_hash` is a SHA-256 hex digest of the identity token -- the raw
/// token is never bound here or anywhere else against this table.
pub(crate) async fn get_oauth_session(
    db: &D1Database,
    token_hash: &str,
) -> Result<Option<OauthSession>> {
    db.prepare(GET_OAUTH_SESSION)
        .bind(&[JsValue::from_str(token_hash)])?
        .first(None)
        .await
}

/// Refreshes (or inserts) the cached verification for a token hash after a
/// successful GitHub verification.
pub(crate) async fn upsert_oauth_session(
    db: &D1Database,
    token_hash: &str,
    provider: &str,
    provider_user_id: &str,
    verified_at: i64,
) -> Result<()> {
    db.prepare(UPSERT_OAUTH_SESSION)
        .bind(&[
            JsValue::from_str(token_hash),
            JsValue::from_str(provider),
            JsValue::from_str(provider_user_id),
            JsValue::from_f64(verified_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}
