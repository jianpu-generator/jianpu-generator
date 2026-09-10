//! Raw D1 queries against the `docs` / `users` / `user_identities` /
//! `oauth_sessions` tables.
//! Each query lives in its own file under `queries/`, loaded here via
//! `include_str!` for the actual `D1Database::prepare()` calls -- this
//! module is the only place those files are read.
//!
//! Compile-time query checking via `sqlx::query_file!` (as originally
//! sketched in `TODO-synced-share-rust-d1-migration.md` §2) isn't possible
//! in this crate's own wasm-targeted compile -- see the long comment on the
//! `[build-dependencies] sqlx` entry in `Cargo.toml`. The pragmatic
//! substitute used instead: `tests/query_syntax.rs`, a native-only test
//! that loads these same query files (unmodified, via the identical
//! `include_str!` paths) and runs them through `sqlx`'s statement
//! preparation (not execution) against the shadow SQLite database built by
//! `build.rs`, to catch a broken/typo'd query before it would otherwise
//! only surface at real D1 runtime. This is weaker than `query_file!`'s
//! compile-time type-checking -- it doesn't check bound-parameter types or
//! result-column names against Rust types, and any D1-vs-SQLite dialect
//! drift is still uncaught -- but it's an honest, reasonable substitute
//! given the constraint above, not a silent downgrade.

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::doc::StoredDoc;

const GET_DOC_BY_SHARE_ID: &str = include_str!("../queries/get_doc_by_share_id.sql");
const INSERT_DOC: &str = include_str!("../queries/insert_doc.sql");
const UPDATE_DOC: &str = include_str!("../queries/update_doc.sql");
const SHARE_ID_EXISTS: &str = include_str!("../queries/share_id_exists.sql");
const GET_USER_IDENTITY: &str = include_str!("../queries/get_user_identity.sql");
const INSERT_USER: &str = include_str!("../queries/insert_user.sql");
const INSERT_USER_IDENTITY: &str = include_str!("../queries/insert_user_identity.sql");
const GET_OAUTH_SESSION: &str = include_str!("../queries/get_oauth_session.sql");
const UPSERT_OAUTH_SESSION: &str = include_str!("../queries/upsert_oauth_session.sql");

/// Raw row shape from `get_doc_by_share_id.sql` -- SQLite/D1 has no native
/// boolean column type, so `ended` comes back as an integer and is
/// converted in `From<StoredDocRow> for StoredDoc` below.
#[derive(Deserialize)]
struct StoredDocRow {
    share_id: String,
    owner_user_id: String,
    filename: String,
    content: String,
    revision: i64,
    ended: i64,
    created_at: i64,
    updated_at: i64,
}

impl From<StoredDocRow> for StoredDoc {
    fn from(row: StoredDocRow) -> Self {
        StoredDoc {
            share_id: row.share_id,
            owner_user_id: row.owner_user_id,
            filename: row.filename,
            content: row.content,
            revision: row.revision,
            ended: row.ended != 0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub(crate) async fn get_doc_by_share_id(
    db: &D1Database,
    share_id: &str,
) -> Result<Option<StoredDoc>> {
    let row: Option<StoredDocRow> = db
        .prepare(GET_DOC_BY_SHARE_ID)
        .bind(&[JsValue::from_str(share_id)])?
        .first(None)
        .await?;
    Ok(row.map(StoredDoc::from))
}

pub(crate) async fn insert_doc(db: &D1Database, doc: &StoredDoc) -> Result<()> {
    db.prepare(INSERT_DOC)
        .bind(&[
            JsValue::from_str(&doc.share_id),
            JsValue::from_str(&doc.owner_user_id),
            JsValue::from_str(&doc.filename),
            JsValue::from_str(&doc.content),
            JsValue::from_f64(doc.revision as f64),
            JsValue::from_f64(if doc.ended { 1.0 } else { 0.0 }),
            JsValue::from_f64(doc.created_at as f64),
            JsValue::from_f64(doc.updated_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

pub(crate) async fn update_doc(db: &D1Database, doc: &StoredDoc) -> Result<()> {
    db.prepare(UPDATE_DOC)
        .bind(&[
            JsValue::from_str(&doc.share_id),
            JsValue::from_str(&doc.filename),
            JsValue::from_str(&doc.content),
            JsValue::from_f64(doc.revision as f64),
            JsValue::from_f64(if doc.ended { 1.0 } else { 0.0 }),
            JsValue::from_f64(doc.updated_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

pub(crate) async fn share_id_exists(db: &D1Database, share_id: &str) -> Result<bool> {
    let present: Option<i64> = db
        .prepare(SHARE_ID_EXISTS)
        .bind(&[JsValue::from_str(share_id)])?
        .first(Some("present"))
        .await?;
    Ok(present.is_some())
}

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
