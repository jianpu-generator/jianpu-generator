# File Split Plan — max 500 lines

Generated: 2026-06-18

## Files to split (10 total)

| File | Current | Extractions | Resulting size |
|---|---|---|---|
| `src/midi/mod.rs` | 504 | `midi_notes.rs` | ~455 |
| `src/grouper/tests.rs` | 520 | `tests_lyrics.rs` | ~420 |
| `web/src/App.css` | 505 | `file-tab-bar.css`, `preview.css` | ~137 |
| `src/grid_layout/tests.rs` | 566 | `tests_layout.rs` | ~271 |
| `src/grouper/mod.rs` | 582 | `directive_grouper.rs`, `lyrics_pairing.rs` | ~372 |
| `web/src/hooks/useJianpuWorker.ts` | 587 | `workerHelpers.ts` | ~481 |
| `src/compiler/mod.rs` | 600 | `beam.rs`, `part_slice.rs` | ~155 |
| `src/lib.rs` | 578 | `filters.rs`, `measure_spans.rs`, `split_track.rs` | ~285 |
| `src/grid_layout/layout.rs` | 593 | `layout_decoration.rs` + move 3 fns into existing `expand.rs` | ~360 |
| `src/parser/score/interleaved_parser.rs` | 596 | `interleaved_column_lines.rs`, `interleaved_accumulators.rs` | ~288 |

---

## Detailed splits

### 1. `src/midi/mod.rs` → `midi_notes.rs`

**Extract to `src/midi/midi_notes.rs`** (lines 459–503 of current file):
- `note_name_to_semitone`
- `pitch_to_scale_offset`
- `accidental_offset` → `pub(super)`
- `resolve_midi_note` → `pub(super)`
- `duration_to_ticks` → `pub(super)`

**`mod.rs` changes:**
- Add `mod midi_notes;`
- Add `use midi_notes::{accidental_offset, duration_to_ticks, resolve_midi_note};`
- Remove the 5 extracted functions
- Keep existing `pub(crate)` re-exports via the `use` above (tests.rs uses `use super::*`)

---

### 2. `src/grouper/tests.rs` → `tests_lyrics.rs`

**Extract to `src/grouper/tests_lyrics.rs`** (lines 420–520 of current file):
- `lyrics_overflow_recovers_with_error_on_measure`
- `lyrics_underflow_recovers_with_error_on_measure`
- `lyrics_underflow_error_span_covers_lyrics_line_not_notes`
- `measures_without_lyrics_underflow_have_no_errors`
- `cross_measure_tie_closing_note_does_not_consume_syllable`

`tests_lyrics.rs` needs its own:
- `use super::*;` (gives access to grouper private items — same as tests.rs)
- `use crate::parser;`
- Local duplicate of `parse_and_group` helper (3 lines)

**`grouper/mod.rs` changes:**
- Add after `#[cfg(test)] mod tests;`:
  ```rust
  #[cfg(test)]
  #[path = "tests_lyrics.rs"]
  mod tests_lyrics;
  ```

---

### 3. `web/src/App.css` → `file-tab-bar.css` + `preview.css`

**Extract to `web/src/file-tab-bar.css`** (lines 62–280):
- Everything from `/* File tab bar */` through `.file-tab-bar-restore:hover`

**Extract to `web/src/preview.css`** (lines 356–506):
- Everything from `/* Preview */` through the end of file (incl. `.play-measure-*` + `@keyframes`)

**`App.css` keeps** lines 1–61 and 281–355:
- App shell: `.app`, `.app-header`, `.workspace`, `.pane`, `.editor-layout`, `.editor-main`
- Pane divider + preview pane background (`.pane--preview`, `.pane-divider`)
- Editor section: `.editor`, `.editor-toolbar`, `.editor-surface`
- Error panel: `.error-panel*`

**`web/src/App.tsx` changes:**
- Add `import './file-tab-bar.css'`
- Add `import './preview.css'`

---

### 4. `src/grid_layout/tests.rs` → `tests_layout.rs`

**Extract to `src/grid_layout/tests_layout.rs`** (lines 272–566 of current file):
- `make_block_with_decorations` helper
- `hdr()`, `cfg_wide()` helpers
- All `layout_*` tests
- All `decoration_*` tests
- `footer_row_*` tests

`tests_layout.rs` needs its own `use` block (no `use super::*` — already uses crate-absolute paths):
```rust
use crate::ast::parsed::JianPuPitch;
use crate::compiler::types::{ColumnElement, CompileResult, Decoration, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::{GridContent, Header, VAlign};
use crate::render_config::RenderConfig;
```

**`grid_layout/layout.rs` changes:**
- Add after existing `#[cfg(test)] #[path = "tests.rs"] mod tests;`:
  ```rust
  #[cfg(test)]
  #[path = "tests_layout.rs"]
  mod tests_layout;
  ```

---

### 5. `src/grouper/mod.rs` → `directive_grouper.rs` + `lyrics_pairing.rs`

**Extract to `src/grouper/directive_grouper.rs`** (lines 50–140):
- `DirectiveGrouper` struct + full `impl DirectiveGrouper` block

