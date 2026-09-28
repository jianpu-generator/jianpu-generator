//! Writes the worker's OpenAPI spec (`live_share_worker::openapi_json`) to
//! `web/src/generated/live-share-worker/openapi.json`, which `web`'s
//! `build:worker-types` script turns into its typed client -- see
//! `crates/live-share-worker/src/handlers/routes.rs`. Also fails if the
//! route list itself is invalid (e.g. a path template that doesn't match its
//! handler's path-params struct).
//!
//! Alongside it, writes `shareIdFormat.json`
//! (`live_share_worker::share_id::share_id_format`): the `share_id` length
//! and pattern the web client parses share links with. It's a separate
//! file (rather than read out of `openapi.json`) because it's a runtime
//! value, not a type, and keeps the whole spec out of the SPA/Pages
//! Function bundles.

use std::fs;
use std::path::PathBuf;

use live_share_worker::share_id::share_id_format;

#[test]
fn export_openapi() -> Result<(), Box<dyn std::error::Error>> {
    let json = live_share_worker::openapi_json()?;
    let out_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/src/generated/live-share-worker");
    fs::create_dir_all(&out_dir)?;
    fs::write(out_dir.join("openapi.json"), json)?;
    fs::write(
        out_dir.join("shareIdFormat.json"),
        serde_json::to_string_pretty(&share_id_format())?,
    )?;
    Ok(())
}
