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

Do not manually run `cargo build`/`cargo test`/the e2e suite as a pre-commit check — the pre-commit hook already runs them, acting like a CI gate. It's fine to run tests during development to verify your own fix, but don't re-run the full suite right before `git commit` just to double-check.

The pre-commit hook runs the full e2e (Playwright) suite plus cargo checks, which regularly takes a few minutes — longer than the default 2-minute Bash tool timeout. A plain `git commit` will hit that timeout even though the hook itself is still running to completion in the background. Always pass an explicit `timeout` of at least 300000ms (5 minutes), and prefer 480000ms (8 minutes) to be safe, on the `git commit` Bash call to avoid wasting a turn on a false timeout.

Avoid using uncommon abbreviations when naming (e.g. `TimedRdParser` for "recursive descent" — spell it out as `TimedRecursiveDescentParser` instead). Widely understood abbreviations (e.g. `Ast`, `Id`, `Http`) are fine.

Test cases should not be inlined with the source code, they should live in separate files.

Never use tuple in new data structures, always use struct instead.

## Architecture documentation

The rendering pipeline layers, entry points, key types, and domain glossary are documented in `ARCHITECTURE.md`.

- When a layer's entry function signature or module path changes, **MUST update `ARCHITECTURE.md`** in the same commit.
- When a key type is added, removed, or renamed in any layer, **MUST update `ARCHITECTURE.md`** in the same commit.
- When a new domain term is introduced or an existing term is redefined, **MUST update the glossary in `ARCHITECTURE.md`** in the same commit.
