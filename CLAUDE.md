# Syntax documentation

The `.jianpu` input syntax is documented in `syntax.md`.

- When a commit introduces or changes user-facing `.jianpu` syntax, **MUST update `syntax.md`** in the same commit.
- Syntax-affecting code lives under `src/parser/` and `src/desugar.rs`.


## Coding style
Prefer functional programming style and use the `itertools` library, unless the iterative version is simpler, shorter and easier to understand.


## How to generate the SVG?
Example:
```sh
cargo run -- generate svg simple.jianpu
```

## Committing

Commit message titles must always include a scope: `<type>(<scope>): <description>`, e.g. `feat(sequence): allow entries to omit parts for a single occurrence`. The scope identifies **where** the change lives (the module/directory/area touched, e.g. `sequence`, `grid-layout`, `midi`, `editor`), never **what** the change does or which feature it adds — that belongs in the description. When a change spans multiple modules in service of one feature, pick the scope by where the change is centered (the module doing the most work, or the one a reader would look in first), not by naming the feature itself. Never omit the scope, even for small or single-file changes. Don't restate the scope word in the description; the scope already says it.

Do not manually run `cargo build`/`cargo test` as a pre-commit check — the pre-commit hook already runs them, acting like a CI gate. The full e2e suite is not part of the hook; CI (`.github/workflows/ci.yml`) runs it on pull requests. It's fine to run tests during development to verify your own fix, but don't re-run the full suite right before `git commit` just to double-check.

The pre-commit hook runs cargo checks and the web typecheck, which can take a few minutes — longer than the default 2-minute Bash tool timeout. A plain `git commit` will hit that timeout even though the hook itself is still running to completion in the background. Always pass an explicit `timeout` of at least 300000ms (5 minutes), and prefer 480000ms (8 minutes) to be safe, on the `git commit` Bash call to avoid wasting a turn on a false timeout.

## Running e2e tests

When you run Playwright e2e tests yourself locally, run only the specific tests you need, with plain Playwright: `cd web && pnpm test:e2e --grep "..."`. Do **not** use the auto-retry script `pnpm test:e2e:resolve` locally — it exists for CI (`web/scripts/resolve-e2e-flakes.ts`), where the full suite runs and its known flaky tests are re-run until the failing set settles. A scoped local run is small enough to read directly, and a failure there should be investigated, not retried away. Stop any dev server you started once the run finishes.

Avoid using uncommon abbreviations when naming (e.g. `TimedRdParser` for "recursive descent" — spell it out as `TimedRecursiveDescentParser` instead). Widely understood abbreviations (e.g. `Ast`, `Id`, `Http`) are fine.

Test cases should not be inlined with the source code, they should live in separate files.

Never use tuple in new data structures, always use struct instead.

## Web frontend must stay dumb

Keep the TypeScript web frontend as dumb as possible; put logic in Rust (compiled to wasm for the worker). The web layer passes raw UI state (e.g. hidden/soloed sets) to Rust and renders what comes back. Don't derive effective state (solo resolution, track filtering) in TS hooks.

## Architecture documentation

The rendering pipeline layers, entry points, key types, and domain glossary are documented in `ARCHITECTURE.md`.

- When a layer's entry function signature or module path changes, **MUST update `ARCHITECTURE.md`** in the same commit.
- When a key type is added, removed, or renamed in any layer, **MUST update `ARCHITECTURE.md`** in the same commit.
- When a new domain term is introduced or an existing term is redefined, **MUST update the glossary in `ARCHITECTURE.md`** in the same commit.

## Debugging production

When asked why something happened in production (e.g. a user got signed out, or can't sign in), **query the `app_log` D1 table before guessing**. It holds 30 days of what the Synced Share worker did (see "App log (observability)" in `ARCHITECTURE.md`). Saved queries live in `crates/live-share-worker/queries/observability/` (`recent.sql`, `auth_rejections.sql`, `auth_by_version.sql`, `sign_ins_per_email.sql`). Run one against the production database from `crates/live-share-worker`:

```sh
pnpm exec wrangler d1 execute jianpu-live-share --remote --command="$(cat queries/observability/auth_rejections.sql)"
```

- Keep the `--command=` form: the saved queries start with a `-- ` comment, which a space-separated `--command` would read as a flag. `--file` returns no rows for a remote `SELECT`.
- This uses wrangler's stored login (`pnpm exec wrangler whoami`). `cf d1 query <database-id> --sql="..."` also works, but needs `CLOUDFLARE_API_TOKEN` or `cf auth login`.
- Against a local `cf dev` database, use `sqlite3 .wrangler/state/v3/d1/miniflare-D1DatabaseObject/<hash>.sqlite ".read queries/observability/recent.sql"`. `wrangler d1 execute --local` can't find the database, because the D1 binding lives in `cloudflare.config.ts`.
