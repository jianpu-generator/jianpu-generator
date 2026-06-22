# Cheatsheet Feature — Design

## Overview

Add a `?` button to the app header that opens a modal cheatsheet — a sectioned table of `.jianpu` syntax rows, each showing a description, the syntax token, and a live-rendered SVG snippet. SVGs are rendered eagerly at app start via a dedicated WASM worker and read synchronously when the modal opens.

---

## Example Data

### Shared source of truth

All cheatsheet examples live in `cheatsheet-examples.toml` at the repo root. This file is the single source of truth consumed by both the Rust build-time validation test and the TypeScript UI.

### Structure

The file contains an array of sections, each with a title and a list of examples. Each example has a `kind` discriminant. `note`, `chord`, and `line` examples fit in TOML inline tables; `score` examples use `[[section.examples]]` array-of-tables entries because their `source` field requires multiline strings.

The structure below is illustrative (using `\n` for clarity); the actual file uses TOML multiline basic strings (`"""..."""`) for `source` values:

```toml
[[section]]
title = "Notes"
examples = [
  { kind = "note",  description = "Quarter note",      syntax = "1",    token = "1"    },
  { kind = "note",  description = "Rest",              syntax = "0",    token = "0"    },
  { kind = "note",  description = "Eighth note",       syntax = "1_",   token = "1_"   },
  { kind = "note",  description = "Sixteenth note",    syntax = "1=",   token = "1="   },
  { kind = "note",  description = "Dotted note",       syntax = "1.",   token = "1."   },
  { kind = "note",  description = "Dotted eighth",     syntax = "1_.",  token = "1_."  },
  { kind = "note",  description = "Octave up",         syntax = "1'",   token = "1'"   },
  { kind = "note",  description = "Two octaves up",    syntax = "1''",  token = "1''"  },
  { kind = "note",  description = "Octave down",       syntax = "1,",   token = "1,"   },
  { kind = "note",  description = "Two octaves down",  syntax = "1,,",  token = "1,,"  },
  { kind = "note",  description = "Extension",         syntax = "1 -",  token = "1 -"  },
]

[[section]]
title = "Chords"
examples = [
  { kind = "chord", description = "Major",             syntax = "1",    token = "1"    },
  { kind = "chord", description = "Minor",             syntax = "1m",   token = "1m"   },
  { kind = "chord", description = "Dominant 7th",      syntax = "17",   token = "17"   },
  { kind = "chord", description = "Major 7th",         syntax = "1M7",  token = "1M7"  },
  { kind = "chord", description = "Minor 7th",         syntax = "1m7",  token = "1m7"  },
  { kind = "chord", description = "Sharp",             syntax = "1#",   token = "1#"   },
  { kind = "chord", description = "Sharp minor 7th",   syntax = "1#m7", token = "1#m7" },
  { kind = "chord", description = "Flat",              syntax = "3b",   token = "3b"   },
  { kind = "chord", description = "Slash bass",        syntax = "1/5",  token = "1/5"  },
]

[[section]]
title = "Groups"
examples = [
  { kind = "line", description = "Slur",        syntax = "(1 2)",       notes_line = "(1 2) 0 0"       },
  { kind = "line", description = "Nested slur", syntax = "(3= (2_1_))", notes_line = "(3= (2_1_)) 0_" },
]

[[section]]
title = "Directives"
examples = [
  { kind = "score", description = "BPM / key / time", syntax = "bpm=120 key=C4 time=4/4", source = "[parts]\nmain = notes\n\n[score]\n(bpm=120 key=C4 time=4/4)\n1 0 0 0" },
  { kind = "score", description = "Section label",    syntax = 'label="Verse 1"',          source = "[parts]\nmain = notes\n\n[score]\n(label=\"Verse 1\")\n1 0 0 0"       },
  { kind = "line",  description = "Inline key change", syntax = "1=C4",                    notes_line = "1=C4 1 2 3"  },
  { kind = "line",  description = "Inline time change", syntax = "3/4",                    notes_line = "3/4 1 2 3"   },
  { kind = "line",  description = "Inline BPM change",  syntax = "bpm=92",                 notes_line = "bpm=92 1 2 3 4" },
]

[[section]]
title = "Lyrics"
examples = [
  { kind = "score", description = "Notes + lyrics",   syntax = "do re mi fa", source = "[parts]\nMelody = notes lyrics\n\n[score]\n1 2 3 4\ndo re mi fa" },
  { kind = "score", description = "Held syllable",    syntax = "你 -",         source = "[parts]\nMelody = notes lyrics\n\n[score]\n1 - 3 0\n你 - 好"     },
  { kind = "score", description = "No-lyrics marker", syntax = "_",            source = "[parts]\nMelody = notes lyrics\n\n[score]\n1 2 3 4\ndo re mi fa\n\n1 2 3 4\n_" },
]

[[section]]
title = "Ditto"
examples = [
  { kind = "score", description = "Ditto line", syntax = '"', source = "[parts]\nMelody = notes\n\n[score]\n1 2 3 4\n\n\"" },
]
```

