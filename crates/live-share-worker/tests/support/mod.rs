//! Shared setup for the native SQLite tests of folder shares: the
//! migrations, the query files and small row helpers.
#![allow(dead_code)]

//! The folder-share SQL run for real against SQLite: the migration over
//! existing rows, start/stop/reuse, status and overview, the viewer's file
//! read, the prefix rewrite next to a batch rename, and ownership.
#![allow(clippy::disallowed_macros)]
#![cfg(not(target_arch = "wasm32"))]

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Executor, Row, SqliteConnection};

pub(crate) const INIT: &str =
    include_str!("../../../../live-share-worker/migrations/0001_init.sql");
pub(crate) const DOCS_EXTERNAL: &str =
    include_str!("../../../../live-share-worker/migrations/0002_docs_external_file_id.sql");
pub(crate) const FILES: &str =
    include_str!("../../../../live-share-worker/migrations/0003_files.sql");
pub(crate) const SHARES_POINT_AT_FILES: &str =
    include_str!("../../../../live-share-worker/migrations/0004_shares_reference_files.sql");
pub(crate) const FOLDER_SHARES: &str =
    include_str!("../../../../live-share-worker/migrations/0005_folder_shares.sql");

pub(crate) const UPSERT_SHARE: &str = include_str!("../../queries/upsert_share.sql");
pub(crate) const END_SHARE: &str = include_str!("../../queries/end_share.sql");
pub(crate) const GET_SHARE_STATUS_BY_FILE: &str =
    include_str!("../../queries/get_share_status_by_file.sql");
pub(crate) const GET_SHARE_VIEW: &str = include_str!("../../queries/get_share_view.sql");
pub(crate) const UPSERT_FOLDER_SHARE: &str = include_str!("../../queries/upsert_folder_share.sql");
pub(crate) const END_FOLDER_SHARE: &str = include_str!("../../queries/end_folder_share.sql");
pub(crate) const GET_FOLDER_SHARE_STATUS: &str =
    include_str!("../../queries/get_folder_share_status.sql");
pub(crate) const GET_FOLDER_SHARE_HEADER: &str =
    include_str!("../../queries/get_folder_share_header.sql");
pub(crate) const GET_FOLDER_FILES_STATS: &str =
    include_str!("../../queries/get_folder_files_stats.sql");
pub(crate) const LIST_FOLDER_FILES: &str = include_str!("../../queries/list_folder_files.sql");
pub(crate) const LIST_LIVE_FILE_SHARES: &str =
    include_str!("../../queries/list_live_file_shares_by_owner.sql");
pub(crate) const LIST_LIVE_FOLDER_SHARES: &str =
    include_str!("../../queries/list_live_folder_shares_by_owner.sql");
pub(crate) const LIST_FOLDER_SHARE_PREFIXES: &str =
    include_str!("../../queries/list_folder_share_prefixes.sql");
pub(crate) const LIST_ACTIVE_FILES_UNDER_PREFIXES: &str =
    include_str!("../../queries/list_active_files_under_prefixes.sql");
pub(crate) const REWRITE_FOLDER_SHARE_PREFIX: &str =
    include_str!("../../queries/rewrite_folder_share_prefix.sql");
pub(crate) const RENAME_FILES_BATCH: &str = include_str!("../../queries/rename_files_batch.sql");
pub(crate) const DELETE_SHARES_OF_USERS: &str =
    include_str!("../../queries/delete_shares_of_users.sql");

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MigratedShare {
    pub(crate) share_id: String,
    pub(crate) owner_user_id: String,
    pub(crate) file_id: String,
    pub(crate) folder_prefix: Option<String>,
    pub(crate) ended_at: Option<i64>,
    pub(crate) created_at: i64,
}

pub(crate) async fn connect() -> Result<SqliteConnection, sqlx::Error> {
    // Foreign keys stay on (sqlx's default), like D1.
    SqliteConnectOptions::new()
        .filename(":memory:")
        .connect()
        .await
}

pub(crate) async fn migrate_up_to_file_shares(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    for migration in [INIT, DOCS_EXTERNAL, FILES] {
        connection.execute(migration).await?;
    }
    Ok(())
}

pub(crate) async fn add_user(
    connection: &mut SqliteConnection,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (id, created_at) VALUES (?1, 0)")
        .bind(user_id)
        .execute(connection)
        .await?;
    Ok(())
}

pub(crate) async fn add_file(
    connection: &mut SqliteConnection,
    id: &str,
    owner: &str,
    name: &str,
    content: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO files (id, owner_user_id, name, content, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, 0, 0)",
    )
    .bind(id)
    .bind(owner)
    .bind(name)
    .bind(content)
    .execute(connection)
    .await?;
    Ok(())
}

pub(crate) async fn trash(connection: &mut SqliteConnection, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE files SET trashed_at = 9 WHERE id = ?1")
        .bind(id)
        .execute(connection)
        .await?;
    Ok(())
}

pub(crate) async fn start_folder(
    connection: &mut SqliteConnection,
    new_share_id: &str,
    owner: &str,
    folder: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query(UPSERT_FOLDER_SHARE)
        .bind(new_share_id)
        .bind(owner)
        .bind(folder)
        .bind(1)
        .fetch_one(connection)
        .await?
        .try_get("share_id")
}

pub(crate) async fn strings(
    connection: &mut SqliteConnection,
    query: &str,
    owner: &str,
    column: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query(query)
        .bind(owner)
        .fetch_all(connection)
        .await?
        .iter()
        .map(|row| row.try_get(column))
        .collect()
}

pub(crate) async fn migrated() -> Result<SqliteConnection, sqlx::Error> {
    let mut connection = connect().await?;
    migrate_up_to_file_shares(&mut connection).await?;
    connection.execute(SHARES_POINT_AT_FILES).await?;
    connection.execute(FOLDER_SHARES).await?;
    add_user(&mut connection, "u1").await?;
    add_user(&mut connection, "u2").await?;
    Ok(connection)
}
