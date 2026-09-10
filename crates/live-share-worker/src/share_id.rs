//! Random id generation: server-side `share_id`s (per
//! `TODO-synced-share-rust-d1-migration.md` §1 -- no more client-derived
//! id) and the pseudo `users.id` the identity stub mints on
//! create-on-first-sight.
//!
//! Uses `js_sys::Math::random()` (re-exported by `worker`) rather than
//! pulling in a `rand`/`getrandom` dependency -- it's already available in
//! every Worker's JS environment and keeps this crate's dependency surface
//! small.

use worker::{D1Database, Result};

use crate::db;

/// Matches `SHARE_ID_LENGTH` in `web/src/syncedShareUrl.ts` -- do not
/// change without updating that file too (out of scope for this crate).
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

/// Generates a fresh, D1-unique `share_id`.
///
/// Collision handling per TODO §1: a collision is already astronomically
/// unlikely at this id length, so on the rare hit this appends exactly one
/// extra character from the same charset (belt-and-suspenders) rather than
/// looping/regenerating. That fallback id is one character longer than
/// `SHARE_ID_LENGTH`, so it would not match today's client-side
/// `SHARE_ID_PATTERN` in `web/src/syncedShareUrl.ts` (fixed-length) -- an
/// accepted, pre-existing tension flagged as an open item in TODO §1, not
/// something this task changes `web/` to fix.
pub(crate) async fn generate_unique_share_id(db: &D1Database) -> Result<String> {
    let candidate = generate_id(SHARE_ID_LENGTH);
    if !db::share_id_exists(db, &candidate).await? {
        return Ok(candidate);
    }

    let mut extended = candidate;
    extended.push(random_char());
    Ok(extended)
}
