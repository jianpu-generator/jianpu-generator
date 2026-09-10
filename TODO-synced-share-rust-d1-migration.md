# Synced Share: Rust + D1 migration, GitHub-required ownership

Context: `live-share-worker` is currently TypeScript + Workers KV, with
anonymous (no-login) share ownership via a device-secret-derived
`ownerToken`. Decided in discussion: rewrite the worker in Rust
(`workers-rs`), move storage from KV to D1, and make **GitHub sign-in a
hard requirement for the Synced Share feature** — there is no longer an
anonymous ownership path. Share ownership is always a verified GitHub user
id; the device-secret/`ownerToken` mechanism is retired entirely, not kept
as a fallback.

Diesel is **not** used anywhere in this plan — it cannot reach D1 (no driver
target exists for D1's binding-based access) without writing a full custom
Diesel backend, which is out of scope. SQLx is used only in the
raw-SQL + local-shadow-SQLite-check pattern discussed, not as a live-D1 driver.

No backward compatibility is required anywhere in this plan (confirmed): no
data migration from the current KV store, no protocol byte-compatibility
requirement. There is currently only one live share in production; it can
simply be recreated after cutover if lost. Share links will look different
(new format, see §1) and old links simply stop working — that's accepted.

## 0. Decisions (resolved)

- [x] **GitHub sign-in is required for Synced Share to work at all.** There
      is no anonymous/no-login path any more. Creating a share requires a
      verified GitHub identity; the device-secret `ownerToken` scheme
      (`getOrCreateDeviceSecret`, `deriveSyncedShareIdentity` in
      `web/src/syncedShareUrl.ts`) is deleted, not kept as a fallback.
- [x] **Ownership = verified GitHub user id, permanently, from creation.**
      A share is created already bound to the creator's verified GitHub id
      (via `GET /user`, using a token from the dedicated Synced Share sign-in
      connection — see below). There is no "unpinned" state to reason about,
      and no device-token/GitHub precedence question, since the device token
      no longer exists in this system.
- [x] **Credential conflicts are rejected outright, never guessed at.** A
      write whose verified GitHub id doesn't match the share's
      `owner_github_id`, or whose GitHub verification fails, is rejected.
      No fallback of any kind.
- [x] **Fail closed, with one retry first.** On a GitHub verification
      failure (expired/invalid token, non-2xx from GitHub, timeout), retry
      once; if the retry also fails, reject the write. The failure must be
      reported to the UI in a form the user can screenshot (explicit error
      message with failure reason and timestamp), not a silent failure or
      generic toast.
- [x] Cache **a hash of the GitHub OAuth token**, never the raw token, in
      the verification-cache table.
- [x] **Auth and storage are separate concerns — use separate GitHub
      connections, not the shared storage-backend token.** The existing
      `githubAuth.ts` connection is scoped for the storage backend's
      Contents API access (broad, e.g. `repo` write) and was opt-in for a
      niche feature; Synced Share is now mandatory for a core, everyone-uses-it
      feature, so forcing every Synced Share user through that broad-scope
      consent (and sending that high-privilege token on every sync write)
      would be a real over-ask and a larger leak blast-radius than
      necessary. Instead: a **dedicated, minimally-scoped "sign in with
      GitHub" connection** used only for identity verification (no more
      scope than `GET /user` needs), independent of whether the user ever
      connects the storage backend. Implementation-wise this can reuse the
      *same registered GitHub OAuth App* (`client_id`), just requesting a
      different, minimal `scope` for this flow and storing its token
      separately from the storage-backend token — a second registered app
      is not required unless full isolation (independent rate limits/
      revocation) is later wanted.
