//! Queries against the `shares` table (Synced Share).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::protocol::ShareStatus;
use crate::share::ShareView;

const GET_SHARE_VIEW: &str = include_str!("../../queries/get_share_view.sql");
const UPSERT_SHARE: &str = include_str!("../../queries/upsert_share.sql");
const END_SHARE: &str = include_str!("../../queries/end_share.sql");
const GET_SHARE_STATUS_BY_FILE: &str = include_str!("../../queries/get_share_status_by_file.sql");
const SHARE_ID_EXISTS: &str = include_str!("../../queries/share_id_exists.sql");

/// Raw row shape from `get_share_view.sql` -- kept separate from
/// `ShareView` so `crate::share` (the pure, D1-free module) never has to
/// derive `serde::Deserialize` for D1's sake, same as `StoredFileRow`.
#[derive(Deserialize)]
struct ShareViewRow {
    ended_at: Option<i64>,
    trashed_at: Option<i64>,
    filename: String,
    content: String,
    revision: i64,
    owner_login: Option<String>,
}

impl From<ShareViewRow> for ShareView {
    fn from(row: ShareViewRow) -> Self {
        ShareView {
            ended_at: row.ended_at,
            trashed_at: row.trashed_at,
            filename: row.filename,
            content: row.content,
            revision: row.revision,
            owner_login: row.owner_login,
        }
    }
}

/// Raw row shape from `get_share_status_by_file.sql`.
#[derive(Deserialize)]
struct ShareStatusRow {
    share_id: String,
    ended_at: Option<i64>,
}

/// The anonymous viewer read -- `None` when no share has this id.
pub(crate) async fn get_share_view(db: &D1Database, share_id: &str) -> Result<Option<ShareView>> {
    let row: Option<ShareViewRow> = db
        .prepare(GET_SHARE_VIEW)
        .bind(&[JsValue::from_str(share_id)])?
        .first(None)
        .await?;
    Ok(row.map(ShareView::from))
}

/// Starts or resumes sharing `file_id`, returning the file's share id --
/// the existing one if it was ever shared before, else `new_share_id`. The
/// caller must have already checked it owns `file_id`.
pub(crate) async fn upsert_share(
    db: &D1Database,
    new_share_id: &str,
    file_id: &str,
    now: i64,
) -> Result<Option<String>> {
    db.prepare(UPSERT_SHARE)
        .bind(&[
            JsValue::from_str(new_share_id),
            JsValue::from_str(file_id),
            JsValue::from_f64(now as f64),
        ])?
        .first(Some("share_id"))
        .await
}

/// Marks `file_id`'s share ended. `false` -> 404: no share for a file this
/// caller owns.
pub(crate) async fn end_share(
    db: &D1Database,
    file_id: &str,
    owner_user_id: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(END_SHARE)
        .bind(&[
            JsValue::from_f64(now as f64),
            JsValue::from_str(file_id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    let changes = result.meta()?.and_then(|meta| meta.changes).unwrap_or(0);
    Ok(changes > 0)
}

/// The owner's view of `file_id`'s share -- `None` when this caller has
/// never shared it (or doesn't own it).
pub(crate) async fn get_share_status_by_file(
    db: &D1Database,
    file_id: &str,
    owner_user_id: &str,
) -> Result<Option<ShareStatus>> {
    let row: Option<ShareStatusRow> = db
        .prepare(GET_SHARE_STATUS_BY_FILE)
        .bind(&[JsValue::from_str(file_id), JsValue::from_str(owner_user_id)])?
        .first(None)
        .await?;
    Ok(row.map(|row| ShareStatus {
        share_id: row.share_id,
        ended: row.ended_at.is_some(),
    }))
}

pub(crate) async fn share_id_exists(db: &D1Database, share_id: &str) -> Result<bool> {
    let present: Option<i64> = db
        .prepare(SHARE_ID_EXISTS)
        .bind(&[JsValue::from_str(share_id)])?
        .first(Some("present"))
        .await?;
    Ok(present.is_some())
}
