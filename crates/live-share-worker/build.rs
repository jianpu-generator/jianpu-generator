//! Shadow-SQLite build step.
//!
//! See the long comment on the `[build-dependencies] sqlx` entry in
//! `Cargo.toml` for why `sqlx` lives here (native, build-time only) rather
//! than as a normal dependency of this crate.
//!
//! This script (re)creates a local shadow SQLite database
//! (`crates/live-share-worker/shadow.sqlite`, gitignored) by applying every
//! migration under `live-share-worker/migrations/` (the *existing*
//! top-level directory -- see the task's naming note; this build step reads
//! those files in place rather than duplicating or moving them) in order.
//!
//! Nothing in this crate uses `sqlx::query_file!` yet (that lands with
//! real queries in a later task), so there is nothing to check the shadow
//! DB *against* yet -- this step only proves the DB gets created and
//! migrated correctly, ready for that later task to point compile-time
//! query macros at it.
//!
//! Build scripts always compile and run for the *host* target, never for
//! the crate's own `--target`, so this runs unchanged whether the crate
//! itself is being checked for `wasm32-unknown-unknown` or the host.

use std::error::Error;
use std::path::{Path, PathBuf};

use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Executor};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // `live-share-worker/migrations` is a sibling of `crates/`, i.e.
    // `<repo root>/live-share-worker/migrations` -- referenced relatively
    // from this crate at `crates/live-share-worker/`, not duplicated here.
    let migrations_dir = manifest_dir.join("../../live-share-worker/migrations");
    let migrations_dir = migrations_dir.canonicalize()?;

    println!("cargo:rerun-if-changed={}", migrations_dir.display());
    for entry in std::fs::read_dir(&migrations_dir)?.flatten() {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }
    println!("cargo:rerun-if-changed=build.rs");

    let shadow_db_path = manifest_dir.join("shadow.sqlite");

    // Rebuild the shadow DB from scratch each time build.rs runs (it only
    // runs when the migrations actually changed, per the rerun-if-changed
    // directives above), rather than trying to apply migrations
    // incrementally against a possibly-stale file.
    if shadow_db_path.exists() {
        std::fs::remove_file(&shadow_db_path)?;
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(migrate_shadow_db(&shadow_db_path, &migrations_dir))
}

async fn migrate_shadow_db(
    shadow_db_path: &Path,
    migrations_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let connect_options = SqliteConnectOptions::new()
        .filename(shadow_db_path)
        .create_if_missing(true);

    let mut connection = connect_options.connect().await?;

    // Applying the migrations directly (rather than via `sqlx migrate!`,
    // which wants them compiled into the binary) keeps this a pure
    // build-time step with no runtime footprint in the crate itself.
    let migrator = Migrator::new(migrations_dir).await?;
    migrator.run(&mut connection).await?;

    // Sanity check: confirm the tables from 0001_init.sql actually landed,
    // so a silently-empty shadow db doesn't pass this build step unnoticed.
    connection.execute("SELECT id FROM users LIMIT 0").await?;

    Ok(())
}
