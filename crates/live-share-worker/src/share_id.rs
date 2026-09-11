//! Random id generation: server-side `share_id`s (per
//! `TODO-synced-share-rust-d1-migration.md` §1 -- no more client-derived
//! id) and the pseudo `users.id` the identity stub mints on
//! create-on-first-sight.
//!
//! Uses `js_sys::Math::random()` (re-exported by `worker`) rather than
//! pulling in a `rand`/`getrandom` dependency -- it's already available in
//! every Worker's JS environment and keeps this crate's dependency surface
//! small.
//!
//! `generate_unique_id` is D1-/worker-free (generic over an injected
//! `exists` check), so `tests/share_id.rs` exercises the collision-handling
//! loop with a fake `exists` closure -- no real D1 needed. Matches this
//! crate's existing split between pure/testable logic and
//! D1-/wasm-runtime-facing glue (see `lib.rs`'s module doc comment).

use std::future::Future;

use worker::{D1Database, Result};

use crate::db;

/// Matches `SHARE_ID_LENGTH` in `web/src/syncedShareUrl.ts` -- do not
/// change without updating that file too (out of scope for this crate).
/// Every `share_id` this crate mints is exactly this many characters,
/// including on collision (see `generate_unique_share_id`) -- no consumer
/// of a `share_id`, here or client-side, needs to handle more than one
/// length.
pub(crate) const SHARE_ID_LENGTH: usize = 11;

/// Not constrained by any client-side pattern (internal id only) -- just
/// long enough to be unguessable, per the schema's own comment ("same style
/// as share_id -- not sequential").
pub(crate) const USER_ID_LENGTH: usize = 21;

/// Matches `SHARE_ID_PATTERN`'s charset (`[0-9A-Za-z_-]`) in
/// `web/src/syncedShareUrl.ts`.
const ID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

pub(crate) fn generate_id(length: usize) -> String {
    (0..length).map(|_| random_char()).collect()
}

fn random_char() -> char {
    let index = (worker::js_sys::Math::random() * ID_CHARSET.len() as f64) as usize;
    let index = index.min(ID_CHARSET.len().saturating_sub(1));
    let byte = ID_CHARSET.get(index).copied().unwrap_or(b'0');
    byte as char
}

/// Generates a fresh id, re-rolling a brand new candidate (via
/// `generate_candidate`, same length every time, never extended) on each
/// collision reported by `exists`, per the TODO §8 fix for "shareId
/// collision fallback can mint an id the client can't parse back out of a
/// link": the earlier length-extension fallback minted a 12-char id that
/// `web/src/syncedShareUrl.ts`'s fixed-length `SHARE_ID_PATTERN` couldn't
/// parse. Re-rolling instead keeps every id this crate mints exactly the
/// same length, so no consumer needs to handle more than one length.
///
/// Takes `generate_candidate` as a parameter (rather than calling
/// `generate_id` directly) so this loop is `js_sys`-/worker-free and
/// `tests/share_id.rs` can drive it with a fake, deterministic candidate
/// generator -- `js_sys::Math::random()` (used by the real `generate_id`)
/// panics outside a wasm target.
///
/// This is a true loop rather than a fixed number of attempts, but a
/// second collision is astronomically unlikely at this charset/length
/// (per TODO §1's original collision-handling decision), so it isn't a
/// practical infinite loop.
pub async fn generate_unique_id<GenerateCandidate, Exists, ExistsFut, E>(
    mut generate_candidate: GenerateCandidate,
    mut exists: Exists,
) -> Result<String, E>
where
    GenerateCandidate: FnMut() -> String,
    Exists: FnMut(String) -> ExistsFut,
    ExistsFut: Future<Output = Result<bool, E>>,
{
    loop {
        let candidate = generate_candidate();
        if !exists(candidate.clone()).await? {
            return Ok(candidate);
        }
    }
}

/// Generates a fresh, D1-unique `share_id`.
pub(crate) async fn generate_unique_share_id(db: &D1Database) -> Result<String> {
    generate_unique_id(
        || generate_id(SHARE_ID_LENGTH),
        |candidate| async move { db::share_id_exists(db, &candidate).await },
    )
    .await
}