Needs:
```rust
use crate::ast::grouped::{MeasureDirectives, TimeSignature};
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::error::Spanned;
```

Visibility: `pub(super) struct DirectiveGrouper` with `pub(super) fn new` and `pub(super) fn process_all`.

**Extract to `src/grouper/lyrics_pairing.rs`** (lines 463–579):
- `attach_paired_lyrics`
- `pair_lyrics_to_notes`

Needs:
```rust
use crate::ast::grouped::{GroupedMeasure, NoteEvent};
use crate::ast::parsed::{JianPuPitch, Syllable};
use crate::error::{IrrecoverableError, RecoverableError, Span};
```

Visibility: `pub(super) fn attach_paired_lyrics`, `fn pair_lyrics_to_notes`.

**`grouper/mod.rs` changes:**
- Add `mod directive_grouper;` and `mod lyrics_pairing;`
- Add `use directive_grouper::DirectiveGrouper;`
- Add `use lyrics_pairing::attach_paired_lyrics;`
- Remove extracted structs/functions

---

### 6. `web/src/hooks/useJianpuWorker.ts` → `workerHelpers.ts`

**Extract to `web/src/hooks/workerHelpers.ts`** (lines 6–111):
- `measureRangeInSpan` → export
- `enabledTracksForRender` → export
- `disabledLyricsForRender` → export
- `downloadPdf` → export
- `pdfFilenameFromActiveFile` → export
- `zipFilenameFromActiveFile` → export
- `baseNameFromActiveFile` → export
- `downloadZip` → export

`workerHelpers.ts` needs:
```typescript
import { findIndex, findLastIndex } from 'remeda'
import type { MeasureSpan, PartInfo } from '../types'
```

**`useJianpuWorker.ts` changes:**
- Replace inline definitions with:
  ```typescript
  import {
    measureRangeInSpan, enabledTracksForRender, disabledLyricsForRender,
    downloadPdf, pdfFilenameFromActiveFile, zipFilenameFromActiveFile,
    baseNameFromActiveFile, downloadZip,
  } from './workerHelpers'
  ```
- Remove the `findIndex`/`findLastIndex` import (moved to workerHelpers.ts)
- Remove `MeasureSpan` from types import if only used by extracted fns

---

### 7. `src/compiler/mod.rs` → `beam.rs` + `part_slice.rs`

**Extract to `src/compiler/beam.rs`** (lines 152–220):
- `BeamEntry` struct → `pub(super)`
- `flush_beam_buffer` → `pub(super)`
- `compute_underline_levels` (private)

Needs:
```rust
use crate::compiler::types::{ColumnElement, ElementContent};
```

**Extract to `src/compiler/part_slice.rs`** (lines 222–597):
- `PartState` struct
- `compile_part_slice` → `pub(super)`
- `compile_note`, `compile_rest`, `compile_chord` (private)

Needs:
```rust
use super::beam::{BeamEntry, flush_beam_buffer};
use super::PartSliceResult;
use super::slur_chains::{extend_note_chains, PendingSlurOpen, SlurKey};
use crate::ast::grouped::{GroupedChordNote, GroupedNote, GroupedRest, NoteEvent, PartSlice};
use crate::ast::parsed::{PartKind, Syllable};
use crate::compiler::types::{ColumnElement, ElementContent, SlurSpan};
```

**`compiler/mod.rs` changes:**
- Add `mod beam;` and `mod part_slice;`
- Add `use beam::{BeamEntry, flush_beam_buffer};`
- Add `use part_slice::compile_part_slice;`
- Keep: `PartSliceResult`, `compile`, `compile_measure`, `collect_decorations`

---

### 8. `src/lib.rs` → `filters.rs` + `measure_spans.rs` + `split_track.rs`

**Extract to `src/filters.rs`** (lines 206–263):
- `apply_track_filter` → `pub`
- `filter_tracks` → `pub`
- `apply_lyrics_filter` → `pub`

Needs:
```rust
use crate::ast::grouped::{PartRow, Score};
use crate::ast::parsed::PartKind;
```

**Extract to `src/measure_spans.rs`** (lines 265–344):
- `find_measure_at_byte_offset` → `pub`
- `find_measure_at_line_number` → `pub`
- `MeasureSourceSpan` struct → `pub`
- `list_measure_spans_from_source` → `pub`

Needs:
```rust
use crate::ast::grouped::Score;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use crate::parser;
```

**Extract to `src/split_track.rs`** (lines 346–497):
- `sanitize_track_name` → `pub`
- `split_track_label` → `pub`
- `split_track_filename` → `pub`
- `collect_track_names` → `pub`
- `split_pdf_filename` → `pub`
- `split_track_names` → `pub`
- `SplitPdfEntry` struct → `pub`
- `write_split_pdfs_from_source` → `pub` (cfg feature = "pdf")
- `zip_split_pdfs` → `pub` (cfg feature = "pdf")

Needs:
```rust
use crate::ast::grouped::Score;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use crate::{list_parts_from_source, render_svgs};
use crate::filters::filter_tracks;
```

