//! Exponential backoff with jitter.

const BASE_MS: u64 = 2_000;
const CAP_MS: u64 = 60_000;

/// Delay before the next retry after `consecutive_failures` failures.
///
/// `exp = min(2000 * 2^(n-1), 60000)`, `half = exp / 2`, and the result is
/// `half + floor(jitter_unit.clamp(0, 1) * half)`. A count of zero is treated
/// as one failure.
pub fn backoff_ms(consecutive_failures: u32, jitter_unit: f64) -> u64 {
    let doublings = consecutive_failures.saturating_sub(1).min(16);
    let exp = (BASE_MS << doublings).min(CAP_MS);
    let half = exp / 2;
    let jitter = (jitter_unit.clamp(0.0, 1.0) * half as f64).floor() as u64;
    half + jitter
}