- [x] **This new sign-in uses standard authorization-code + PKCE
      (redirect-based), not device flow** — decided over reusing the
      existing `@octokit/auth-oauth-device` pattern because sign-in is now
      mandatory/core rather than an opt-in power-user feature, and
      redirect-based "Sign in with GitHub" (one click, no manual code entry)
      is meaningfully lower-friction for something everyone must do. Use
      `oauth4webapi` for this — a small, spec-compliant, dependency-free
      OAuth2/OIDC client built on Fetch + Web Crypto (no Node-specific
      APIs), so the same library runs unmodified in the browser (to start
      the redirect + PKCE challenge) and in the Worker (to perform the
      token exchange, since Workers can hold the client secret as a secret
      binding — established earlier that a Worker is a real server for this
      purpose). Its docs explicitly cover GitHub as a plain-OAuth2
      (non-OIDC) example provider.
- [x] **No cleanup job for the verification-cache table.** Accept unbounded
      row growth at this project's scale; do not build TTL sweeping/cron
      cleanup.
- [x] **The create-share endpoint's abuse protection is GitHub verification
      itself** — since creating a share now requires a verified GitHub
      identity (not an open/anonymous call), no separate rate-limiting
      layer is needed for this reason alone.
- [x] **Testing**: mock the GitHub verification call in unit tests (inject a
      fake GitHub client); exercise the actual Worker/D1 integration via
      `wrangler dev`, not a from-scratch simulation of the Workers runtime.
- [x] **Reading a share stays fully anonymous.** GitHub is required only to
      create/own a share (write side). The unguessable `shareId` in the link
      remains the sole credential needed to view — no login for viewers,
      matching the feature's original point (a link you can send to anyone).
- [x] **No seamless OAuth-then-continue flow.** "Start sync" stays enabled
      even when GitHub isn't connected; clicking it while unconnected
      triggers the GitHub OAuth prompt, but does **not** automatically
      proceed to creating the share afterward — the user must click "start
      sync" again once connected. If the OAuth prompt is abandoned/expires,
      the UI just returns to the idle state (no share created, no dangling
      state).
- [x] **Errors are maximally verbose, with one hard exception.** On any
      failure (verification failure, retries exhausted, etc.) show the full
      error context — reason, timestamps, GitHub's raw status/error body,
      internal state — even if it looks like a stack trace, so it's directly
      useful when screenshotted for debugging. The one thing that must
      always be redacted regardless: the GitHub OAuth token and its stored
      hash never appear in any error surfaced to the UI.
- [x] **Error presentation**: a full-screen dialog/modal (not a toast or
      inline banner), containing the full verbose error context (per above)
      plus a message directing the user to screenshot it and file a GitHub
      issue, with a direct link:
      `https://github.com/jianpu-generator/jianpu-generator/issues/new`.
- [x] **Retry/backoff**: use an executor-agnostic backoff crate (e.g. the
      `backoff` crate's `future` module, which awaits a supplied future
      rather than owning its own tokio-based timer — required since Workers
      run on `wasm-bindgen-futures`, not tokio); 2 retries, exponential
      delay, capped around ~2s total wait before failing closed.

## Task breakdown (commit-sized, in dependency order)

Each row is meant to be one commit/PR. Scope follows this repo's commit
convention (`<type>(<scope>): <description>`); the scope shown is a
suggestion, pick the module doing the most work if a task ends up touching
more than expected. Items are cross-referenced back to the numbered
sections below (`§1`, `§4`, etc.) — do the detailed work there, check items
off in both places.

- [x] **1. Schema migrations** — `.sql` files for `users` /
      `user_identities` / `oauth_sessions` / `docs` (§1). Scope: `schema`.
      Depends on: nothing.
- [x] **2. Rust crate scaffold** — new `live-share-worker` crate,
      `wasm32-unknown-unknown` target, `worker` + `sqlx` deps, shadow-SQLite
      build step (§2). Scope: `live-share-worker`. Depends on: 1.
- [x] **3. `sqruff` pre-commit** — lint/format gate scoped to `queries/` and
      migration `.sql` files (§3). Scope: `lint`. Depends on: 1.
