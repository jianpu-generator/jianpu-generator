//! The rules of email + password accounts, D1- and WebCrypto-free so
//! `tests/auth_*.rs` can unit-test them directly: what counts as an email and
//! an acceptable password (`email`, `password`), how a password hash is
//! stored and compared (`password`), when repeated wrong passwords lock an
//! email out (`lockout`), and how long a session lasts (`session`).
//!
//! The parts that need the Workers runtime -- deriving PBKDF2 bits and
//! drawing random bytes through WebCrypto (`crate::crypto`), and the D1
//! reads and writes (`crate::db`) -- call into these. The routes themselves
//! are `crate::handlers::auth`; resolving a session token on every other
//! route is `crate::identity`.

pub mod email;
pub mod lockout;
pub mod password;
pub mod session;
