//! Queries against the `sessions` table (see `crate::auth::session`). Every
//! one takes the token's hash; the raw token is never bound to a query.

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, D1PreparedStatement, Result};

const INSERT_SESSION: &str = include_str!("../../queries/insert_session.sql");
const GET_SESSION: &str = include_str!("../../queries/get_session.sql");
const EXTEND_SESSION: &str = include_str!("../../queries/extend_session.sql");
const DELETE_SESSION: &str = include_str!("../../queries/delete_session.sql");
const DELETE_OTHER_SESSIONS: &str = include_str!("../../queries/delete_other_sessions.sql");

/// A live session's row.
#[derive(Deserialize)]
pub(crate) struct Session {
    pub(crate) user_id: String,
    pub(crate) expires_at: i64,
}

/// The statement starting a session, for a caller that batches it (sign-up
/// creates the account and its first session in one transaction).
pub(crate) fn insert_session_statement(
    db: &D1Database,
    token_hash: &str,
    user_id: &str,
    created_at: i64,
    expires_at: i64,
) -> Result<D1PreparedStatement> {
    db.prepare(INSERT_SESSION).bind(&[
        JsValue::from_str(token_hash),
        JsValue::from_str(user_id),
        JsValue::from_f64(created_at as f64),
        JsValue::from_f64(expires_at as f64),
    ])
}

pub(crate) async fn insert_session(
    db: &D1Database,
    token_hash: &str,
    user_id: &str,
    created_at: i64,
    expires_at: i64,
) -> Result<()> {
    insert_session_statement(db, token_hash, user_id, created_at, expires_at)?
        .run()
        .await?;
    Ok(())
}

/// The session behind `token_hash`, unless there is none or it expired
/// before `now`.
pub(crate) async fn get_live_session(
    db: &D1Database,
    token_hash: &str,
    now: i64,
) -> Result<Option<Session>> {
    db.prepare(GET_SESSION)
        .bind(&[JsValue::from_str(token_hash), JsValue::from_f64(now as f64)])?
        .first(None)
        .await
}

pub(crate) async fn extend_session(
    db: &D1Database,
    token_hash: &str,
    expires_at: i64,
) -> Result<()> {
    db.prepare(EXTEND_SESSION)
        .bind(&[
            JsValue::from_str(token_hash),
            JsValue::from_f64(expires_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

pub(crate) async fn delete_session(db: &D1Database, token_hash: &str) -> Result<()> {
    db.prepare(DELETE_SESSION)
        .bind(&[JsValue::from_str(token_hash)])?
        .run()
        .await?;
    Ok(())
}

/// The statement ending every session of `user_id` but `kept_token_hash`.
pub(crate) fn delete_other_sessions_statement(
    db: &D1Database,
    user_id: &str,
    kept_token_hash: &str,
) -> Result<D1PreparedStatement> {
    db.prepare(DELETE_OTHER_SESSIONS).bind(&[
        JsValue::from_str(user_id),
        JsValue::from_str(kept_token_hash),
    ])
}
