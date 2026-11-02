//! Queries against the `file_versions` table (a cloud file's version
//! history, see `crate::file_history`).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::file_history::VersionRow;

const INSERT_FILE_VERSION: &str = include_str!("../../queries/insert_file_version.sql");
const ADVANCE_FILE_VERSION_LINK: &str = include_str!("../../queries/advance_file_version_link.sql");
const LIST_FILE_VERSIONS: &str = include_str!("../../queries/list_file_versions.sql");
const PRUNE_FILE_VERSIONS: &str = include_str!("../../queries/prune_file_versions.sql");

#[derive(Deserialize)]
struct VersionRowRaw {
    revision: i64,
    newer_revision: i64,
    reverse_patch: String,
    saved_at: i64,
}

/// Every stored version of the caller's file `file_id`, newest first.
pub(crate) async fn list_file_versions(
    db: &D1Database,
    file_id: &str,
    owner_user_id: &str,
) -> Result<Vec<VersionRow>> {
    let rows: Vec<VersionRowRaw> = db
        .prepare(LIST_FILE_VERSIONS)
        .bind(&[JsValue::from_str(file_id), JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| VersionRow {
            revision: row.revision,
            newer_revision: row.newer_revision,
            reverse_patch: row.reverse_patch,
            saved_at: row.saved_at,
        })
        .collect())
}

/// Stores `version` of `file_id` and prunes the file's history down to
/// `keep` rows, in one D1 batch.
pub(crate) async fn insert_file_version(
    db: &D1Database,
    file_id: &str,
    version: &VersionRow,
    keep: i64,
) -> Result<()> {
    let insert = db.prepare(INSERT_FILE_VERSION).bind(&[
        JsValue::from_str(file_id),
        JsValue::from_f64(version.revision as f64),
        JsValue::from_f64(version.newer_revision as f64),
        JsValue::from_str(&version.reverse_patch),
        JsValue::from_f64(version.saved_at as f64),
    ])?;
    let prune = db
        .prepare(PRUNE_FILE_VERSIONS)
        .bind(&[JsValue::from_str(file_id), JsValue::from_f64(keep as f64)])?;
    db.batch(vec![insert, prune]).await?;
    Ok(())
}

/// Moves the link of the version that pointed at `old_head` to `new_head`
/// (a save that left the text unchanged).
pub(crate) async fn advance_file_version_link(
    db: &D1Database,
    file_id: &str,
    old_head: i64,
    new_head: i64,
) -> Result<()> {
    db.prepare(ADVANCE_FILE_VERSION_LINK)
        .bind(&[
            JsValue::from_str(file_id),
            JsValue::from_f64(old_head as f64),
            JsValue::from_f64(new_head as f64),
        ])?
        .run()
        .await?;
    Ok(())
}
