//! D1-free, unit-testable pieces of the identity-verification cache
//! described in `TODO-synced-share-rust-d1-migration.md` §0/§6 (task 7):
//! the `oauth_sessions` TTL check and the backoff-wrapped retry driver
//! wrapping a GitHub verification call. The D1-facing glue that calls these
//! (hashing the token, reading/writing `oauth_sessions`, calling the real
//! `IdentityProvider`) lives in `crate::identity` and is not exercised by
//! host-side `cargo test`, matching this crate's existing split between
//! pure/testable modules (`doc`, `protocol`, `resolve_role`) and
//! D1-/wasm-runtime-facing ones -- see `lib.rs`'s module doc comment.
//!
//! `retry_with_backoff` is generic over the operation and the sleep
//! function, so unit tests (`tests/verification.rs`) exercise the exact
//! retry/backoff contract from TODO §0 (2 retries, exponential delay, fail
//! closed) using a fake operation and an instant `sleep` -- no real GitHub
//! call, no real timer, per that section's testing decision ("mock the
//! GitHub verification call in unit tests").

use std::future::Future;
use std::time::Duration;

use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use serde::Serialize;

/// `oauth_sessions.verified_at` TTL, per the schema comment in
/// `migrations/0001_init.sql` ("~1hr TTL checked in app code").
pub const SESSION_TTL_MILLIS: i64 = 60 * 60 * 1000;

/// TODO §0's retry/backoff policy: 2 retries (so up to 3 attempts total),
/// capped around ~2s of total wait before failing closed.
pub const MAX_RETRIES: u32 = 2;
pub const MAX_TOTAL_WAIT: Duration = Duration::from_secs(2);

/// Whether a cached `oauth_sessions` row (verified at `verified_at`) is
/// still fresh at `now`, both in milliseconds since the epoch. A `now`
/// before `verified_at` (clock skew) is treated as fresh, not as an
/// underflow -- the row was just written.
pub fn session_is_fresh(verified_at: i64, now: i64, ttl_millis: i64) -> bool {
    now.saturating_sub(verified_at) < ttl_millis
}

/// What's left of a verification attempt after `retry_with_backoff` gives
/// up: the last error seen and how many attempts were actually made.
/// Deliberately carries no token or token hash -- callers turn this into a
/// UI-surfaceable error (`crate::identity::VerificationFailure`), and TODO
/// §0's one hard redaction is that the token/hash must never reach that
/// surface, so it must never be threaded through here either.
#[derive(Debug)]
pub struct RetryFailure<E> {
    pub last_error: E,
    pub attempts: u32,
}

/// Retries `operation` (an async, side-effecting call -- the real GitHub
/// verification call in production) up to `max_retries` additional times
/// after an initial attempt, sleeping (via the injected `sleep`) for an
/// exponentially increasing delay between attempts. Bails out early,
/// without exhausting `max_retries`, if the `backoff` policy's
/// `max_total_wait` cap is reached first. Fails closed: any error from the
/// final attempt is returned as `Err`, never silently swallowed.
///
/// D1-/worker-free by design -- `sleep` stands in for `worker::Delay` in
/// production and an instant no-op in tests, so this is testable without a
/// real timer or a real Workers runtime.
pub async fn retry_with_backoff<Operation, OperationFut, Sleep, SleepFut, T, E>(
    max_retries: u32,
    max_total_wait: Duration,
    mut operation: Operation,
    sleep: Sleep,
) -> Result<T, RetryFailure<E>>
where
    Operation: FnMut() -> OperationFut,
    OperationFut: Future<Output = Result<T, E>>,
    Sleep: Fn(Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    let mut backoff = ExponentialBackoffBuilder::new()
        .with_max_elapsed_time(Some(max_total_wait))
        .build();

    let mut attempts = 0u32;
    loop {
        attempts += 1;
        match operation().await {
            Ok(value) => return Ok(value),
            Err(last_error) => {
                let retries_so_far = attempts - 1;
                if retries_so_far >= max_retries {
                    return Err(RetryFailure {
                        last_error,
                        attempts,
                    });
                }
                match backoff.next_backoff() {
                    Some(delay) => sleep(delay).await,
                    None => {
                        return Err(RetryFailure {
                            last_error,
                            attempts,
                        })
                    }
                }
            }
        }
    }
}

/// A verification failure carrying enough structured detail -- reason,
/// timestamp, attempt count -- for a future UI (task 9) to render
/// verbosely, per TODO §0's "errors are maximally verbose" decision. The
/// one hard exception from that same decision: this must never carry the
/// token or its hash, which is enforced by construction here since neither
/// ever exists as a field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationFailure {
    pub reason: String,
    pub failed_at: i64,
    pub attempts: u32,
}
