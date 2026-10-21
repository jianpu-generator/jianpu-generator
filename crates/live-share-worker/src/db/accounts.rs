//! Queries against the `users` / `password_credentials` /
//! `sign_in_lockouts` tables: creating an account, looking up and changing
//! its password hash, and an email's failed sign-ins (see `crate::auth` and
//! `crate::handlers::auth`).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, D1PreparedStatement, Result};

use crate::auth::lockout::LockoutState;

const INSERT_USER: &str = include_str!("../../queries/insert_user.sql");
const INSERT_PASSWORD_CREDENTIALS: &str =
    include_str!("../../queries/insert_password_credentials.sql");
const GET_PASSWORD_CREDENTIALS_BY_EMAIL: &str =
    include_str!("../../queries/get_password_credentials_by_email.sql");
const GET_PASSWORD_CREDENTIALS_BY_USER: &str =
    include_str!("../../queries/get_password_credentials_by_user.sql");
const UPDATE_PASSWORD_HASH: &str = include_str!("../../queries/update_password_hash.sql");
const GET_SIGN_IN_LOCKOUT: &str = include_str!("../../queries/get_sign_in_lockout.sql");
const UPSERT_SIGN_IN_LOCKOUT: &str = include_str!("../../queries/upsert_sign_in_lockout.sql");
const DELETE_SIGN_IN_LOCKOUT: &str = include_str!("../../queries/delete_sign_in_lockout.sql");

/// An account's sign-in credentials (`password_credentials`).
#[derive(Deserialize)]
pub(crate) struct PasswordCredentials {
    pub(crate) user_id: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
}

/// A new account: its `users` row and its credentials, plus whatever else
/// must land in the same transaction (`also`, e.g. its first session).
/// Fails with D1's `UNIQUE constraint failed` when `email` already has an
/// account, writing nothing.
pub(crate) async fn insert_account(
    db: &D1Database,
    credentials: &PasswordCredentials,
    created_at: i64,
    also: Vec<D1PreparedStatement>,
) -> Result<()> {
    let created_at = JsValue::from_f64(created_at as f64);
    let statements = [
        db.prepare(INSERT_USER)
            .bind(&[JsValue::from_str(&credentials.user_id), created_at.clone()])?,
        db.prepare(INSERT_PASSWORD_CREDENTIALS).bind(&[
            JsValue::from_str(&credentials.user_id),
            JsValue::from_str(&credentials.email),
            JsValue::from_str(&credentials.password_hash),
            created_at,
        ])?,
    ]
    .into_iter()
    .chain(also)
    .collect();
    db.batch(statements).await?;
    Ok(())
}

pub(crate) async fn get_password_credentials_by_email(
    db: &D1Database,
    email: &str,
) -> Result<Option<PasswordCredentials>> {
    db.prepare(GET_PASSWORD_CREDENTIALS_BY_EMAIL)
        .bind(&[JsValue::from_str(email)])?
        .first(None)
        .await
}

pub(crate) async fn get_password_credentials_by_user(
    db: &D1Database,
    user_id: &str,
) -> Result<Option<PasswordCredentials>> {
    db.prepare(GET_PASSWORD_CREDENTIALS_BY_USER)
        .bind(&[JsValue::from_str(user_id)])?
        .first(None)
        .await
}

/// The statement storing a user's new password hash, for a caller that
/// batches it with ending their other sessions.
pub(crate) fn update_password_hash_statement(
    db: &D1Database,
    user_id: &str,
    password_hash: &str,
    updated_at: i64,
) -> Result<D1PreparedStatement> {
    db.prepare(UPDATE_PASSWORD_HASH).bind(&[
        JsValue::from_str(user_id),
        JsValue::from_str(password_hash),
        JsValue::from_f64(updated_at as f64),
    ])
}

/// Raw row shape from `get_sign_in_lockout.sql`.
#[derive(Deserialize)]
struct LockoutRow {
    failed_count: i64,
    locked_until: Option<i64>,
    lockout_level: i64,
}

pub(crate) async fn get_sign_in_lockout(
    db: &D1Database,
    email: &str,
) -> Result<Option<LockoutState>> {
    let row: Option<LockoutRow> = db
        .prepare(GET_SIGN_IN_LOCKOUT)
        .bind(&[JsValue::from_str(email)])?
        .first(None)
        .await?;
    Ok(row.map(|row| LockoutState {
        failed_count: row.failed_count,
        locked_until: row.locked_until,
        lockout_level: row.lockout_level,
    }))
}

pub(crate) async fn upsert_sign_in_lockout(
    db: &D1Database,
    email: &str,
    state: &LockoutState,
) -> Result<()> {
    db.prepare(UPSERT_SIGN_IN_LOCKOUT)
        .bind(&[
            JsValue::from_str(email),
            JsValue::from_f64(state.failed_count as f64),
            state
                .locked_until
                .map(|millis| JsValue::from_f64(millis as f64))
                .unwrap_or(JsValue::NULL),
            JsValue::from_f64(state.lockout_level as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

pub(crate) async fn delete_sign_in_lockout(db: &D1Database, email: &str) -> Result<()> {
    db.prepare(DELETE_SIGN_IN_LOCKOUT)
        .bind(&[JsValue::from_str(email)])?
        .run()
        .await?;
    Ok(())
}
