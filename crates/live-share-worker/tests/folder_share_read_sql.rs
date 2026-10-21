//! The folder-share SQL run for real against SQLite (2/2): folder reads,
//! the viewer header, the overview, the prefix rewrite and the e2e reset.
#![allow(clippy::disallowed_macros)]
#![cfg(not(target_arch = "wasm32"))]

mod support;

use sqlx::Row;
use support::*;

#[tokio::test]
async fn a_folder_read_matches_whole_segments_and_skips_trashed_and_foreign_files(
) -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    add_file(&mut connection, "a", "u1", "Hymns/a.jianpu", "A").await?;
    add_file(&mut connection, "k", "u1", "Hymns/Empty/.keep", "").await?;
    add_file(&mut connection, "n", "u1", "Hymns/Sub/n.jianpu", "NN").await?;
    add_file(&mut connection, "t", "u1", "Hymns/trashed.jianpu", "T").await?;
    add_file(&mut connection, "o", "u1", "Hymns 2/o.jianpu", "O").await?;
    add_file(&mut connection, "p", "u1", "Hymns.jianpu", "P").await?;
    add_file(&mut connection, "w", "u1", "Hymns%/w.jianpu", "W").await?;
    add_file(&mut connection, "x", "u2", "Hymns/x.jianpu", "X").await?;
    trash(&mut connection, "t").await?;

    let stats = sqlx::query(GET_FOLDER_FILES_STATS)
        .bind("u1")
        .bind("Hymns")
        .fetch_one(&mut connection)
        .await?;
    let names: Vec<String> = sqlx::query(LIST_FOLDER_FILES)
        .bind("u1")
        .bind("Hymns")
        .fetch_all(&mut connection)
        .await?
        .iter()
        .map(|row| row.try_get("name"))
        .collect::<Result<_, _>>()?;
    let wildcard = sqlx::query(LIST_FOLDER_FILES)
        .bind("u1")
        .bind("Hymns%")
        .fetch_all(&mut connection)
        .await?;

    assert_eq!(stats.try_get::<i64, _>("file_count")?, 3);
    assert_eq!(stats.try_get::<i64, _>("content_bytes")?, 3);
    assert_eq!(
        names,
        ["Hymns/Empty/.keep", "Hymns/Sub/n.jianpu", "Hymns/a.jianpu"]
    );
    assert_eq!(wildcard.len(), 1, "% is a literal, not a wildcard");
    Ok(())
}

#[tokio::test]
async fn content_bytes_count_utf8_bytes_not_characters() -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    add_file(&mut connection, "a", "u1", "F/a.jianpu", "你好").await?;

    let stats = sqlx::query(GET_FOLDER_FILES_STATS)
        .bind("u1")
        .bind("F")
        .fetch_one(&mut connection)
        .await?;

    assert_eq!(stats.try_get::<i64, _>("content_bytes")?, 6);
    Ok(())
}

#[tokio::test]
async fn the_viewer_header_resolves_a_folder_share_and_not_a_file_share() -> Result<(), sqlx::Error>
{
    let mut connection = migrated().await?;
    add_file(&mut connection, "f1", "u1", "a.jianpu", "").await?;
    start_folder(&mut connection, "folder-id", "u1", "Hymns/Advent").await?;
    sqlx::query(UPSERT_SHARE)
        .bind("file-id")
        .bind("u1")
        .bind("f1")
        .bind(1)
        .execute(&mut connection)
        .await?;

    let header = sqlx::query(GET_FOLDER_SHARE_HEADER)
        .bind("folder-id")
        .fetch_one(&mut connection)
        .await?;
    let file_share = sqlx::query(GET_FOLDER_SHARE_HEADER)
        .bind("file-id")
        .fetch_optional(&mut connection)
        .await?;
    let folder_as_file = sqlx::query(GET_SHARE_VIEW)
        .bind("folder-id")
        .fetch_optional(&mut connection)
        .await?;

    assert_eq!(
        header.try_get::<String, _>("folder_prefix")?,
        "Hymns/Advent"
    );
    assert_eq!(header.try_get::<String, _>("owner_user_id")?, "u1");
    assert!(file_share.is_none());
    assert!(folder_as_file.is_none());
    Ok(())
}

#[tokio::test]
async fn the_overview_lists_only_live_links_of_the_owner() -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    add_file(&mut connection, "live", "u1", "Hymns/live.jianpu", "").await?;
    add_file(&mut connection, "stopped", "u1", "stopped.jianpu", "").await?;
    add_file(&mut connection, "binned", "u1", "binned.jianpu", "").await?;
    add_file(&mut connection, "theirs", "u2", "theirs.jianpu", "").await?;
    for (share_id, owner, file) in [
        ("s-live", "u1", "live"),
        ("s-stopped", "u1", "stopped"),
        ("s-binned", "u1", "binned"),
        ("s-theirs", "u2", "theirs"),
    ] {
        sqlx::query(UPSERT_SHARE)
            .bind(share_id)
            .bind(owner)
            .bind(file)
            .bind(1)
            .execute(&mut connection)
            .await?;
    }
    sqlx::query(END_SHARE)
        .bind(5)
        .bind("stopped")
        .bind("u1")
        .execute(&mut connection)
        .await?;
    trash(&mut connection, "binned").await?;
    start_folder(&mut connection, "f-live", "u1", "Hymns").await?;
    start_folder(&mut connection, "f-stopped", "u1", "Old").await?;
    sqlx::query(END_FOLDER_SHARE)
        .bind(5)
        .bind("u1")
        .bind("Old")
        .execute(&mut connection)
        .await?;
    start_folder(&mut connection, "f-theirs", "u2", "Theirs").await?;

    let files = strings(&mut connection, LIST_LIVE_FILE_SHARES, "u1", "name").await?;
    let folders = strings(
        &mut connection,
        LIST_LIVE_FOLDER_SHARES,
        "u1",
        "folder_prefix",
    )
    .await?;
    let prefixes = strings(
        &mut connection,
        LIST_FOLDER_SHARE_PREFIXES,
        "u1",
        "folder_prefix",
    )
    .await?;

    assert_eq!(files, ["Hymns/live.jianpu"]);
    assert_eq!(folders, ["Hymns"]);
    assert_eq!(
        prefixes.len(),
        2,
        "stopped folder links are still tracked for renames"
    );
    Ok(())
}

