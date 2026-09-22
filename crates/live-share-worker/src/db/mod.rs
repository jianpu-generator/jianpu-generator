//! Raw D1 queries against the `docs` / `users` / `user_identities` /
//! `oauth_sessions` / `files` tables, split by table into `docs`, `identity`,
//! and `files` submodules (re-exported here so every call site keeps using
//! the flat `db::function_name` form regardless of which table it hits).
//! Each query lives in its own file under `queries/`, loaded via
//! `include_str!` for the actual `D1Database::prepare()` calls -- these
//! submodules are the only place those files are read.
//!
//! Compile-time query checking via `sqlx::query_file!` (as originally
//! sketched in `TODO-synced-share-rust-d1-migration.md` §2) isn't possible
//! in this crate's own wasm-targeted compile -- see the long comment on the
//! `[build-dependencies] sqlx` entry in `Cargo.toml`. The pragmatic
//! substitute used instead: `tests/query_syntax.rs`, a native-only test
//! that loads these same query files (unmodified, via the identical
//! `include_str!` paths) and runs them through `sqlx`'s statement
//! preparation (not execution) against the shadow SQLite database built by
//! `build.rs`, to catch a broken/typo'd query before it would otherwise
//! only surface at real D1 runtime. This is weaker than `query_file!`'s
//! compile-time type-checking -- it doesn't check bound-parameter types or
//! result-column names against Rust types, and any D1-vs-SQLite dialect
//! drift is still uncaught -- but it's an honest, reasonable substitute
//! given the constraint above, not a silent downgrade.

mod docs;
mod files;
mod identity;

pub(crate) use docs::*;
pub(crate) use files::*;
pub(crate) use identity::*;
