//! Pure outbox state machine for cloud file synchronization.

pub mod backoff;
pub mod codec;
pub mod enqueue;
pub mod merge;
pub mod scheduling;
pub mod summary;
pub mod transitions;
pub mod types;
