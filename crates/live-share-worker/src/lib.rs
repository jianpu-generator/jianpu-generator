//! Entry point for the Rust rewrite of the Synced Share Worker (see
//! `TODO-synced-share-rust-d1-migration.md`, task 4, and the "Synced Share
//! worker" section of `ARCHITECTURE.md` for the full picture).
//!
//! Ports `live-share-worker/src/{resolveRole,doc,index,protocol}.ts` to
//! Rust, retargeted from Workers KV + an anonymous `ownerToken` to D1 +
//! GitHub-required ownership (`docs.owner_user_id`). Identity resolution
//! routes every write through `identity::resolve_verified_user_id`, a
//! hashed-token cache in front of `identity::github::GithubIdentityProvider`
//! (real `GET /user` verification), per
//! `TODO-synced-share-rust-d1-migration.md` §0/§6.
//!
//! `doc`, `protocol`, `resolve_role`, and `verification` are `pub` (and
//! D1/JsValue-free) so `tests/*.rs` can unit-test them directly, per this
//! repo's convention of keeping tests in separate files rather than inline
//! `#[cfg(test)]` modules. Everything else here is D1- or
//! wasm-runtime-facing and stays crate-private -- it isn't exercised by
//! host-side `cargo test` (per `TODO-synced-share-rust-d1-migration.md` §0:
//! real D1 integration testing is deferred to `wrangler dev`, not built
//! here; the GitHub verification call itself is mocked in `verification`'s
//! unit tests instead of hit for real, per that section's testing
//! decision).

mod db;
mod handlers;
mod identity;
mod oauth;

pub mod doc;
pub mod protocol;
pub mod resolve_role;
pub mod share_id;
pub mod verification;

use worker::{event, Context, Cors, Env, Method, Request, Response, Result};

/// The web app calls this worker cross-origin, so every response --
/// including the `OPTIONS` preflight a `POST`'s JSON body triggers -- needs
/// these. `*` is fine: there's no cookie/session auth here, just an
/// unguessable `share_id` plus an `identity_token` in the request body,
/// neither of which `Access-Control-Allow-Origin` exposes to an origin that
/// doesn't already have them (same reasoning as the old TS `index.ts`).
fn cors_config() -> Cors {
    Cors::new()
        .with_origins(["*"])
        .with_methods([Method::Get, Method::Post, Method::Options])
        .with_allowed_headers(["Content-Type"])
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let cors = cors_config();

    if req.method() == Method::Options {
        return Response::empty()?.with_status(204).with_cors(&cors);
    }

    let response = handlers::router().run(req, env).await?;
    response.with_cors(&cors)
}
