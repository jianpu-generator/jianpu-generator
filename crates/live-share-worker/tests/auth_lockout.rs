//! Unit tests for `auth::lockout`: when wrong passwords lock an email, for
//! how long, and the countdown the 429 carries.
#![allow(clippy::disallowed_macros)]

use live_share_worker::auth::lockout::{
    after_failed_sign_in, retry_after_seconds, LockoutState, BASE_LOCKOUT_MILLIS,
    MAX_FAILED_SIGN_INS, MAX_LOCKOUT_LEVEL,
};

const NOW: i64 = 1_760_000_000_000;

fn after_failures(count: i64, start: Option<LockoutState>) -> Option<LockoutState> {
    (0..count).fold(start, |state, _| {
        Some(after_failed_sign_in(state.as_ref(), NOW))
    })
}

#[test]
fn four_wrong_passwords_only_count() {
    assert_eq!(
        after_failures(MAX_FAILED_SIGN_INS - 1, None),
        Some(LockoutState {
            failed_count: 4,
            locked_until: None,
            lockout_level: 0,
        })
    );
}

#[test]
fn the_fifth_wrong_password_locks_for_fifteen_minutes() {
    assert_eq!(
        after_failures(MAX_FAILED_SIGN_INS, None),
        Some(LockoutState {
            failed_count: 0,
            locked_until: Some(NOW + 15 * 60 * 1000),
            lockout_level: 1,
        })
    );
}

#[test]
fn each_later_lockout_doubles() {
    let second = after_failures(2 * MAX_FAILED_SIGN_INS, None);

    assert_eq!(
        second.and_then(|state| state.locked_until),
        Some(NOW + 30 * 60 * 1000)
    );
}

#[test]
fn the_pause_stops_doubling_at_the_max_level() {
    let state = LockoutState {
        failed_count: MAX_FAILED_SIGN_INS - 1,
        locked_until: None,
        lockout_level: MAX_LOCKOUT_LEVEL + 20,
    };

    assert_eq!(
        after_failed_sign_in(Some(&state), NOW).locked_until,
        Some(NOW + (BASE_LOCKOUT_MILLIS << MAX_LOCKOUT_LEVEL))
    );
}

#[test]
fn retry_after_rounds_up_to_whole_seconds() {
    let state = LockoutState {
        failed_count: 0,
        locked_until: Some(NOW + 1_001),
        lockout_level: 1,
    };

    assert_eq!(retry_after_seconds(&state, NOW), Some(2));
    assert_eq!(retry_after_seconds(&state, NOW + 1_000), Some(1));
}

#[test]
fn a_lockout_that_has_ended_or_never_started_has_no_retry_after() {
    let ended = LockoutState {
        failed_count: 0,
        locked_until: Some(NOW),
        lockout_level: 1,
    };
    let never = LockoutState {
        failed_count: 3,
        locked_until: None,
        lockout_level: 0,
    };

    assert_eq!(retry_after_seconds(&ended, NOW), None);
    assert_eq!(retry_after_seconds(&never, NOW), None);
}