- [x] **4. Port worker logic to Rust** — `resolveRole`/`doc`/`index`/
      `protocol` against the new schema, with identity resolution stubbed
      (no real GitHub verification yet); update `ARCHITECTURE.md` in the
      **same commit** per this repo's docs rule (§4, §5). Scope:
      `live-share-worker`. Depends on: 2.
- [ ] **5. Wrangler + CI wiring** — `wrangler.toml` D1 binding and
      `worker-build`, CI/pre-commit build of the new crate for
      `wasm32-unknown-unknown` (§4). Scope: `live-share-worker`. Depends
      on: 4.
- [ ] **6. Dedicated GitHub sign-in plumbing** — `IdentityProvider` trait +
      `GithubIdentityProvider`, PKCE module (`oauth4webapi`), Worker
      callback route + client-secret binding (§6). Scope:
      `synced-share-auth`. Depends on: 2.
- [ ] **7. Verification + caching in the worker** — hashed-token lookup
      against `oauth_sessions`, backoff-wrapped GitHub call on miss/stale,
      create-on-first-sight `user_identities` row, wired into
      `resolveRole`-equivalent ownership check (§6). Scope:
      `synced-share-auth`. Depends on: 4, 6.
- [ ] **8. Client sign-in + owner UI** — popup OAuth flow, "Sync" popover
      when disconnected, "Synced as @username" identity chip, send the
      token on every write (mockup Screens 1–3). Scope: `synced-share-ui`.
      Depends on: 6.
- [ ] **9. Client error dialog** — full-screen verbose failure dialog with
      the one hard redaction (token/hash), "file a GitHub issue" link
      (mockup Screen 4). Scope: `synced-share-ui`. Depends on: 7, 8.
- [ ] **10. Viewer attribution + response scrubbing** — "Shared by
      @username" on `SyncedShareBanner` (mockup Screen 5), confirm the
      public doc response never leaks token/hash/internal ids (§6). Scope:
      `synced-share-ui`. Depends on: 7.
- [ ] **11. Retire the old path** — delete
      `getOrCreateDeviceSecret`/`deriveSyncedShareIdentity`/
      `SyncedShareIdentity` and the old TypeScript worker source; update
      Playwright scenarios for the new create-share flow, run via
      `pnpm test:e2e:resolve` ([[feedback_use_e2e_resolve_script]]) (§4,
      §6). Scope: `synced-share`. Depends on: 8, 9, 10 verified working.
- [ ] **12. Rollout** — `wrangler d1 create --location apac`, decide
      staging-vs-direct-to-prod cutover (§7). Scope: `live-share-worker`.
      Depends on: 11.

Mockup reference for 8–10: `synced-share-screens` artifact (5 screens:
sign-in prompt, OAuth popup, synced/owned state, error dialog, viewer
banner).

## 1. Schema design and shareId generation

- [ ] **`shareId` keeps its current format/length/charset** (whatever
      `deriveSyncedShareIdentity` currently produces), but is now generated
      server-side at share-creation time (after GitHub verification)
      instead of derived offline from a device secret.
- [ ] **Collision handling: append exactly one extra character** from the
      same charset if the generated id collides with an existing row (checked
      against D1), rather than regenerating from scratch or looping through
      many attempts. (Collision is already astronomically unlikely; this is
      belt-and-suspenders, not a retry loop.)
- [ ] Client persists the returned `shareId` locally (keyed by file) so
      stopping/resuming sync on the same file reuses the same link.
