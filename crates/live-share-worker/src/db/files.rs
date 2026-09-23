//! Queries against the `files` table (cloud storage backend).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::files::{ContentWriteAttempt, StoredFile};

const LIST_FILES_BY_OWNER: &str = include_str!("../../queries/list_files_by_owner.sql");
const INSERT_FILE: &str = include_str!("../../queries/insert_file.sql");
const GET_FILE_BY_ID: &str = include_str!("../../queries/get_file_by_id.sql");
const UPDATE_FILE_CONTENT: &str = include_str!("../../queries/update_file_content.sql");
const RENAME_FILE: &str = include_str!("../../queries/rename_file.sql");
const TRASH_FILE: &str = include_str!("../../queries/trash_file.sql");
const RESTORE_FILE: &str = include_str!("../../queries/restore_file.sql");

/// Raw row shape from the `files` queries below -- no column needs
/// converting (D1/SQLite's `INTEGER` maps straight onto
/// `Option<i64>` for `trashed_at`), but a separate row type is kept anyway
/// so `crate::files` (the pure, D1-free module) never has to derive
/// `serde::Deserialize` for D1's sake.
#[derive(Deserialize)]
struct StoredFileRow {
    id: String,
    owner_user_id: String,
    name: String,
    content: String,
    revision: i64,
    trashed_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

impl From<StoredFileRow> for StoredFile {
    fn from(row: StoredFileRow) -> Self {
        StoredFile {
            id: row.id,
            owner_user_id: row.owner_user_id,
            name: row.name,
            content: row.content,
            revision: row.revision,
            trashed_at: row.trashed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// All of an owner's files, active and trashed alike -- `crate::handlers`
/// partitions the result by `trashed_at` before it reaches the client.
pub(crate) async fn list_files_by_owner(
    db: &D1Database,
    owner_user_id: &str,
) -> Result<Vec<StoredFile>> {
    let rows: Vec<StoredFileRow> = db
        .prepare(LIST_FILES_BY_OWNER)
        .bind(&[JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows.into_iter().map(StoredFile::from).collect())
}

/// Creates a new file row (new file / duplicate / import). Propagates a D1
/// error as-is, including a `idx_files_owner_name` unique-constraint
/// violation on a name-collision race -- `crate::handlers` maps that one to
/// a `409 name_taken` response via `is_unique_constraint_violation`.
pub(crate) async fn insert_file(db: &D1Database, file: &StoredFile) -> Result<()> {
    db.prepare(INSERT_FILE)
        .bind(&[
            JsValue::from_str(&file.id),
            JsValue::from_str(&file.owner_user_id),
            JsValue::from_str(&file.name),
            JsValue::from_str(&file.content),
            JsValue::from_f64(file.created_at as f64),
            JsValue::from_f64(file.updated_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

/// Scoped by `id` + `owner_user_id` -- `None` covers both "no such file"
/// and "not this caller's file", indistinguishable by design (this worker
/// never reveals whether a given id belongs to someone else).
pub(crate) async fn get_file_by_id(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
) -> Result<Option<StoredFile>> {
    let row: Option<StoredFileRow> = db
        .prepare(GET_FILE_BY_ID)
        .bind(&[JsValue::from_str(id), JsValue::from_str(owner_user_id)])?
        .first(None)
        .await?;
    Ok(row.map(StoredFile::from))
}

/// The atomic CAS content-save write. Reads the D1 result's `meta().changes`
/// to tell "applied" from "zero rows affected" (revision moved on, file was
/// trashed, or it isn't this caller's file) -- see
/// `crate::files::classify_content_write`, which turns this into the
/// caller-facing outcome once combined with a `get_file_by_id` lookup on the
/// zero-rows case.
pub(crate) async fn update_file_content(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    content: &str,
    expected_revision: i64,
    now: i64,
) -> Result<ContentWriteAttempt> {
    let result = db
        .prepare(UPDATE_FILE_CONTENT)
        .bind(&[
            JsValue::from_str(content),
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
            JsValue::from_f64(expected_revision as f64),
        ])?
        .run()
        .await?;
    rows_changed(&result)
}

/// Atomic rename, no revision gate. Returns whether a row was actually
/// affected (`false` -> 404: no such file for this caller) -- can also
/// propagate a `idx_files_owner_name` unique-constraint violation on a
/// name-collision race, same as `insert_file`.
pub(crate) async fn rename_file(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    name: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(RENAME_FILE)
        .bind(&[
            JsValue::from_str(name),
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    Ok(rows_changed(&result)? == ContentWriteAttempt::Applied)
}

/// Atomic delete (move to the bin). `false` -> 404: no such active file for
/// this caller (already trashed, wrong owner, or never existed).
pub(crate) async fn trash_file(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(TRASH_FILE)
        .bind(&[
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    Ok(rows_changed(&result)? == ContentWriteAttempt::Applied)
}

/// Atomic restore. `false` -> 404: no such trashed file for this caller.
/// Can also propagate a `idx_files_owner_name` unique-constraint violation
/// on a name-collision race, same as `insert_file`/`rename_file`.
pub(crate) async fn restore_file(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    name: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(RESTORE_FILE)
        .bind(&[
            JsValue::from_str(name),
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    Ok(rows_changed(&result)? == ContentWriteAttempt::Applied)
}

/// Reads a `D1Result`'s `meta().changes` to classify a write as
/// `Applied`/`RowsAffectedZero`. `changes` being absent from the metadata
/// (rather than `Some(0)`) is treated the same as zero -- this worker never
/// runs a write whose success is ambiguous enough to need a third state.
/// Synchronous -- `D1Result::meta()` itself isn't async.
fn rows_changed(result: &worker::D1Result) -> Result<ContentWriteAttempt> {
    let changes = result.meta()?.and_then(|meta| meta.changes).unwrap_or(0);
    Ok(if changes > 0 {
        ContentWriteAttempt::Applied
    } else {
        ContentWriteAttempt::RowsAffectedZero
    })
}
