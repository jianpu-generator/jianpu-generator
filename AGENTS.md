## Syntax documentation

The `.jianpu` input syntax is documented in `syntax.md`.

- When a commit introduces or changes user-facing `.jianpu` syntax, **MUST update `syntax.md`** in the same commit.
- When a commit introduces or changes user-facing `.jianpu` syntax, **MUST update the `demo/` folder** in the same commit (add/update the example measure demonstrating the feature in the relevant `demo/NN-*.jianpu` file). Each file in `demo/` is a complete, standalone-valid `.jianpu` document (its own `# metadata`/`# parts`/`# score`) — the web editor opens them individually as a folder of demo files, so a fragment missing its own header would fail to render there even though it's never concatenated with the others.
- Syntax-affecting code lives under `src/parser/` and `src/desugar.rs`.

## Coding style

Prefer functional programming style:
- **TypeScript**: use the `remeda` library (`import * as R from 'remeda'`)
- **Rust**: use the `itertools` crate

## UI components

Prefer Radix UI primitives over DIY implementations for interactive controls (sliders, selects, dialogs, checkboxes, tooltips, etc.). Available packages: `@radix-ui/react-dialog`, `@radix-ui/react-select`, `@radix-ui/react-slider`, `@radix-ui/react-tooltip`, `@radix-ui/react-progress`. Install additional Radix packages as needed rather than rolling custom components.

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
  crate — `crates/live-share-worker` — to the wasm bundle `wrangler`
  expects, per `crates/live-share-worker/wrangler.toml`'s `[build] command`):
  install with `cargo install worker-build`. Only needed to actually build
  the deployable bundle (e.g. via `wrangler dev`/`deploy`, or running
  `worker-build` directly to sanity-check the wrangler config) — the
  pre-commit `live-share-worker-wasm-check` job doesn't need it, since it
  only runs `cargo check --target wasm32-unknown-unknown`, and skips with a
  message rather than failing if the `wasm32-unknown-unknown` target isn't
  installed (`rustup target add wasm32-unknown-unknown`).

## Cross-boundary invariants (Rust ↔ TS)

Data crossing the Rust/TS boundary (wasm bindings, serde JSON, worker `postMessage` protocol) can carry an invariant a simple type system could enforce — a finite set of string tags, a field name, a fixed-arity shape — but that's instead left as a bare `string`/`number`/generic object on one or both sides (e.g. `TagOut` in `crates/jianpu-wasm/src/svg_types.rs` becoming hand-typed `data-tag` strings re-embedded in ~7 TS files' `querySelector` calls, with no shared type), so a typo or rename compiles clean on both sides and only breaks at runtime; never add a new instance of this — represent such data as a tagged union (Rust `enum` with `#[serde(tag = "...")]` matched by an exhaustive TS `switch`/`never` check) or, failing that, a shared narrow type/constant every consumer references, tighten any existing instance you touch rather than adding another hand-typed occurrence, and ask the user first if the right enforcement mechanism isn't obvious.
