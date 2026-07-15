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