### TypeScript type

```typescript
export type CheatsheetExample =
  | { kind: 'note';  description: string; syntax: string; token: string }
  | { kind: 'chord'; description: string; syntax: string; token: string }
  | { kind: 'line';  description: string; syntax: string; notesLine: string }
  | { kind: 'score'; description: string; syntax: string; source: string }

export interface CheatsheetSection {
  title: string
  examples: CheatsheetExample[]
}

export const cheatsheetSections: CheatsheetSection[]
```

---

## Rust/WASM Snippet Rendering

### Parser change

`[metadata]` becomes optional. When absent, the parser uses empty defaults (no title/author). Snippet mode already skips header rendering so these values are never used.

### Snippet pipeline flag

A `snippet: bool` parameter threads through `grid_layout::layout()`. When `true`:
- `make_header_rows()` is skipped
- Footer row is skipped
- Bar number and part label elements are skipped
- `usable_h` is not reduced by `header_h`
- After coordinate resolution, a tight viewBox is computed from content bounds using the existing page margin as padding
- `AbsolutePage.width_pt` / `height_pt` are set to the bounding box dimensions (not A4)

### Four WASM-exposed functions

Each passes `snippet = true` through the pipeline and returns a single SVG string:

```rust
#[wasm_bindgen] pub fn render_note_token_snippet(token: &str) -> Result<String, String>
#[wasm_bindgen] pub fn render_chord_token_snippet(token: &str) -> Result<String, String>
#[wasm_bindgen] pub fn render_notes_line_snippet(notes_line: &str) -> Result<String, String>
#[wasm_bindgen] pub fn render_parts_score_snippet(source: &str) -> Result<String, String>
```

**Internal boilerplate construction:**

| Function | Constructed source |
|---|---|
| `render_note_token_snippet("1_")` | `[parts]\nmain = notes\n[score]\n1_ 0_ 0 0` |
| `render_chord_token_snippet("1m")` | `[parts]\nmain = chord\n[score]\n1m - - -` |
| `render_notes_line_snippet("(1 2) 0 0")` | `[parts]\nmain = notes\n[score]\n(1 2) 0 0` (4/4 assumed) |
| `render_parts_score_snippet(source)` | parses directly; `[metadata]` optional |

All four functions return a single SVG string. Snippets always produce one tight-viewBox page.

**Boilerplate padding strategy for `render_note_token_snippet`:** append enough rests to not overshoot 4/4 (16 quarter-beats). The parser's shortfall-fill extends the trailing rest to complete the measure, so e.g. `1_ 0_ 0 0` (2+2+4+4=12 explicit) is fine — the last `0` is extended to 8 by the parser. Never overshoot.

### Build-time validation test

`src/cheatsheet_examples_test.rs` contains a `#[cfg(test)]` module with the validation tests. It must be registered in `src/lib.rs` as:

