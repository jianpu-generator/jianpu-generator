//! Per-email lockout after repeated wrong passwords (the `sign_in_lockouts`
//! table). Counted per email whether or not it has an account, so a lockout
//! never reveals which emails do. The per-IP rate limit in front of the
//! sign-in route (`cloudflare.config.ts`) is separate.

/// Wrong passwords in a row that lock an email.
pub const MAX_FAILED_SIGN_INS: i64 = 5;

/// The first lockout's length; each later one doubles it.
pub const BASE_LOCKOUT_MILLIS: i64 = 15 * 60 * 1000;

/// Doublings stop here (15 min × 2^10 ≈ 10.7 days), so the pause can't
/// overflow however often the limit is reached.
pub const MAX_LOCKOUT_LEVEL: i64 = 10;

/// One `sign_in_lockouts` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockoutState {
    /// Wrong passwords since the last lockout (or the last success).
    pub failed_count: i64,
    /// While in the future, sign-in is refused without checking the
    /// password.
    pub locked_until: Option<i64>,
    /// How many times the email has been locked since its last successful
    /// sign-in; the next lockout lasts `BASE_LOCKOUT_MILLIS × 2^level`.
    pub lockout_level: i64,
}

/// Whole seconds until a lockout ends (rounded up, so the countdown never
/// reaches 0 while still locked), or `None` when `state` isn't locked at
/// `now`.
pub fn retry_after_seconds(state: &LockoutState, now: i64) -> Option<i64> {
    let remaining = state.locked_until? - now;
    (remaining > 0).then(|| (remaining + 999) / 1000)
}

/// The row after one more wrong password at `now`. The `MAX_FAILED_SIGN_INS`th
/// in a row locks the email and starts counting afresh, so the next lockout
/// again takes `MAX_FAILED_SIGN_INS` wrong passwords once this one ends.
pub fn after_failed_sign_in(state: Option<&LockoutState>, now: i64) -> LockoutState {
    let lockout_level = state.map_or(0, |state| state.lockout_level);
    let failed_count = state.map_or(0, |state| state.failed_count) + 1;
    if failed_count < MAX_FAILED_SIGN_INS {
        return LockoutState {
            failed_count,
            locked_until: state.and_then(|state| state.locked_until),
            lockout_level,
        };
    }
    let pause = BASE_LOCKOUT_MILLIS << lockout_level.clamp(0, MAX_LOCKOUT_LEVEL);
    LockoutState {
        failed_count: 0,
        locked_until: Some(now + pause),
        lockout_level: lockout_level + 1,
    }
}
