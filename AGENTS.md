## Branching and PRs

**Never commit directly to `master`.** Always work on a feature branch and open a pull request. The full e2e suite is too large to run in the pre-commit hook, so it runs in CI (`.github/workflows/ci.yml`) on the PR instead — the PR is the gate, not the local commit.

### Before/after screenshots in PRs

When a PR changes rendered output (layout, spacing, rows, glyphs, colors — anything visible in the SVG/PDF or the web UI), its description **MUST include a `## Before / after` section** with a two-column `| Before | After |` table of images, so the reviewer can eyeball the change instead of reproducing it. Skip it for changes with no visual effect (parser errors, MIDI, refactors, CI, docs).

- Render both from the **same minimal `.jianpu` score** (ideally the one from the new test case), with the fix toggled off for "before" and on for "after". Show the score source in a code block above the table.
- Generate SVGs with `cargo run --features cli -- generate svg <file>`. Crop each to the relevant region by shrinking `width`/`height`/`viewBox` so the change is legible rather than a mostly-white A4 page.
- GitHub strips inline `<svg>` from descriptions and relative image paths don't resolve, so push the images to an orphan branch named `pr-<number>-assets` and link them via `https://raw.githubusercontent.com/<owner>/<repo>/pr-<number>-assets/<file>.svg`. Don't commit them into the PR's own branch. The assets branch can be deleted after merge.
- Add one sentence under the table saying what differs, and say if you haven't viewed the images yourself.

## Syntax documentation

The `.jianpu` input syntax is documented in `syntax.md`.

- When a commit introduces or changes user-facing `.jianpu` syntax, **MUST update `syntax.md`** in the same commit.
- When a commit introduces or changes user-facing `.jianpu` syntax, **MUST update the `demo/` folder** in the same commit (add/update the example measure demonstrating the feature in the relevant `demo/NN-*.jianpu` file). Each file in `demo/` is a complete, standalone-valid `.jianpu` document (its own `# metadata`/`# parts`/`# score`) — the web editor opens them individually as a folder of demo files, so a fragment missing its own header would fail to render there even though it's never concatenated with the others.
- Syntax-affecting code lives under `src/parser/` and `src/desugar.rs`.

## Coding style

Prefer functional programming style:
- **TypeScript**: use the `remeda` library (`import * as R from 'remeda'`)
- **Rust**: use the `itertools` crate

## Node scripts

Node scripts (`web/scripts/`, e2e mock/static servers, Playwright reporters, anything run with `node`) are TypeScript (`.ts`) run directly with `node` (native type stripping — no `tsx`/`ts-node`, so stick to erasable syntax and explicit `.ts` import extensions), never untyped `.mjs`/`.js`, and must be covered by `tsc -b` (the pre-commit `web-typecheck` job) — add any new script's path to the matching `web/tsconfig.*.json` `include`. Parse CLI arguments with `commander`, not hand-rolled `process.argv` slicing. The only exception is config a tool requires in a fixed JS format (e.g. `web/.pnpmfile.cjs`).

## UI components

Prefer Radix UI primitives over DIY implementations for interactive controls (sliders, selects, dialogs, checkboxes, tooltips, etc.). Available packages: `@radix-ui/react-dialog`, `@radix-ui/react-select`, `@radix-ui/react-slider`, `@radix-ui/react-tooltip`, `@radix-ui/react-progress`. Install additional Radix packages as needed rather than rolling custom components.

## Browser storage

`localStorage`/`sessionStorage` in the web app is only for:
- **transient relays** (e.g. the GitHub sign-in popup's PKCE verifier and auth-result handoff),
- **local files** (the `local` storage backend's file store),
- **per-device preferences** (e.g. storage backend choice, part toggles, the sign-in token).

Anything else, especially state that must agree across devices (e.g. whether a file is shared, which share belongs to which file), lives in the database and is derived from it, never mirrored into browser storage.

## Local dev

To run local dev, use `mprocs -c dekit.yaml`.

## Local tooling

- `sqruff` (SQL linter/formatter, used by the pre-commit `sqruff` job on
  `live-share-worker/migrations/**/*.sql` and
  `crates/live-share-worker/queries/**/*.sql` — see `.sqruff`): install with
  `cargo install sqruff --locked --version 0.29.3`. Pinned because
  `sqruff` 0.34.1 fails to compile from crates.io as of this writing (a
  `PathBuf == str` type error in `sqruff-cli-lib`), and newer releases
  require a rustc version ahead of this repo's. The pre-commit job skips
  with a message rather than failing if `sqruff` isn't on `PATH`.
- `worker-build` (Cloudflare's build tool that compiles a `workers-rs`
  crate — `crates/live-share-worker` — to the wasm bundle `cf`
  expects, per `crates/live-share-worker/wrangler.config.ts`'s `build.command`):
  install with `cargo install worker-build`. Only needed to actually build
  the deployable bundle (e.g. via `cf dev`/`deploy`, or running
  `worker-build` directly to sanity-check the cf config) — the
  pre-commit `live-share-worker-wasm-check` job doesn't need it, since it
  only runs `cargo check --target wasm32-unknown-unknown`, and skips with a
  message rather than failing if the `wasm32-unknown-unknown` target isn't
  installed (`rustup target add wasm32-unknown-unknown`).

## Cross-boundary invariants (Rust ↔ TS)

A cross-boundary invariant is any value, shape or rule that two sides must agree on. The two sides can be Rust and TS, or either of them and config/scripts. Examples:

- a string tag
- a field name
- an index space
- an id format
- a numeric limit
- a URL path or status code
- a SQL column
- an env var name
- a label format like `"N: Name"`

It applies on every channel:

- wasm/WIT exports and their jco `.d.ts`
- serde JSON and worker `postMessage` messages
- the live-share worker's HTTP API
- the D1 schema, including test scripts that query it
- deploy/build config (`cloudflare.config.ts`, workflows, `.env`)
- Pages Functions
- e2e selectors and mocks

**Never keep such an agreement by hand.** Drift must be a compile, typecheck or build error, or impossible by construction. To get there, use one of these:

- **Generated types:** the WIT/jco `.d.ts`, or the OpenAPI `schema.ts` from `handlers::routes()`. Use a WIT `variant`/`enum`/`record` rather than a bare `string`/`u32`/`list<u8>` whose meaning TS must know.
- **One shared data file** that both sides read at build time, e.g. `fonts/fonts.json`.
- **Logic on one side only.** If TS needs something Rust already computes (a part's name after hidden parts are filtered, a formatted label, which font measures which text, a section kind), make Rust return it. Don't re-derive it in TS.

These do **not** count as a fix. They narrow the hand-matching instead of removing it:

- "must match X" / "mirrors Y" comments
- a TS constant or `satisfies Record<…>` restating a Rust value
- a hand-typed interface or cast on a raw `fetch`/`.json()`
- a regex re-implementing a Rust lexer rule
- positional parameters whose meaning the caller must know
- a test asserting that two hand-written copies agree

Rules:

- Never add a new instance.
- Remove any existing instance you touch rather than adding another occurrence.
- If none of the options above fits (e.g. values fixed by an external deploy platform), stop and ask the user before settling for less.
- `TODO-cross-boundary-invariants.md` tracks the known instances and how each was fixed.