- [x] **Schema is provider-agnostic (future-proof for more than GitHub)**,
      per discussion — ownership points at an internal user, not a raw
      GitHub id:
  ```sql
  CREATE TABLE users (
      id TEXT PRIMARY KEY,              -- unguessable random id, same style
                                         -- as share_id — not sequential
      created_at INTEGER NOT NULL
  );

  CREATE TABLE user_identities (
      provider TEXT NOT NULL,           -- 'github' today; future: others
      provider_user_id TEXT NOT NULL,   -- TEXT: not every provider's id is numeric
      user_id TEXT NOT NULL REFERENCES users(id),
      login TEXT,                       -- best-effort cached display name
      linked_at INTEGER NOT NULL,
      PRIMARY KEY (provider, provider_user_id)
  );
  -- index on user_id: reverse lookup, "this user's linked identities"

  CREATE TABLE oauth_sessions (
      token_hash TEXT PRIMARY KEY,      -- hash of the *Synced Share sign-in*
                                         -- token specifically (never the
                                         -- storage-backend token — see §0)
      provider TEXT NOT NULL,
      provider_user_id TEXT NOT NULL,
      verified_at INTEGER NOT NULL      -- ~1hr TTL checked in app code
  );

  CREATE TABLE docs (
      share_id TEXT PRIMARY KEY,
      owner_user_id TEXT NOT NULL REFERENCES users(id),
      filename TEXT NOT NULL,
      content TEXT NOT NULL,
      revision INTEGER NOT NULL DEFAULT 0,
      ended INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
  );
  ```
  Scope note: this is schema/verification future-proofing only — no
  account-linking UI, no merge-accounts tooling, no app-wide session concept
  is being built now. A `user_identities` row is created automatically the
  first time a given `(provider, provider_user_id)` is verified; the same
  person authenticating via a second provider later would get a second,
  separate `users` row until an actual linking feature is built (out of
  scope here).
- [ ] Open items carried over, still to decide during implementation:
      whether `revision` still acts as an optimistic-concurrency guard on
      writes (it did in the old `StoredDoc`), whether `login` is worth
      caching for "shared by @username" UI, and whether `owner_user_id`
      needs an index for a possible future "my shares" list.
- [x] Write the schema as plain `.sql` migration files (shared source for
      both real D1 via `wrangler d1 migrations apply` and the local shadow
      SQLite used for SQLx compile-time checks).

## 2. Set up Rust + SQLx + shadow SQLite (no Diesel)