```rust
#[cfg(test)]
mod cheatsheet_examples_test;
```

The test reads `cheatsheet-examples.toml` via `include_str!("../cheatsheet-examples.toml")`, calls the appropriate render function for each example's `kind`, and asserts `Ok(svg)` where `svg` is non-empty. Any render failure fails `cargo test`.

---

## Web Architecture

### Module-level SVG singleton

`web/src/cheatsheetSvgs.ts`:
- Imports `cheatsheet-examples.toml` via `vite-plugin-toml` (new dev dependency)
- Creates a dedicated `Worker` (separate from the editor worker) at module load time
- Immediately sends all ~29 render requests using 4 new `WorkerRequest` variants
- Stores results in `Map<number, string>` (flat index → single SVG)
- Worker lives for the app lifetime; never terminated

**Flat index scheme:** examples are numbered 0–N across all sections in declaration order (section 0 example 0, section 0 example 1, …, section 1 example 0, …). The `id` field in each `WorkerRequest` is this flat index. The singleton map key and the worker request `id` use the same flat index.

The singleton is considered complete by the time the user can open the cheatsheet. No subscription or async handling in the dialog.

### Worker message types

Four new variants added to `WorkerRequest`:

```typescript
| { type: 'renderNoteTokenSnippet';  id: number; token: string     }
| { type: 'renderChordTokenSnippet'; id: number; token: string     }
| { type: 'renderNotesLineSnippet';  id: number; notesLine: string }
| { type: 'renderPartsScoreSnippet'; id: number; source: string    }
```

Each response uses a new `WorkerResponse` variant (not the existing `ok` variant, which carries `diagnostics` / `diagnosticViewZones` fields that snippets don't need):

```typescript
| { type: 'snippetOk'; id: number; svg: string }
```

### `AppHeader` component

`web/src/components/AppHeader.tsx`:
- Owns `const [cheatsheetOpen, setCheatsheetOpen] = useState(false)`
- Renders `<h1>`, subtitle span, `?` trigger button, and `<CheatsheetDialog>`
- `App.tsx` replaces the current inline `<header className="app-header">` block with `<AppHeader />`

### `CheatsheetDialog` component

`web/src/components/Cheatsheet.tsx`:
- Props: `open: boolean`, `onOpenChange: (open: boolean) => void`
- On open: reads the singleton map synchronously
- Renders 6 sections (Notes, Chords, Groups, Directives, Lyrics, Ditto), each with a heading and rows of `description | syntax | SVG`
- Uses Radix `Dialog.Root` in controlled mode

---

## Files Created / Modified

| File | Change |
|---|---|
| `cheatsheet-examples.toml` | New — shared example data |
| `src/parser/` | Make `[metadata]` optional |
| `src/grid_layout/layout.rs` | `snippet: bool` flag — skip header/footer/bar numbers/labels, tight viewBox |
| `src/serializer/mod.rs` | Use computed bounding box dimensions for `width`/`height`/`viewBox` |
| `src/lib.rs` | Four new `#[wasm_bindgen]` snippet functions |
| `src/cheatsheet_examples_test.rs` | New — build-time validation test |
| `web/package.json` + lockfile | Add `@radix-ui/react-dialog`, `vite-plugin-toml` |
| `web/vite.config.ts` | Register `vite-plugin-toml` |
| `web/src/worker/jianpu.worker.ts` | Four new `WorkerRequest` variants + handlers |
| `web/src/cheatsheetSvgs.ts` | New — module-level singleton |
| `web/src/components/AppHeader.tsx` | New — header with open state |
| `web/src/components/AppHeader.css` | New — header styles (extracted from App.css) |
| `web/src/components/Cheatsheet.tsx` | New — `CheatsheetDialog` component |
| `web/src/components/Cheatsheet.css` | New — modal styles |
| `web/src/App.tsx` | Replace inline header with `<AppHeader />` |
| `web/src/App.css` | Remove header styles now in `AppHeader.css` |
