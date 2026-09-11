//! Unit tests for the D1-free retry/backoff and TTL logic in
//! `verification.rs` (`TODO-synced-share-rust-d1-migration.md` §0/§6, task
//! 7). Per that section's testing decision, the GitHub verification call
//! itself is mocked here -- a fake, counting operation stands in for
//! `identity::github::GithubIdentityProvider`, and `sleep` is instant --
//! rather than hitting real GitHub or a real timer.
#![allow(clippy::disallowed_macros)]

use std::cell::Cell;
use std::time::Duration;

use live_share_worker::verification::{retry_with_backoff, session_is_fresh, RetryFailure};

/// Stands in for `worker::Delay` -- resolves immediately, but records how
/// many times (and for how long) it was asked to sleep.
async fn instant_sleep(delays: &Cell<Vec<Duration>>, delay: Duration) {
    let mut recorded = delays.take();
    recorded.push(delay);
    delays.set(recorded);
}

#[tokio::test]
async fn succeeds_on_the_first_attempt_without_sleeping() {
    let attempts = Cell::new(0u32);
    let delays: Cell<Vec<Duration>> = Cell::new(Vec::new());

    let result = retry_with_backoff(
        2,
        Duration::from_secs(2),
        || {
            attempts.set(attempts.get() + 1);
            std::future::ready(Ok::<_, &'static str>("resolved-user-id"))
        },
        |delay| instant_sleep(&delays, delay),
    )
    .await;

    assert_eq!(result.expect("should succeed"), "resolved-user-id");
    assert_eq!(attempts.get(), 1);
    assert!(delays.take().is_empty());
}

#[tokio::test]
async fn retries_on_failure_and_succeeds_before_exhausting_retries() {
    let attempts = Cell::new(0u32);
    let delays: Cell<Vec<Duration>> = Cell::new(Vec::new());

    let result = retry_with_backoff(
        2,
        Duration::from_secs(2),
        || {
            let attempt = attempts.get() + 1;
            attempts.set(attempt);
            async move {
                if attempt < 2 {
                    Err("github verification failed: status=500")
                } else {
                    Ok("resolved-user-id")
                }
            }
        },
        |delay| instant_sleep(&delays, delay),
    )
    .await;

    assert_eq!(result.expect("should succeed"), "resolved-user-id");
    assert_eq!(attempts.get(), 2);
    assert_eq!(delays.take().len(), 1, "should sleep once between attempts");
}

#[tokio::test]
async fn fails_closed_after_exhausting_all_retries() {
    let attempts = Cell::new(0u32);
    let delays: Cell<Vec<Duration>> = Cell::new(Vec::new());

    let result = retry_with_backoff(
        2,
        Duration::from_secs(2),
        || {
            attempts.set(attempts.get() + 1);
            std::future::ready(Err::<&'static str, _>(
                "github verification failed: status=401",
            ))
        },
        |delay| instant_sleep(&delays, delay),
    )
    .await;

    let RetryFailure {
        last_error,
        attempts: attempts_made,
    } = result.expect_err("should fail closed");

    assert_eq!(last_error, "github verification failed: status=401");
    assert_eq!(attempts_made, 3, "initial attempt plus 2 retries");
    assert_eq!(attempts.get(), 3);
    assert_eq!(delays.take().len(), 2, "should sleep between each retry");
}

#[tokio::test]
async fn never_retries_when_max_retries_is_zero() {
    let attempts = Cell::new(0u32);
    let delays: Cell<Vec<Duration>> = Cell::new(Vec::new());

    let result = retry_with_backoff(
        0,
        Duration::from_secs(2),
        || {
            attempts.set(attempts.get() + 1);
            std::future::ready(Err::<&'static str, _>("github verification failed"))
        },
        |delay| instant_sleep(&delays, delay),
    )
    .await;

    assert!(result.is_err());
    assert_eq!(attempts.get(), 1);
    assert!(delays.take().is_empty());
}

#[test]
fn treats_a_session_within_the_ttl_as_fresh() {
    assert!(session_is_fresh(
        1_000,
        1_000 + 59 * 60 * 1000,
        60 * 60 * 1000
    ));
}

#[test]
fn treats_a_session_past_the_ttl_as_stale() {
    assert!(!session_is_fresh(
        1_000,
        1_000 + 61 * 60 * 1000,
        60 * 60 * 1000
    ));
}

#[test]
fn treats_clock_skew_before_verified_at_as_fresh() {
    assert!(session_is_fresh(10_000, 1_000, 60 * 60 * 1000));
}