- [x] Add `live-share-worker` as a new crate in the Rust workspace, target
      `wasm32-unknown-unknown`, depending on `worker` (Cloudflare's crate)
      and `sqlx` (SQLite feature only, for compile-time query checking).
      Lives at `crates/live-share-worker/` (see task 2's naming note: the
      top-level `live-share-worker/` dir still holds the old TS worker).
      `sqlx` ended up scoped to `[build-dependencies]` rather than a normal
      dependency — see the comment on that entry in
      `crates/live-share-worker/Cargo.toml` for why (its "sqlite" feature
      pulls in `libsqlite3-sys`, whose C build doesn't compile for
      `wasm32-unknown-unknown`, confirmed by trying it directly).
- [x] Set up local shadow `.sqlite` file, populated from the same migration
      files as §1, for `sqlx::query_file!` to check against at build time.
      (`crates/live-share-worker/shadow.sqlite`, gitignored; no
      `query_file!` calls exist yet since there are no real queries until
      task 4 — this only proves the DB gets created/migrated correctly.)
- [x] Write one `.sql` file per query under a `queries/` dir; each query
      read once via `query_file!` (compile-time check) and again via
      `include_str!` for the actual `D1Database::prepare()` call at
      runtime, so both share one source of truth. (`query_file!` itself
      isn't usable in this crate's wasm-targeted compile -- see the
      scaffold's own comment on this; landed instead as: `src/db.rs` reads
      each `queries/*.sql` file via `include_str!` for real
      `D1Database::prepare()` calls, and `tests/query_syntax.rs` --
      native-only, `sqlx` as a dev-dependency -- reads the same files to
      *prepare* (not execute) them against the shadow SQLite database as a
      substitute syntax check. See `ARCHITECTURE.md`'s "D1 query checking"
      section for what this does and doesn't catch.)
- [x] Wire up whatever build-order step (build script or documented dev
      command) ensures the shadow DB is migrated before `cargo build`/`cargo
      check` runs, so the compile-time query check has something to check
      against. (`crates/live-share-worker/build.rs`, using `sqlx` +
      `tokio` as build-dependencies, applies
      `live-share-worker/migrations/*.sql` in order via `sqlx::migrate`.)
- [ ] Add integration tests run against `wrangler dev` (per §0 testing
      decision) as a backstop for D1-vs-SQLite dialect differences the
      shadow-SQLite check can't catch.

## 3. Set up `sqruff` in pre-commit

- [x] Add `sqruff` (Rust-native SQL linter/formatter) as a pre-commit step
      in the lefthook config, scoped to the new `queries/`/migrations `.sql`
      files. Installed via `cargo install sqruff --locked --version 0.29.3`
      (documented in `AGENTS.md`) — pinned because 0.34.1 fails to compile
      from crates.io (a `PathBuf == str` type error in `sqruff-cli-lib`)
      and newer releases need a rustc ahead of this repo's; the lefthook
      job (`sqruff` in the parallel validation group) skips with a message
      instead of failing the commit if the binary isn't on `PATH`. Config
      lives in `.sqruff` at repo root, dialect `sqlite`, with `LT01`/`LT02`
      excluded and a comment explaining why (they fight this repo's
      intentional no-space-before-paren and column-aligned trailing
      comment conventions, not SQLite dialect issues). Runs `sqruff lint`
      only, not `fix` — `fix` reflows the aligned trailing comments in
      `0001_init.sql` in a way not worth auto-applying on every commit.
- [x] Spot-check `sqruff`'s SQLite dialect handling against the actual query
      files before relying on it as a gate (flagged earlier as newer/less
      battle-tested than `sqlfluff`). Ran `sqruff lint`/`fix` directly
      against `live-share-worker/migrations/0001_init.sql`: it parses
      standard SQLite `CREATE TABLE`/`CREATE INDEX` syntax with no
      dialect-related false positives; all findings were generic style
      rules (indent, spacing, line length), not dialect bugs. Shortened one
      over-80-char comment line in that file to satisfy `LT05`.
- [x] Confirm ordering: per project convention, keep this fast enough to
      run before/alongside the fast checks, not mixed into the slow e2e
      stage ([[feedback_lefthook_e2e_ordering]]). Added as `sqruff` in the
      existing `parallel: true` fast-validation group in `lefthook.yml`
      (alongside `dead-pub-check` etc.), scoped by `glob` to just the SQL
      paths so it's a no-op on unrelated commits — not mixed into the
      `e2e-tests` stage.

## 4. Migrate TS worker logic to Rust

- [x] Port `resolveRole.ts`, `doc.ts`, `index.ts`, `protocol.ts` to Rust,
      targeting the new `docs` table from §1 directly, and rewritten around
      GitHub-required ownership (no device-token code path to port at all).
      (`crates/live-share-worker/src/{resolve_role,doc,handlers,protocol}.rs`;
      identity resolution stubbed per §0/§6 -- `src/identity/stub.rs`, not
      real GitHub verification, which is task 6/7.)
- [x] Add the new "create share" endpoint from §1, gated on GitHub
      verification; update `web/src/syncedShareUrl.ts` (device-secret logic
      deleted per §0), `useSyncedShareOwner.ts`, `useSyncedShareViewer.ts`,
      and `web/src/syncedShare/protocol.ts` for the new flow. No
      byte-compatibility constraint with the old protocol (per §0).
      (`POST /shares` added in `crates/live-share-worker/src/handlers.rs`,
      gated on the *stub* identity resolver for now -- real GitHub
      verification is task 6/7. The `web/` changes in this bullet --
      `syncedShareUrl.ts`, `useSyncedShareOwner.ts`,
      `useSyncedShareViewer.ts`, `web/src/syncedShare/protocol.ts` -- are
      explicitly out of scope for task 4, tasks 8-11.)
- [ ] Update `wrangler.toml`: build command pointing at `worker-build`
      (or equivalent), D1 binding instead of the KV namespace binding.
- [ ] Update CI/pre-commit to build the new crate for `wasm32-unknown-unknown`
      as part of the existing `cargo build`/`cargo test` gate.
- [ ] Run the existing Playwright e2e suite against the new worker via
      `pnpm test:e2e:resolve` ([[feedback_use_e2e_resolve_script]]), updating
      scenarios for the new GitHub-required create-share flow.
- [ ] Remove the old TypeScript worker source once parity is verified.

## 5. Update project docs (required by this repo's rules, same commit as §4)

- [x] Update `ARCHITECTURE.md`: new entry point/module paths for the
      Rust worker, any key types added/removed/renamed (e.g. `StoredDoc`
      → whatever the Rust struct is named), the KV→D1 terminology change,
      the GitHub-required create-share flow, and the retirement of the
      device-secret/anonymous-ownership concept, in the domain glossary as
      needed.

## 6. Implement GitHub-required ownership

- [ ] Build the new dedicated, minimally-scoped "sign in with GitHub"
      connection for Synced Share identity (per §0), using `oauth4webapi`
      for authorization-code + PKCE (redirect-based) — a new module
      distinct from `githubAuth.ts`'s device-flow code, storing its own
      token independently from the storage-backend token. Add a Worker
      route for the redirect callback + token exchange (holding the client
      secret as a Worker secret binding). Define the `IdentityProvider`
      trait discussed, with one `GithubIdentityProvider` implementation for
      now.
- [ ] Client (`useSyncedShareOwner.ts`): "start sync" stays enabled always;
      if this Synced Share GitHub connection isn't present, clicking it
      starts the redirect + PKCE sign-in flow but does not auto-continue
      afterward (per §0 — user clicks "start sync" again once connected).
      Send this connection's token with every write. No device-token code
      path exists any more.
- [ ] Worker: on every write, check `oauth_sessions` for a fresh verified
      identity (hashed-token lookup); on miss/stale, call the
      `IdentityProvider` (GitHub's `GET /user` today) to resolve
      `(provider, provider_user_id)`, refresh the cache, and look up (or
      create-on-first-sight) the matching `user_identities` row to get
      `user_id`. On failure, retry via the backoff policy (§0), then reject
      (fail closed) with a specific, UI-surfaced error. Never trust a
      client-asserted identity directly.
- [ ] `resolveRole`-equivalent logic: owner = resolved `user_id` matches
      `docs.owner_user_id` exactly. Any mismatch or verification failure
      (after retries) is rejected outright. No other cases exist.
- [ ] Ensure the public doc response (`toPublicDoc`-equivalent) never leaks
      the raw token, its hash, or internal `user_id`/`user_identities` rows
      beyond what the UI actually needs (e.g. a cached `login` for display).
- [ ] Delete `getOrCreateDeviceSecret`/`deriveSyncedShareIdentity` and the
      `SyncedShareIdentity` type from `web/src/syncedShareUrl.ts` — no
      longer used anywhere.

## 7. Rollout

- [ ] Create the D1 database with `wrangler d1 create --location apac` —
      the nearest available location hint to Kuala Lumpur (D1 location
      hints are broad regions — `wnam`/`enam`/`sam`/`weur`/`eeur`/`apac`/`oc`
      — not city-level, so `apac` is the closest available choice, not a
      guarantee of KL-adjacency). Note the latency tradeoff for
      geographically distant users in the rollout notes, not as a launch
      blocker.
- [ ] Decide staging vs. direct-to-prod cutover for the worker rewrite.
- [ ] No rollback/compat plan needed (per §0) — direct cutover, old worker
      retired once the new one is verified. Existing share link(s) will stop
      working and can be recreated.
