//! Queries against the `docs` table (Synced Share).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::doc::StoredDoc;

const GET_DOC_BY_SHARE_ID: &str = include_str!("../../queries/get_doc_by_share_id.sql");
const GET_DOC_BY_OWNER_AND_EXTERNAL_FILE: &str =
    include_str!("../../queries/get_doc_by_owner_and_external_file.sql");
const INSERT_DOC: &str = include_str!("../../queries/insert_doc.sql");
const UPDATE_DOC: &str = include_str!("../../queries/update_doc.sql");
const SHARE_ID_EXISTS: &str = include_str!("../../queries/share_id_exists.sql");

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
    external_file_id: Option<String>,
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
            external_file_id: row.external_file_id,
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

/// Idempotent-create lookup: an existing `share_id` for this owner +
/// external file, if any -- see `crate::share_creation::resolve_share_id`.
pub(crate) async fn get_doc_by_owner_and_external_file(
    db: &D1Database,
    owner_user_id: &str,
    external_file_id: &str,
) -> Result<Option<String>> {
    db.prepare(GET_DOC_BY_OWNER_AND_EXTERNAL_FILE)
        .bind(&[
            JsValue::from_str(owner_user_id),
            JsValue::from_str(external_file_id),
        ])?
        .first(Some("share_id"))
        .await
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
            doc.external_file_id
                .as_deref()
                .map(JsValue::from_str)
                .unwrap_or(JsValue::NULL),
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
