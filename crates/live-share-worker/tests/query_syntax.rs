//! Native-only SQL syntax check for the `.sql` query files under
//! `queries/`, run against the shadow SQLite database `build.rs` builds
//! from `live-share-worker/migrations/`.
//!
//! This is the pragmatic substitute for `sqlx::query_file!`'s compile-time
//! checking described in `src/db.rs`'s doc comment and the
//! `[build-dependencies] sqlx` comment in `Cargo.toml`: those macros can't
//! run inside this crate's own wasm-targeted compile (the
//! `libsqlite3-sys`-on-`wasm32-unknown-unknown` problem), so instead this
//! test loads the exact same `include_str!`-embedded query text used by
//! `src/db.rs` at runtime and asks `sqlx` (here a dev-dependency, so it
//! never touches the wasm build) to *prepare* -- not execute -- each one
//! against the shadow schema. Preparing (rather than executing) means no
//! dummy bound values are needed and `NOT NULL`/foreign-key constraints
//! never come into play; this only proves each query parses and its
//! tables/columns exist, which is weaker than `query_file!`'s full
//! bound-parameter and result-column type checking, and does not catch any
//! D1-vs-SQLite dialect drift -- an honest, reasonable substitute given the
//! constraint above, not a silent downgrade of it.
//!
//! Not run under the `wasm32-unknown-unknown` target (this repo's `cargo
//! check --target wasm32-unknown-unknown` only checks, never tests, that
//! target, but the `cfg` below is a defensive belt-and-suspenders in case
//! that ever changes).

#![cfg(not(target_arch = "wasm32"))]

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Executor};

const QUERIES: &[(&str, &str)] = &[
    (
        "get_doc_by_share_id.sql",
        include_str!("../queries/get_doc_by_share_id.sql"),
    ),
    ("insert_doc.sql", include_str!("../queries/insert_doc.sql")),
    ("update_doc.sql", include_str!("../queries/update_doc.sql")),
    (
        "share_id_exists.sql",
        include_str!("../queries/share_id_exists.sql"),
    ),
    (
        "get_user_identity.sql",
        include_str!("../queries/get_user_identity.sql"),
    ),
    (
        "insert_user.sql",
        include_str!("../queries/insert_user.sql"),
    ),
    (
        "insert_user_identity.sql",
        include_str!("../queries/insert_user_identity.sql"),
    ),
    (
        "get_oauth_session.sql",
        include_str!("../queries/get_oauth_session.sql"),
    ),
    (
        "upsert_oauth_session.sql",
        include_str!("../queries/upsert_oauth_session.sql"),
    ),
    (
        "get_owner_login.sql",
        include_str!("../queries/get_owner_login.sql"),
    ),
];

#[tokio::test]
async fn every_query_file_parses_against_the_shadow_schema() -> Result<(), sqlx::Error> {
    let shadow_db_path = concat!(env!("CARGO_MANIFEST_DIR"), "/shadow.sqlite");
    let mut connection = SqliteConnectOptions::new()
        .filename(shadow_db_path)
        .connect()
        .await?;

    for (name, query) in QUERIES {
        connection
            .prepare(query)
            .await
            .map_err(|error| sqlx::Error::Protocol(format!("{name} failed to parse: {error}")))?;
    }

    Ok(())
}