**`lib.rs` changes:**
- Add `pub mod filters;`, `pub mod measure_spans;`, `pub mod split_track;`
- Add re-exports: `pub use filters::*; pub use measure_spans::*; pub use split_track::*;`
- Remove extracted code
- Keep: `RenderOutput`, `PartInfo`, `compile`, `render_svgs`, `render_svgs_from_source*`, `write_wav*`, `write_pdf*`

---

### 9. `src/grid_layout/layout.rs` → `layout_decoration.rs` + additions to `expand.rs`

**Extract to `src/grid_layout/layout_decoration.rs`** (lines 320–457):
- `deco_order` local fn (inline closure → stays local inside `make_decoration_row`)
- `make_decoration_row` → `pub(super)`
- `make_separator_row` → `pub(super)`
- `make_header_rows` → `pub(super)`

Needs:
```rust
use crate::compiler::types::{Decoration, MeasureBlock};
use crate::grid_layout::layout::{decoration_row_height, header_subtitle_author_row_height, header_title_row_height, has_any_decoration, separator_row_height, LABEL_COLS};
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, Header, VAlign};
```

Wait — `layout_decoration.rs` is a sibling of `layout.rs` (both inside `grid_layout`), not a child. So it accesses `layout::*` via crate paths. The functions it calls (`decoration_row_height`, etc.) need to be `pub(super)` in `layout.rs`.

**Move into existing `src/grid_layout/expand.rs`** (lines 178–313 of layout.rs):
- `expand_lyric_part` → `pub(crate)`
- `expand_note_part` → `pub(crate)` (already uses `#[allow(clippy::indexing_slicing)]`)
- `expand_system_to_rows` → `pub(crate)`

These already import from `layout.rs` via `use crate::grid_layout::layout::*`.

**`layout.rs` changes:**
- Add `#[path = "layout_decoration.rs"] mod decoration;` (or declare in `mod.rs`)
- Import `use decoration::{make_decoration_row, make_separator_row, make_header_rows};`
- Move expand functions to `expand.rs` and import via `use super::expand::{expand_lyric_part, expand_note_part, expand_system_to_rows};`
- Remove extracted code

**`grid_layout/mod.rs` changes:**
- Add `mod layout_decoration;` (or use `#[path]` in layout.rs — either works)

---

### 10. `src/parser/score/interleaved_parser.rs` → `interleaved_column_lines.rs` + `interleaved_accumulators.rs`

Uses the existing `#[path = "..."]` submodule pattern already established in this file.

**Extract to `src/parser/score/interleaved_column_lines.rs`** as a child module of `interleaved_parser` (lines 294–529):
- `process_padded_columns`
- `process_lyrics_column_line`
- `process_notes_column_line`
- `process_column_line`

Uses `use super::*` so it inherits `BarGroupContext`, `TrackAccumulator`, `SlotAction`, and all imports from parent. Additional imports needed:
```rust
use super::beat_padding::{validate_and_pad_beats, PaddedBeats};
use super::errors::invariant;
use super::{timed_events_mut, notes_syllables_mut};
use crate::parser::score::token_parser;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
```

Declare in `interleaved_parser.rs`:
```rust
#[path = "interleaved_column_lines.rs"]
mod column_lines;
use column_lines::{process_padded_columns};
```

(`process_bar_group` calls `process_padded_columns` directly, so only that one needs explicit use.)

**Extract to `src/parser/score/interleaved_accumulators.rs`** as a child module (lines 176–213, 532–584):
- `build_slot_actions`
- `init_accumulators`
- `build_parse_result`

Uses `use super::*` for `TrackAccumulator`, `SlotAction`, `DittoMeasures`, etc. Additional:
```rust
use crate::ast::parsed::{PartDecl, PartKind, ParsedTimedTrack, ParsedScore, ParsedLyrics};
use crate::error::{IrrecoverableError, Span};
use super::errors::invariant;
use super::ditto::DittoMeasures;
```

Declare in `interleaved_parser.rs`:
```rust
#[path = "interleaved_accumulators.rs"]
mod accumulators;
use accumulators::{build_slot_actions, init_accumulators, build_parse_result};
```

---

## Execution order

Since all 10 splits are independent of each other, they can be executed in any order. Suggested batches for parallel execution:

**Batch A** (simplest, lowest risk):
1. `src/midi/mod.rs`
2. `src/grouper/tests.rs`
3. `web/src/App.css`

**Batch B** (test files + TS hook):
4. `src/grid_layout/tests.rs`
5. `web/src/hooks/useJianpuWorker.ts`

**Batch C** (Rust module splits — medium):
6. `src/grouper/mod.rs`
7. `src/compiler/mod.rs`

**Batch D** (Rust module splits — complex):
8. `src/lib.rs`
9. `src/grid_layout/layout.rs`
10. `src/parser/score/interleaved_parser.rs`

## Verification after all splits

```sh
cargo test
cd web && pnpm exec tsc -b
python3 scripts/check-max-file-lines.py $(find src web/src -name '*.rs' -o -name '*.ts' -o -name '*.tsx' -o -name '*.css' | grep -v target | grep -v node_modules)
```
