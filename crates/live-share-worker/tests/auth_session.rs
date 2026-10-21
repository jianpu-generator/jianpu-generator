//! Unit tests for `auth::session`: the stored token hash and when a used
//! session's expiry moves.
#![allow(clippy::disallowed_macros)]

use live_share_worker::auth::session::{
    extended_expiry, hash_token, SESSION_EXTEND_BELOW_MILLIS, SESSION_LIFETIME_MILLIS,
};

const NOW: i64 = 1_760_000_000_000;
const DAY: i64 = 24 * 60 * 60 * 1000;

#[test]
fn a_session_lasts_sixty_days() {
    assert_eq!(SESSION_LIFETIME_MILLIS, 60 * DAY);
}

#[test]
fn hashes_a_token_as_lowercase_hex_sha256() {
    assert_eq!(
        hash_token("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn a_fresh_session_is_left_alone() {
    assert_eq!(extended_expiry(NOW + SESSION_LIFETIME_MILLIS, NOW), None);
    assert_eq!(
        extended_expiry(NOW + SESSION_EXTEND_BELOW_MILLIS, NOW),
        None
    );
}

#[test]
fn a_session_used_a_day_later_gets_sixty_days_from_now() {
    assert_eq!(
        extended_expiry(NOW + SESSION_EXTEND_BELOW_MILLIS - 1, NOW),
        Some(NOW + SESSION_LIFETIME_MILLIS)
    );
}
