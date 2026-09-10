//! Scaffold for the Rust rewrite of the Synced Share Worker (see
//! `TODO-synced-share-rust-d1-migration.md`, task 2). This crate is
//! deliberately a stub: it only proves the `worker` dependency and the
//! shadow-SQLite build step (`build.rs`) work end to end for
//! `wasm32-unknown-unknown`. The actual `resolveRole`/`doc`/`index`/
//! `protocol` logic ported from the TypeScript worker under
//! `live-share-worker/src/*.ts` lands in a later task.

use worker::{event, Context, Env, Request, Response, Result};

#[event(fetch)]
async fn fetch(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    // Placeholder handler -- real request routing lands in a later task.
    Response::ok("live-share-worker: not yet implemented")
}
