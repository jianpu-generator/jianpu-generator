# Does live-share-worker use sqlx for compile-time query checking?

Discussion recap (2026-09-11) on whether `crates/live-share-worker` gets
`sqlx`-style compile-time SQL checking, and where the gaps are.

## Short answer

`sqlx` is a real dependency and it does compile and run — but only on the
host, for the build script and the test suite. It never ships in the
deployed Cloudflare Worker (wasm) binary, and it's not doing true
compile-time checking the way `sqlx::query!`/`query_file!` normally do.

## Why sqlx can't be a normal dependency here

The worker targets `wasm32-unknown-unknown`. `sqlx`'s `"sqlite"` feature
pulls in `libsqlite3-sys`, which compiles SQLite's C amalgamation via `cc`
— that C build assumes a hosted C environment (`stdio.h` etc.) that
`wasm32-unknown-unknown` doesn't provide. Confirmed by actually trying it:
`cargo check --target wasm32-unknown-unknown` fails inside
`libsqlite3-sys`'s build script. Not a version/feature-flag issue — sqlx's
SQLite row/type decoding depends on the `sqlx-sqlite` driver crate
regardless of which macros you use.

So the real query path (`src/db.rs`) uses `include_str!` to embed each
`.sql` file under `queries/`, and runs it through Cloudflare's own
`D1Database::prepare()`/`.bind()` JS API at runtime. Zero `sqlx`
involvement there.

## Where sqlx actually lives

- **`[build-dependencies]`** (`build.rs`, host-only): applies every
  migration in `migrations/*.sql` to a fresh local SQLite file
  (`shadow.sqlite`, gitignored), so there's a real schema to check
  queries against.
- **`[dev-dependencies]`** (`tests/query_syntax.rs`): opens that same
  `shadow.sqlite` and calls `sqlx`'s `.prepare(query)` — not execute — on
  every file under `queries/`. SQLite has to resolve every table/column
  name to compile the statement, so this catches typos like a renamed
  column or a query referencing a table that no longer exists.

## How the check actually catches a typo

1. `build.rs` runs before any compile (`cargo build`/`check`/`test`/`run`
   all trigger it) and rebuilds `shadow.sqlite` from the migrations.
2. The test opens that file and calls `prepare()` on each query string.
3. `prepare()` doesn't run the query, but SQLite still must resolve every
   table/column name to build an execution plan — so a typo'd column
   surfaces as e.g. `no such column: ttoken_hash`.
4. The test maps that into `sqlx::Error::Protocol(...)` and fails.

Important nuance: this is a **test-time** check, not a **compile-time**
one. `cargo build`/`cargo check` succeed even with a broken query — only
`cargo test` catches it. Real `sqlx::query_file!` would instead fail
`cargo build` itself, because it's a proc-macro that runs during
compilation. This repo's version is described in the code comments as
"the pragmatic substitute," not a full equivalent.

## Idea raised: move the check into build.rs

Feasible. `build.rs` already builds the shadow DB — add the same
`prepare()` loop there and panic/exit non-zero on failure instead of
returning `Err` from a `#[tokio::test]`. Since `build.rs` runs before
*any* compile, this would be a **stronger** gate than today (fires even
for someone who only runs `cargo check`, without ever running tests).

Two things to get right if we do this:
- Add `cargo:rerun-if-changed=<queries dir>` — `build.rs` currently only
  watches `migrations/` and itself, not `queries/*.sql`.
- You lose the nicely-named, per-query test failure output in favor of a
  build script's raw stderr dump — a minor ergonomics trade, not a
  blocker.

Not yet decided whether to make this change — just confirmed it's doable.

## What this check does *not* catch: bound-parameter types

Even with the above move, `prepare()` without binding only proves two
things: the SQL parses, and every table/column name it references
exists. It does **not** check:

- how many `?` placeholders a query has vs. how many values `src/db.rs`
  actually binds at the call site,
- the order values are bound in,
- or type compatibility between what's bound and what the column expects.

That's exactly the part real `sqlx::query!`/`query_file!` normally
provides — those macros inspect the prepared statement's parameter
metadata *and* cross-reference it against the Rust call site at compile
time. It can't happen here because the actual binding happens against
D1's JS API (`JsValue`s passed to `D1Database::prepare().bind(...)`) at
runtime in the wasm build — a completely different, untyped code path
that the SQLite-side `prepare()` check never sees.

Practical consequence: swapping the order of two bound parameters (or
binding the wrong count) compiles fine, passes `query_syntax`, and only
surfaces as a runtime error against real D1 — or worse, silently wrong
data if the swapped values happen to coerce to plausible types.

Closing that gap for real would mean hand-writing a check that parses
each query's `?` placeholders and cross-references it against the
corresponding `.bind()` call in `src/db.rs` — there's no off-the-shelf
macro bridging "SQLite-side compile-time check" to "D1 JS API runtime
call" the way `sqlx` bridges its own macro to its own driver. Not
attempted; just flagging the gap for whoever picks this up next.

## Also considered: swapping sqlx for something lighter (e.g. rusqlite)

Wouldn't buy much. The C-compile dependency (`libsqlite3-sys`) only
matters for the wasm target, and this usage is build/dev-only (host
target) either way — so swapping drivers trades one SQLite binding for
another without eliminating anything. Not pursued.

## Relevant files

- `crates/live-share-worker/Cargo.toml` — the `[build-dependencies]` /
  `[dev-dependencies]` `sqlx` entries, with the long reasoning comment.
- `crates/live-share-worker/build.rs` — shadow DB creation/migration.
- `crates/live-share-worker/tests/query_syntax.rs` — the `prepare()`-based
  syntax check.
- `crates/live-share-worker/src/db.rs` — the real runtime query path
  (`include_str!` + D1 `prepare()`/`.bind()`), no sqlx.
