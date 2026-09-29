//! Test-only hard deletes behind `POST /e2e/reset` -- see
//! `crate::e2e_reset`.

use worker::wasm_bindgen::JsValue;
use worker::{D1Database, D1Result, Result};

const DELETE_SHARES_OF_PROVIDER_USERS: &str =
    include_str!("../../queries/delete_shares_of_provider_users.sql");
const DELETE_FILES_OF_PROVIDER_USERS: &str =
    include_str!("../../queries/delete_files_of_provider_users.sql");

/// How many rows `delete_rows_of_provider_users` removed from each table.
pub(crate) struct DeletedRows {
    pub(crate) shares: usize,
    pub(crate) files: usize,
}

/// Deletes every file (and those files' shares) owned by the users behind
/// `provider`'s identities with one of `provider_user_ids`. One D1 batch,
/// so it's a single transaction: shares go first, since `shares.file_id`
/// references `files(id)`.
pub(crate) async fn delete_rows_of_provider_users(
    db: &D1Database,
    provider: &str,
    provider_user_ids: &[String],
) -> Result<DeletedRows> {
    let ids_json = serde_json::to_string(provider_user_ids)
        .map_err(|error| worker::Error::RustError(error.to_string()))?;
    let params = [JsValue::from_str(provider), JsValue::from_str(&ids_json)];
    let results = db
        .batch(vec![
            db.prepare(DELETE_SHARES_OF_PROVIDER_USERS).bind(&params)?,
            db.prepare(DELETE_FILES_OF_PROVIDER_USERS).bind(&params)?,
        ])
        .await?;
    let [shares, files] = results.as_slice() else {
        return Err(worker::Error::RustError(format!(
            "expected 2 batch results, got {}",
            results.len()
        )));
    };
    Ok(DeletedRows {
        shares: changes(shares)?,
        files: changes(files)?,
    })
}

fn changes(result: &D1Result) -> Result<usize> {
    Ok(result.meta()?.and_then(|meta| meta.changes).unwrap_or(0))
}