#[tokio::test]
async fn a_batch_rename_and_a_prefix_rewrite_apply_together() -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    add_file(&mut connection, "a", "u1", "Hymns/a.jianpu", "").await?;
    add_file(&mut connection, "b", "u1", "Hymns/Advent/b.jianpu", "").await?;
    add_file(&mut connection, "o", "u1", "Hymns 2/o.jianpu", "").await?;
    start_folder(&mut connection, "s-hymns", "u1", "Hymns").await?;
    start_folder(&mut connection, "s-advent", "u1", "Hymns/Advent").await?;
    start_folder(&mut connection, "s-other", "u1", "Hymns 2").await?;
    let prefixes = vec![
        "Hymns".to_string(),
        "Hymns/Advent".to_string(),
        "Hymns 2".to_string(),
    ];

    let under = sqlx::query(LIST_ACTIVE_FILES_UNDER_PREFIXES)
        .bind("u1")
        .bind(serde_json::to_string(&prefixes).unwrap())
        .fetch_all(&mut connection)
        .await?;
    assert_eq!(under.len(), 3);

    let mut transaction = sqlx::Connection::begin(&mut connection).await?;
    sqlx::query(RENAME_FILES_BATCH)
        .bind(r#"[{"id":"a","name":"Songs/a.jianpu"},{"id":"b","name":"Songs/Advent/b.jianpu"}]"#)
        .bind(5)
        .bind("u1")
        .execute(&mut *transaction)
        .await?;
    for (to, from) in [("Songs", "Hymns"), ("Songs/Advent", "Hymns/Advent")] {
        sqlx::query(REWRITE_FOLDER_SHARE_PREFIX)
            .bind(to)
            .bind("u1")
            .bind(from)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;

    let mut now = strings(
        &mut connection,
        LIST_LIVE_FOLDER_SHARES,
        "u1",
        "folder_prefix",
    )
    .await?;
    now.sort();
    assert_eq!(now, ["Hymns 2", "Songs", "Songs/Advent"]);
    let ids: Vec<String> = sqlx::query("SELECT share_id FROM shares WHERE folder_prefix = 'Songs'")
        .fetch_all(&mut connection)
        .await?
        .iter()
        .map(|row| row.try_get("share_id"))
        .collect::<Result<_, _>>()?;
    assert_eq!(ids, ["s-hymns"], "the link keeps working: same share id");
    Ok(())
}

#[tokio::test]
async fn a_rewrite_onto_a_folder_that_has_its_own_share_is_rejected_and_rolls_back(
) -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    add_file(&mut connection, "a", "u1", "Hymns/a.jianpu", "").await?;
    start_folder(&mut connection, "s-hymns", "u1", "Hymns").await?;
    start_folder(&mut connection, "s-songs", "u1", "Songs").await?;

    let mut transaction = sqlx::Connection::begin(&mut connection).await?;
    sqlx::query(RENAME_FILES_BATCH)
        .bind(r#"[{"id":"a","name":"Songs/a.jianpu"}]"#)
        .bind(5)
        .bind("u1")
        .execute(&mut *transaction)
        .await?;
    let rewrite = sqlx::query(REWRITE_FOLDER_SHARE_PREFIX)
        .bind("Songs")
        .bind("u1")
        .bind("Hymns")
        .execute(&mut *transaction)
        .await;
    assert!(rewrite.is_err());
    drop(transaction);

    let name: String = sqlx::query("SELECT name FROM files WHERE id = 'a'")
        .fetch_one(&mut connection)
        .await?
        .try_get("name")?;
    assert_eq!(name, "Hymns/a.jianpu");
    Ok(())
}

#[tokio::test]
async fn the_e2e_reset_removes_folder_shares_of_the_listed_accounts_only() -> Result<(), sqlx::Error>
{
    let mut connection = migrated().await?;
    start_folder(&mut connection, "s-mine", "u1", "Hymns").await?;
    start_folder(&mut connection, "s-theirs", "u2", "Hymns").await?;

    let deleted = sqlx::query(DELETE_SHARES_OF_USERS)
        .bind(r#"["u1"]"#)
        .execute(&mut connection)
        .await?;

    assert_eq!(deleted.rows_affected(), 1);
    let left = strings(
        &mut connection,
        LIST_LIVE_FOLDER_SHARES,
        "u2",
        "folder_prefix",
    )
    .await?;
    assert_eq!(left, ["Hymns"]);
    Ok(())
}
