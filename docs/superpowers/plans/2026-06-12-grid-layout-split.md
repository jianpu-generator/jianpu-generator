# Grid Layout Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the monolithic `layout` + `compositor` layers into `grid_layout` (produces column-indexed `GridPage`) and `coordinate_resolver` (converts `GridPage` → `AbsolutePage` with pure arithmetic).

**Architecture:** The new `grid_layout` module replaces `src/layout/` and expands compiler `MeasureBlock`s into flat `GridRow`s with column-indexed `GridElement`s and baked-in `height_pt`. The new `coordinate_resolver` module replaces `src/compositor/mod.rs` and converts grid positions to pixel coordinates using only the `height_pt` and column arithmetic — no musical knowledge. `src/compositor/types.rs` (AbsolutePage, AbsoluteElement, AbsoluteContent) is kept unchanged so the renderer requires no edits.

**Tech Stack:** Rust, no new dependencies. Run tests with `cargo test`. Page size: 595×842 pt (A4). `PAGE_MARGIN = 25.0` pt.

---

## Pragmatic deviations from spec

These are intentional simplifications to avoid touching the renderer:

1. **`GridContent::NoteHead` keeps `octave: i8`** — the renderer renders octave dots inline with the note head element; the OctaveDot sub-rows exist for correct vertical spacing but the resolver emits nothing for them.
2. **`GridContent::Underline` keeps `level: u32`** — the resolver passes this directly to `AbsoluteContent::Underline { level }`.
3. **All header/footer/separator rows use `column_count = 1`** — the resolver computes `usable_width` as `column_width`, giving correct pixel positions for `HAlign::Center` and `HAlign::End`.

---

## File Map

| Action | Path | Purpose |
|--------|------|---------|
| Create | `src/grid_layout/mod.rs` | Module root; `PAGE_MARGIN` const |
| Create | `src/grid_layout/types.rs` | `GridPage`, `GridRow`, `GridElement`, enums |
| Create | `src/grid_layout/layout.rs` | `pub fn layout(...)` — main grid layout function |
| Create | `src/grid_layout/tests.rs` | Unit tests for grid layout |
| Create | `src/coordinate_resolver/mod.rs` | Module root |
| Create | `src/coordinate_resolver/resolve.rs` | `pub fn resolve(...)` — coordinate math |
| Create | `src/coordinate_resolver/tests.rs` | Unit tests for resolver |
| Modify | `src/lib.rs` | Declare new modules; update `render_svgs` pipeline |
| Delete | `src/layout/new_layout.rs` | Replaced by `grid_layout/layout.rs` |
| Delete | `src/layout/new_types.rs` | Replaced by `grid_layout/types.rs` |
| Delete | `src/layout/tests.rs` | Tests moved to `grid_layout/tests.rs` |
| Modify | `src/layout/mod.rs` | Remove submodules (or delete whole dir) |
| Modify | `src/compositor/mod.rs` | Strip to `pub mod types; pub use types::*;` |
| Delete | `src/compositor/header_footer.rs` | Logic moved to `grid_layout/layout.rs` |
| Delete | `src/compositor/tests.rs` | Tests moved to new modules |

---

## Task 1: Create branch and declare new modules

**Files:**
- Modify: `src/lib.rs`
- Create: `src/grid_layout/mod.rs`
- Create: `src/coordinate_resolver/mod.rs`

- [ ] **Step 1: Create branch**

```bash
git checkout -b feat/grid-layout-split
```

- [ ] **Step 2: Create module roots (empty for now)**

Create `src/grid_layout/mod.rs`:
```rust
pub mod types;

pub(crate) const PAGE_MARGIN: f32 = 25.0;
```

Create `src/coordinate_resolver/mod.rs`:
```rust
// placeholder — will export resolve function
```

- [ ] **Step 3: Declare modules in `src/lib.rs`**

Add after the existing `pub mod compositor;` line:
```rust
pub mod coordinate_resolver;
pub mod grid_layout;
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo check
```
Expected: no errors (empty modules compile fine).

- [ ] **Step 5: Commit**

```bash
git add src/grid_layout/mod.rs src/coordinate_resolver/mod.rs src/lib.rs
git commit -m "chore: declare grid_layout and coordinate_resolver module skeletons"
```

---

## Task 2: Define GridPage types

**Files:**
- Create: `src/grid_layout/types.rs`
- Create: `src/grid_layout/tests.rs`
- Modify: `src/grid_layout/mod.rs`

- [ ] **Step 1: Write failing test**

Create `src/grid_layout/tests.rs`:
```rust
use crate::grid_layout::types::{GridRow, HAlign, VAlign, GridContent, GridElement};

#[test]
fn column_width_pt_divides_evenly() {
    let row = GridRow {
        height_pt: 30.0,
        column_count: 10,
        elements: vec![],
    };
    assert_eq!(row.column_width_pt(500.0), 50.0);
}

#[test]
fn column_width_pt_with_label_columns() {
    // 4 label cols + 16 musical cols = 20 total; usable=400 → 20pt each
    let row = GridRow {
        height_pt: 30.0,
        column_count: 20,
        elements: vec![],
    };
    assert_eq!(row.column_width_pt(400.0), 20.0);
}
```

- [ ] **Step 2: Run to verify fail**

```bash
cargo test column_width_pt 2>&1 | tail -10
```
Expected: compile error (types not defined yet).

- [ ] **Step 3: Implement types**

Create `src/grid_layout/types.rs`:
```rust
use crate::ast::parsed::JianPuPitch;

#[derive(Debug, Clone)]
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
}

#[derive(Debug, Clone)]
pub struct GridRow {
    pub height_pt: f32,
    pub column_count: u32,
    pub elements: Vec<GridElement>,
}

impl GridRow {
    /// Column width in points, given the usable page width.
    pub fn column_width_pt(&self, usable_width_pt: f32) -> f32 {
        usable_width_pt / self.column_count as f32
    }
}

#[derive(Debug, Clone)]
pub struct GridElement {
    pub column: u32,
    pub column_span: u32,
    pub halign: HAlign,
    pub valign: VAlign,
    pub content: GridContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone)]
pub enum GridContent {
    /// Note head. `octave > 0` = dots above, `octave < 0` = dots below,
    /// `octave.abs()` = dot count. Octave rendered inline by the renderer;
    /// OctaveDot sub-rows exist for vertical spacing only.
    NoteHead {
        pitch: JianPuPitch,
        octave: i8,
        dotted: bool,
    },
    Rest {
        dotted: bool,
    },
    NoteDash,
    /// Spacing-only row for octave dots. Resolver emits nothing for this.
    OctaveDot,
    ChordSymbol(String),
    /// Durational underline. `level=0` half-beat, `level=1` quarter-beat.
    Underline { level: u32 },
    /// Tie/slur arc. Width = column_span × column_width_pt.
    TieOrSlur,
    /// Closing arc at measure start. Arc runs from left edge to center of column.
    TieOrSlurClose,
    /// Vertical bar line. `height_pt` baked in by grid layout layer.
    BarLine { height_pt: f32 },
    /// Full-width horizontal system separator.
    HorizontalLine,
    /// Part name at column=0, column_span=4 in the note-head sub-row.
    RowLabel(String),
    LyricSyllable(String),
    Bpm(u32),
    TimeSignature { numerator: u32, denominator: u32 },
    SectionLabel(String),
    BarNumber(u32),
    /// Generic styled text for header and footer rows.
    Text {
        content: String,
        font_size: f32,
        bold: bool,
        italic: bool,
    },
}
```

- [ ] **Step 4: Wire test module into `src/grid_layout/mod.rs`**

```rust
pub mod types;

pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run tests to verify pass**

```bash
cargo test column_width_pt
```
Expected: 2 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/grid_layout/types.rs src/grid_layout/mod.rs src/grid_layout/tests.rs
git commit -m "feat: define GridPage, GridRow, GridElement types"
```

---

## Task 3: Height constants and part-type helpers

These private helpers compute per-row heights and classify MeasureRows. They live in `src/grid_layout/layout.rs`.

**Files:**
- Create: `src/grid_layout/layout.rs`
- Modify: `src/grid_layout/mod.rs`
- Modify: `src/grid_layout/tests.rs`

The sub-row heights for a Note/Chord part (all multiples of `base = config.row_height as f32`):

| Sub-row | Constant | Formula |
|---------|----------|---------|
| Tie/slur arc | `ARC_H` | `base * 0.30` |
| Above-octave dots | `DOT_H` | `base * 0.25` |
| Note head | `base` | `base * 1.00` |
| Below-octave dots | `DOT_H` | `base * 0.25` |
| Half-beat underline | `UL_H` | `base * 0.15` |
| Quarter-beat underline | `UL_H` | `base * 0.15` |

Chord-only part (4 sub-rows): arc, chord main (`base * 0.75`), half-beat UL, quarter-beat UL.

- [ ] **Step 1: Write failing tests**

Add to `src/grid_layout/tests.rs`:
```rust
use crate::compiler::types::{ColumnElement, ElementContent, MeasureRow, RowId};
use crate::ast::parsed::JianPuPitch;

fn note_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::NoteHead {
                pitch: JianPuPitch::One,
                octave: 0,
                dotted: false,
            },
        }],
    }
}

fn chord_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::ChordSymbol("Am".to_string()),
        }],
    }
}

fn lyric_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::Lyric("la".to_string()),
        }],
    }
}

use crate::grid_layout::layout::{is_lyric_row, is_chord_only_row, note_part_sub_row_heights, chord_part_sub_row_heights};

#[test]
fn is_lyric_row_detects_lyric() {
    assert!(is_lyric_row(&lyric_row("L")));
    assert!(!is_lyric_row(&note_row("S")));
}

#[test]
fn is_chord_only_row_detects_chord() {
    assert!(is_chord_only_row(&chord_row("C")));
    assert!(!is_chord_only_row(&note_row("S")));
    assert!(!is_chord_only_row(&lyric_row("L")));
}

#[test]
fn note_part_sub_row_heights_sums_correctly() {
    let heights = note_part_sub_row_heights(30.0);
    // arc + above_dot + note_head + below_dot + ul + ul
    // = 9.0 + 7.5 + 30.0 + 7.5 + 4.5 + 4.5 = 63.0
    let sum: f32 = heights.iter().sum();
    assert!((sum - 63.0).abs() < 0.001, "sum={sum}");
    assert_eq!(heights.len(), 6);
}

#[test]
fn chord_part_sub_row_heights_has_four_rows() {
    let heights = chord_part_sub_row_heights(30.0);
    assert_eq!(heights.len(), 4);
}
```

- [ ] **Step 2: Run to verify fail**

```bash
cargo test is_lyric_row 2>&1 | tail -5
```
Expected: compile error.

- [ ] **Step 3: Implement helpers**

Create `src/grid_layout/layout.rs`:
```rust
use crate::compiler::types::{MeasureBlock, MeasureRow, ElementContent};

// ── Row classification ────────────────────────────────────────────────────────

pub(crate) fn is_lyric_row(row: &MeasureRow) -> bool {
    row.elements
        .iter()
        .any(|e| matches!(e.content, ElementContent::Lyric(_)))
}

pub(crate) fn is_chord_only_row(row: &MeasureRow) -> bool {
    if is_lyric_row(row) {
        return false;
    }
    let has_note = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::NoteHead { .. } | ElementContent::Rest { .. }
        )
    });
    !has_note
        && row
            .elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::ChordSymbol(_)))
}

// ── Sub-row heights ───────────────────────────────────────────────────────────

/// Returns the 6 sub-row heights for a Note/Chord part, in order:
/// [arc, above_dot, note_head, below_dot, half_ul, quarter_ul]
pub(crate) fn note_part_sub_row_heights(base: f32) -> [f32; 6] {
    [
        base * 0.30, // tie/slur arc
        base * 0.25, // above-octave dots
        base,        // note head (main)
        base * 0.25, // below-octave dots
        base * 0.15, // half-beat underline
        base * 0.15, // quarter-beat underline
    ]
}

/// Returns the 4 sub-row heights for a Chord-symbol-only part, in order:
/// [arc, chord_main, half_ul, quarter_ul]
pub(crate) fn chord_part_sub_row_heights(base: f32) -> [f32; 4] {
    [
        base * 0.30, // tie/slur arc
        base * 0.75, // chord symbol (main)
        base * 0.15, // half-beat underline
        base * 0.15, // quarter-beat underline
    ]
}

pub(crate) fn lyric_row_height(base: f32) -> f32 {
    base * 0.50
}

pub(crate) fn decoration_row_height(base: f32) -> f32 {
    base * 0.50
}

pub(crate) fn separator_row_height() -> f32 {
    4.0
}

pub(crate) fn header_title_row_height(base: f32) -> f32 {
    base * 0.80
}

pub(crate) fn header_subtitle_author_row_height(base: f32) -> f32 {
    base * 0.50
}

pub(crate) fn footer_row_height(base: f32) -> f32 {
    base * 0.40
}

// ── Column width helper ───────────────────────────────────────────────────────

/// Number of columns in a MeasureBlock (BarLine column + 1).
pub(crate) fn block_column_width(block: &MeasureBlock) -> u32 {
    block
        .rows
        .first()
        .and_then(|row| {
            row.elements
                .iter()
                .find(|e| e.content == ElementContent::BarLine)
        })
        .map(|e| e.column + 1)
        .unwrap_or(1)
}

/// Total height in points for all musical sub-rows in a system
/// (sum over all part rows, excluding lyric rows).
pub(crate) fn system_musical_height_pt(block: &MeasureBlock, base: f32) -> f32 {
    block
        .rows
        .iter()
        .filter(|r| !is_lyric_row(r))
        .map(|r| {
            if is_chord_only_row(r) {
                chord_part_sub_row_heights(base).iter().sum::<f32>()
            } else {
                note_part_sub_row_heights(base).iter().sum::<f32>()
            }
        })
        .sum()
}

/// Total height in points for lyric rows in a system.
pub(crate) fn system_lyric_height_pt(block: &MeasureBlock, base: f32) -> f32 {
    block
        .rows
        .iter()
        .filter(|r| is_lyric_row(r))
        .count() as f32
        * lyric_row_height(base)
}
```

- [ ] **Step 4: Wire layout module**

Add to `src/grid_layout/mod.rs`:
```rust
pub mod layout;
pub mod types;

pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run tests**

```bash
cargo test grid_layout
```
Expected: 6 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/grid_layout/layout.rs src/grid_layout/mod.rs src/grid_layout/tests.rs
git commit -m "feat: add height helpers and part-type classifiers to grid_layout"
```

---

## Task 4: System packing

Pack measures into systems. A new system starts when adding a block would exceed `config.max_columns` or the set of row IDs changes. Returns `Vec<Vec<MeasureBlock>>`.

**Files:**
- Modify: `src/grid_layout/layout.rs`
- Modify: `src/grid_layout/tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/grid_layout/tests.rs`:
```rust
use crate::compiler::types::{MeasureBlock, RowId};
use crate::render_config::RenderConfig;
use crate::grid_layout::layout::pack_into_systems;

fn make_block(row_id: &str, bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: bar_col,
                    content: ElementContent::BarLine,
                },
            ],
        }],
        decorations: vec![],
    }
}

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_columns: 8,
    }
}

#[test]
fn single_block_is_one_system() {
    let blocks = vec![make_block("S", 3)]; // 4 columns
    let systems = pack_into_systems(&blocks, &cfg());
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].len(), 1);
}

#[test]
fn blocks_exceeding_max_columns_split_into_two_systems() {
    // Each block is 4 cols wide; max=8 → fits 2 per system
    let blocks = vec![
        make_block("S", 3),
        make_block("S", 3),
        make_block("S", 3),
    ];
    let systems = pack_into_systems(&blocks, &cfg());
    assert_eq!(systems.len(), 2);
    assert_eq!(systems[0].len(), 2);
    assert_eq!(systems[1].len(), 1);
}

#[test]
fn different_row_ids_start_new_system() {
    let blocks = vec![make_block("A", 3), make_block("B", 3)];
    let systems = pack_into_systems(&blocks, &cfg());
    assert_eq!(systems.len(), 2);
}
```

- [ ] **Step 2: Run to verify fail**

```bash
cargo test pack_into_systems 2>&1 | tail -5
```
Expected: compile error.

- [ ] **Step 3: Implement `pack_into_systems`**

Add to `src/grid_layout/layout.rs`:
```rust
use crate::compiler::types::RowId;
use crate::render_config::RenderConfig;

fn row_ids(block: &MeasureBlock) -> Vec<&RowId> {
    block.rows.iter().map(|r| &r.id).collect()
}

/// Break `blocks` into systems. Each system is a `Vec<MeasureBlock>`.
pub(crate) fn pack_into_systems(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
) -> Vec<Vec<MeasureBlock>> {
    let mut systems: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut current: Vec<MeasureBlock> = Vec::new();
    let mut current_cols: u32 = 0;

    for block in blocks {
        let col_w = block_column_width(block);
        let needs_new = if let Some(first) = current.first() {
            current_cols + col_w > config.max_columns || row_ids(block) != row_ids(first)
        } else {
            false
        };

        if needs_new {
            if !current.is_empty() {
                systems.push(std::mem::take(&mut current));
                current_cols = 0;
            }
        }

        current_cols += col_w;
        current.push(block.clone());
    }

    if !current.is_empty() {
        systems.push(current);
    }

    systems
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test pack_into_systems
```
Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/grid_layout/layout.rs src/grid_layout/tests.rs
git commit -m "feat: implement system packing for grid_layout"
```

---

## Task 5: Grid row expansion (musical content)

Convert a system's measures into flat `GridRow`s. The label columns (0-3) are reserved; compiler columns are offset by 4.

**Files:**
- Modify: `src/grid_layout/layout.rs`
- Modify: `src/grid_layout/tests.rs`

The sub-row indices for a Note/Chord part (0-indexed):
- 0: arc (TieOrSlur, TieOrSlurClose, BarLine starts here)
- 1: above-octave dots (OctaveDot when octave > 0)
- 2: note head (NoteHead, Rest, NoteDash, RowLabel, BarLine height starts here)
- 3: below-octave dots (OctaveDot when octave < 0)
- 4: half-beat underline (Underline level=0)
- 5: quarter-beat underline (Underline level=1)

For a Chord-only part (0-indexed):
- 0: arc (TieOrSlur, TieOrSlurClose)
- 1: chord main (ChordSymbol)
- 2: half-beat underline (Underline level=0)
- 3: quarter-beat underline (Underline level=1)

`LABEL_COLS = 4` — the column offset for all musical content.

- [ ] **Step 1: Write failing tests**

Add to `src/grid_layout/tests.rs`:
```rust
use crate::compiler::types::Decoration;
use crate::grid_layout::layout::expand_system_to_rows;
use crate::grid_layout::types::{GridContent, HAlign, VAlign};

fn make_system_single_note_block() -> Vec<MeasureBlock> {
    vec![make_block("S", 3)] // 4 musical cols, bar at compiler col 3
}

#[test]
fn note_block_expands_to_six_sub_rows_plus_lyric_is_absent() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0);
    // 1 note part × 6 sub-rows, no lyric row
    assert_eq!(rows.len(), 6);
}

#[test]
fn note_head_element_is_in_sub_row_index_2() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0);
    let note_row = &rows[2]; // note-head sub-row
    let has_note = note_row.elements.iter().any(|e| {
        matches!(e.content, GridContent::NoteHead { .. })
    });
    assert!(has_note, "note head should be in sub-row 2");
}

#[test]
fn bar_line_element_has_positive_height_pt() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0);
    let bar = rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::BarLine { .. }));
    let bar = bar.expect("should have a BarLine element");
    if let GridContent::BarLine { height_pt } = bar.content {
        assert!(height_pt > 0.0, "height_pt={height_pt}");
    }
}

#[test]
fn row_label_is_in_note_head_sub_row_at_column_0_span_4() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0);
    let note_row = &rows[2];
    let label = note_row
        .elements
        .iter()
        .find(|e| matches!(e.content, GridContent::RowLabel(_)));
    let label = label.expect("note-head row should have RowLabel");
    assert_eq!(label.column, 0);
    assert_eq!(label.column_span, 4);
}

#[test]
fn column_count_is_label_cols_plus_musical_cols() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0);
    // 4 label cols + 4 musical cols (bar at col 3 → col_width=4)
    assert_eq!(rows[0].column_count, 8);
}
```

- [ ] **Step 2: Run to verify fail**

```bash
cargo test expand_system 2>&1 | tail -5
```
Expected: compile error.

- [ ] **Step 3: Implement `expand_system_to_rows`**

Add to `src/grid_layout/layout.rs`:
```rust
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, VAlign};

const LABEL_COLS: u32 = 4;

/// Convert a system's measures into flat GridRows.
/// Does not include decoration, separator, header, or footer rows.
pub(crate) fn expand_system_to_rows(
    system: &[MeasureBlock],
    base: f32,
) -> Vec<GridRow> {
    let Some(first) = system.first() else {
        return vec![];
    };

    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let column_count = LABEL_COLS + total_musical_cols;

    // Compute bar-line span height: sum of all non-lyric sub-rows
    let bar_height: f32 = first
        .rows
        .iter()
        .filter(|r| !is_lyric_row(r))
        .map(|r| {
            if is_chord_only_row(r) {
                chord_part_sub_row_heights(base).iter().sum::<f32>()
            } else {
                note_part_sub_row_heights(base).iter().sum::<f32>()
            }
        })
        .sum();

    let mut all_rows: Vec<GridRow> = Vec::new();

    for (part_idx, part_template) in first.rows.iter().enumerate() {
        if is_lyric_row(part_template) {
            // Lyric row: one flat row, all measures
            let mut row = GridRow {
                height_pt: lyric_row_height(base),
                column_count,
                elements: vec![],
            };
            let mut measure_col_offset: u32 = 0;
            for block in system {
                let col_w = block_column_width(block);
                if let Some(part_row) = block.rows.get(part_idx) {
                    for el in &part_row.elements {
                        if let ElementContent::Lyric(text) = &el.content {
                            row.elements.push(GridElement {
                                column: LABEL_COLS + measure_col_offset + el.column,
                                column_span: 1,
                                halign: HAlign::Center,
                                valign: VAlign::Center,
                                content: GridContent::LyricSyllable(text.clone()),
                            });
                        }
                    }
                }
                measure_col_offset += col_w;
            }
            all_rows.push(row);
            continue;
        }

        let (sub_heights, sub_count): (Vec<f32>, usize) = if is_chord_only_row(part_template) {
            (chord_part_sub_row_heights(base).to_vec(), 4)
        } else {
            (note_part_sub_row_heights(base).to_vec(), 6)
        };

        // Initialise sub-rows
        let mut sub_rows: Vec<GridRow> = sub_heights
            .iter()
            .map(|&h| GridRow {
                height_pt: h,
                column_count,
                elements: vec![],
            })
            .collect();

        // RowLabel in note-head sub-row (index 2 for note/chord, index 1 for chord-only)
        let head_sub = if is_chord_only_row(part_template) { 1 } else { 2 };
        if !part_template.label.is_empty() {
            sub_rows[head_sub].elements.push(GridElement {
                column: 0,
                column_span: LABEL_COLS,
                halign: HAlign::Center,
                valign: VAlign::Center,
                content: GridContent::RowLabel(part_template.label.clone()),
            });
        }

        // Opening bar line (at start of system, left edge of musical cols)
        // Placed in sub-row 0 (arc row), column = LABEL_COLS, span = 1
        if part_idx == 0 {
            sub_rows[0].elements.push(GridElement {
                column: LABEL_COLS,
                column_span: 1,
                halign: HAlign::Start,
                valign: VAlign::Top,
                content: GridContent::BarLine { height_pt: bar_height },
            });
        }

        // Musical elements across all measures
        let mut measure_col_offset: u32 = 0;
        for block in system {
            let col_w = block_column_width(block);
            if let Some(part_row) = block.rows.get(part_idx) {
                expand_measure_elements(
                    part_row,
                    measure_col_offset,
                    col_w,
                    head_sub,
                    sub_count,
                    bar_height,
                    part_idx,
                    &mut sub_rows,
                );
            }
            measure_col_offset += col_w;
        }

        all_rows.extend(sub_rows);
    }

    all_rows
}

fn expand_measure_elements(
    row: &MeasureRow,
    measure_col_offset: u32,
    _measure_col_w: u32,
    head_sub: usize,
    sub_count: usize,
    bar_height: f32,
    part_idx: usize,
    sub_rows: &mut Vec<GridRow>,
) {
    for el in &row.elements {
        let grid_col = LABEL_COLS + measure_col_offset + el.column;
        match &el.content {
            ElementContent::NoteHead { pitch, octave, dotted } => {
                sub_rows[head_sub].elements.push(GridElement {
                    column: grid_col,
                    column_span: 1,
                    halign: HAlign::Center,
                    valign: VAlign::Center,
                    content: GridContent::NoteHead {
                        pitch: pitch.clone(),
                        octave: *octave,
                        dotted: *dotted,
                    },
                });
            }
            ElementContent::Rest { dotted } => {
                sub_rows[head_sub].elements.push(GridElement {
                    column: grid_col,
                    column_span: 1,
                    halign: HAlign::Center,
                    valign: VAlign::Center,
                    content: GridContent::Rest { dotted: *dotted },
                });
            }
            ElementContent::NoteDash => {
                sub_rows[head_sub].elements.push(GridElement {
                    column: grid_col,
                    column_span: 1,
                    halign: HAlign::Center,
                    valign: VAlign::Center,
                    content: GridContent::NoteDash,
                });
            }
            ElementContent::ChordSymbol(s) => {
                sub_rows[head_sub].elements.push(GridElement {
                    column: grid_col,
                    column_span: 1,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::ChordSymbol(s.clone()),
                });
            }
            ElementContent::Underline { from_column, last_head_column, level, .. } => {
                let span = last_head_column.saturating_sub(*from_column) + 1;
                // half-beat = last sub-row pair index; quarter-beat = next
                // For note (6 sub-rows): ul rows are indices 4 and 5
                // For chord (4 sub-rows): ul rows are indices 2 and 3
                let ul_base = sub_count - 2;
                let ul_sub = ul_base + *level as usize;
                if ul_sub < sub_count {
                    sub_rows[ul_sub].elements.push(GridElement {
                        column: grid_col,
                        column_span: span,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::Underline { level: *level },
                    });
                }
            }
            ElementContent::TieOrSlur { from_column, to_column } => {
                let span = to_column.saturating_sub(*from_column) + 1;
                sub_rows[0].elements.push(GridElement {
                    column: LABEL_COLS + measure_col_offset + from_column,
                    column_span: span,
                    halign: HAlign::Center,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlur,
                });
            }
            ElementContent::TieOrSlurClose { to_column } => {
                sub_rows[0].elements.push(GridElement {
                    column: LABEL_COLS + measure_col_offset + to_column,
                    column_span: 1,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlurClose,
                });
            }
            ElementContent::BarLine => {
                // Only emit from first part to avoid duplicate bar lines
                if part_idx == 0 {
                    sub_rows[0].elements.push(GridElement {
                        column: grid_col,
                        column_span: 1,
                        halign: HAlign::Center,
                        valign: VAlign::Top,
                        content: GridContent::BarLine { height_pt: bar_height },
                    });
                }
            }
            ElementContent::Lyric(_) => {} // handled in lyric-row branch above
        }
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test grid_layout
```
Expected: all grid_layout tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/grid_layout/layout.rs src/grid_layout/tests.rs
git commit -m "feat: expand system measures into flat GridRows"
```

---

## Task 6: Complete `layout()` public function

Adds decoration rows, system separator rows, header/footer rows, and page packing. This completes `src/grid_layout/layout.rs`.

**Files:**
- Modify: `src/grid_layout/layout.rs`
- Modify: `src/grid_layout/mod.rs`
- Modify: `src/grid_layout/tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `src/grid_layout/tests.rs`:
```rust
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::GridPage;
use crate::layout::new_types::Header;

fn hdr() -> Header {
    Header {
        title: "Song".to_string(),
        subtitle: None,
        author: "Me".to_string(),
    }
}

fn cfg_wide() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_columns: 48,
    }
}

#[test]
fn layout_single_block_produces_one_page() {
    let blocks = vec![make_block("S", 3)];
    let pages = layout(&blocks, &cfg_wide(), &hdr(), 595.0, 842.0);
    assert_eq!(pages.len(), 1);
}

#[test]
fn layout_page_has_correct_dimensions() {
    let blocks = vec![make_block("S", 3)];
    let pages = layout(&blocks, &cfg_wide(), &hdr(), 595.0, 842.0);
    assert!((pages[0].width_pt - 595.0).abs() < 0.001);
    assert!((pages[0].height_pt - 842.0).abs() < 0.001);
}

#[test]
fn layout_rows_include_header_and_footer() {
    let blocks = vec![make_block("S", 3)];
    let pages = layout(&blocks, &cfg_wide(), &hdr(), 595.0, 842.0);
    // At minimum: header title row, header subtitle+author row, footer row
    assert!(pages[0].rows.len() >= 3, "len={}", pages[0].rows.len());
}

#[test]
fn layout_page_total_height_does_not_exceed_page_height() {
    let blocks: Vec<_> = (0..10).map(|_| make_block("S", 3)).collect();
    let pages = layout(&blocks, &cfg_wide(), &hdr(), 595.0, 842.0);
    for page in &pages {
        let total: f32 = page.rows.iter().map(|r| r.height_pt).sum();
        assert!(
            total <= page.height_pt,
            "total={total} > page={}",
            page.height_pt
        );
    }
}

#[test]
fn layout_with_bpm_decoration_has_decoration_row() {
    let block = MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("S".to_string()),
            label: "S".to_string(),
            elements: vec![
                ColumnElement { column: 0, content: ElementContent::NoteHead { pitch: JianPuPitch::One, octave: 0, dotted: false } },
                ColumnElement { column: 3, content: ElementContent::BarLine },
            ],
        }],
        decorations: vec![Decoration::Bpm(120)],
    };
    let pages = layout(&[block], &cfg_wide(), &hdr(), 595.0, 842.0);
    let has_bpm = pages[0].rows.iter().flat_map(|r| r.elements.iter()).any(|e| {
        matches!(e.content, GridContent::Bpm(120))
    });
    assert!(has_bpm, "should have Bpm(120) element");
}
```

- [ ] **Step 2: Run to verify fail**

```bash
cargo test layout_single_block 2>&1 | tail -5
```
Expected: compile error (`layout` not exported yet).

- [ ] **Step 3: Implement `layout()` and decoration helpers**

Add to `src/grid_layout/layout.rs`. Add these imports at the top:
```rust
use crate::compiler::types::Decoration;
use crate::layout::new_types::Header;
```

Then add the following functions:

```rust
fn has_any_decoration(block: &MeasureBlock) -> bool {
    !block.decorations.is_empty()
}

/// Emit one decoration GridRow for the decorations of the system-opening measure.
fn make_decoration_row(system: &[MeasureBlock], base: f32) -> GridRow {
    let Some(first) = system.first() else {
        return GridRow { height_pt: decoration_row_height(base), column_count: 1, elements: vec![] };
    };
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let column_count = LABEL_COLS + total_musical_cols;
    let mut elements: Vec<GridElement> = Vec::new();

    for dec in &first.decorations {
        match dec {
            Decoration::Bpm(bpm) => elements.push(GridElement {
                column: LABEL_COLS,
                column_span: column_count - LABEL_COLS,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::Bpm(*bpm),
            }),
            Decoration::TimeSignature { numerator, denominator } => elements.push(GridElement {
                column: LABEL_COLS,
                column_span: column_count - LABEL_COLS,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::TimeSignature { numerator: *numerator, denominator: *denominator },
            }),
            Decoration::SectionLabel(s) => elements.push(GridElement {
                column: LABEL_COLS,
                column_span: column_count - LABEL_COLS,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::SectionLabel(s.clone()),
            }),
            Decoration::BarNumber(n) => elements.push(GridElement {
                column: LABEL_COLS,
                column_span: column_count - LABEL_COLS,
                halign: HAlign::Start,
                valign: VAlign::Bottom,
                content: GridContent::BarNumber(*n),
            }),
        }
    }

    GridRow { height_pt: decoration_row_height(base), column_count, elements }
}

fn make_separator_row() -> GridRow {
    GridRow {
        height_pt: separator_row_height(),
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Start,
            valign: VAlign::Center,
            content: GridContent::HorizontalLine,
        }],
    }
}

fn make_header_rows(header: &Header, base: f32) -> Vec<GridRow> {
    let title_row = GridRow {
        height_pt: header_title_row_height(base),
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: header.title.clone(),
                font_size: base * 1.5,
                bold: false,
                italic: false,
            },
        }],
    };

    let mut subtitle_author_elements: Vec<GridElement> = Vec::new();
    if let Some(subtitle) = &header.subtitle {
        subtitle_author_elements.push(GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: subtitle.clone(),
                font_size: base * 0.8,
                bold: false,
                italic: true,
            },
        });
    }
    subtitle_author_elements.push(GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::End,
        valign: VAlign::Center,
        content: GridContent::Text {
            content: header.author.clone(),
            font_size: base * 0.6,
            bold: false,
            italic: false,
        },
    });
    let subtitle_author_row = GridRow {
        height_pt: header_subtitle_author_row_height(base),
        column_count: 1,
        elements: subtitle_author_elements,
    };

    vec![title_row, subtitle_author_row]
}

fn make_footer_row(page_num: u32, total_pages: u32, base: f32) -> GridRow {
    GridRow {
        height_pt: footer_row_height(base),
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: format!("{page_num} / {total_pages}"),
                font_size: base * 0.6,
                bold: false,
                italic: false,
            },
        }],
    }
}

/// Total height of one system's rows in points.
fn system_total_height(system: &[MeasureBlock], base: f32) -> f32 {
    let Some(first) = system.first() else { return 0.0; };
    let musical = system_musical_height_pt(first, base);
    let lyric = system_lyric_height_pt(first, base);
    let deco = if has_any_decoration(first) { decoration_row_height(base) } else { 0.0 };
    musical + lyric + deco
}

/// Public entry point: convert compiler blocks to GridPages.
pub fn layout(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
) -> Vec<GridPage> {
    let base = config.row_height as f32;
    let systems = pack_into_systems(blocks, config);

    // Fixed per-page overhead (header + footer)
    let header_h: f32 = make_header_rows(header, base).iter().map(|r| r.height_pt).sum();
    let footer_h = footer_row_height(base);
    let usable_h = page_height_pt - 2.0 * super::PAGE_MARGIN - header_h - footer_h;

    // Pack systems into pages
    let mut page_systems: Vec<Vec<Vec<MeasureBlock>>> = Vec::new();
    let mut current_page: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut used_h: f32 = 0.0;

    for system in systems {
        let sys_h = system_total_height(&system, base);
        // Include separator gap for non-first systems
        let gap = if current_page.is_empty() { 0.0 } else { separator_row_height() };
        if !current_page.is_empty() && used_h + gap + sys_h > usable_h {
            page_systems.push(std::mem::take(&mut current_page));
            used_h = 0.0;
        }
        used_h += gap + sys_h;
        current_page.push(system);
    }
    page_systems.push(current_page);

    let total_pages = page_systems.len() as u32;
    page_systems
        .into_iter()
        .enumerate()
        .map(|(page_idx, systems)| {
            let mut rows: Vec<_> = make_header_rows(header, base);

            for (sys_idx, system) in systems.iter().enumerate() {
                if sys_idx > 0 {
                    rows.push(make_separator_row());
                }
                let Some(first) = system.first() else { continue; };
                if has_any_decoration(first) {
                    rows.push(make_decoration_row(system, base));
                }
                rows.extend(expand_system_to_rows(system, base));
            }

            rows.push(make_footer_row(page_idx as u32 + 1, total_pages, base));

            GridPage { width_pt: page_width_pt, height_pt: page_height_pt, rows }
        })
        .collect()
}
```

- [ ] **Step 4: Export `layout` from `src/grid_layout/mod.rs`**

Update `src/grid_layout/mod.rs`:
```rust
pub mod layout;
pub mod types;

pub use layout::layout;
pub use types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};

pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run tests**

```bash
cargo test grid_layout
```
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/grid_layout/layout.rs src/grid_layout/mod.rs src/grid_layout/tests.rs
git commit -m "feat: implement grid_layout::layout() with header, footer, decorations, and page packing"
```

---

## Task 7: Implement coordinate resolver

Converts `Vec<GridPage>` → `Vec<AbsolutePage>` using pure arithmetic. The resolver translates each `GridContent` variant to the corresponding `AbsoluteContent`. It knows nothing about musical meaning — only geometry.

**Files:**
- Modify: `src/coordinate_resolver/mod.rs`
- Create: `src/coordinate_resolver/resolve.rs`
- Create: `src/coordinate_resolver/tests.rs`

**Coordinate formulas:**
- `usable_width_pt = page_width_pt - 2 × PAGE_MARGIN`
- `col_width = row.column_width_pt(usable_width_pt)`
- `x_start = PAGE_MARGIN + element.column as f32 × col_width`
- `x` by `HAlign`: Start → `x_start`, Center → `x_start + element.column_span as f32 × col_width × 0.5`, End → `x_start + element.column_span as f32 × col_width`
- `y` by `VAlign`: Top → `row_y`, Center → `row_y + row.height_pt × 0.5`, Bottom → `row_y + row.height_pt`

- [ ] **Step 1: Write failing tests**

Create `src/coordinate_resolver/tests.rs`:
```rust
use crate::grid_layout::types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};
use crate::compositor::types::{AbsoluteContent, AbsolutePage};
use crate::coordinate_resolver::resolve::resolve;
use crate::ast::parsed::JianPuPitch;

fn single_row_page(element: GridElement) -> GridPage {
    GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            elements: vec![element],
        }],
    }
}

#[test]
fn resolve_empty_pages_returns_empty() {
    assert!(resolve(&[]).is_empty());
}

#[test]
fn note_head_halign_center_has_x_at_center_of_column() {
    // column_count=10, usable=595-50=545, col_width=54.5
    // column=0, halign=Center → x = 25 + 0*54.5 + 54.5*0.5 = 52.25
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::NoteHead { pitch: JianPuPitch::One, octave: 0, dotted: false },
    };
    let page = single_row_page(el);
    let abs = resolve(&[page]);
    let note = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }))
        .expect("should have NoteHead");
    let col_width = (595.0 - 50.0) / 10.0; // 54.5
    let expected_x = 25.0 + 0.0 * col_width + col_width * 0.5;
    assert!(
        (note.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        note.x
    );
}

#[test]
fn valign_top_places_y_at_row_top() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Top,
        content: GridContent::HorizontalLine,
    };
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![
            GridRow { height_pt: 10.0, column_count: 1, elements: vec![] },
            GridRow { height_pt: 20.0, column_count: 1, elements: vec![el] },
        ],
    };
    let abs = resolve(&[page]);
    let line = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::HorizontalLine { .. }))
        .expect("should have HorizontalLine");
    // row_y = PAGE_MARGIN + 10.0 = 35.0; VAlign::Top → y = row_y
    assert!((line.y - 35.0).abs() < 0.01, "y={}", line.y);
}

#[test]
fn halign_end_places_x_at_right_of_column_span() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::End,
        valign: VAlign::Center,
        content: GridContent::Text { content: "Author".to_string(), font_size: 12.0, bold: false, italic: false },
    };
    let page = single_row_page(el);
    let abs = resolve(&[page]);
    let text = abs[0]
        .elements
        .iter()
        .find(|e| matches!(&e.content, AbsoluteContent::Text { content, .. } if content == "Author"))
        .expect("should have Text");
    let col_width = (595.0 - 50.0) / 10.0;
    let expected_x = 25.0 + col_width; // Start + span*col_width = 25 + 54.5
    assert!((text.x - expected_x).abs() < 0.01, "x={} expected={expected_x}", text.x);
}

#[test]
fn octave_dot_grid_content_emits_nothing() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::OctaveDot,
    };
    let page = single_row_page(el);
    let abs = resolve(&[page]);
    assert!(abs[0].elements.is_empty(), "OctaveDot should emit no AbsoluteElement");
}
```

- [ ] **Step 2: Run to verify fail**

```bash
cargo test coordinate_resolver 2>&1 | tail -5
```
Expected: compile error.

- [ ] **Step 3: Implement resolver**

Create `src/coordinate_resolver/resolve.rs`:
```rust
use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, DominantBaseline, FontFamily, FontWeight,
    TextAnchor,
};
use crate::grid_layout::types::{GridContent, GridElement, GridPage, HAlign, VAlign};

const PAGE_MARGIN: f32 = 25.0;

pub fn resolve(pages: &[GridPage]) -> Vec<AbsolutePage> {
    pages.iter().map(resolve_page).collect()
}

fn resolve_page(page: &GridPage) -> AbsolutePage {
    let usable_width = page.width_pt - 2.0 * PAGE_MARGIN;
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = PAGE_MARGIN;

    for row in &page.rows {
        let col_width = row.column_width_pt(usable_width);
        for el in &row.elements {
            let x_start = PAGE_MARGIN + el.column as f32 * col_width;
            let span_width = el.column_span as f32 * col_width;
            let x = match el.halign {
                HAlign::Start => x_start,
                HAlign::Center => x_start + span_width * 0.5,
                HAlign::End => x_start + span_width,
            };
            let y = match el.valign {
                VAlign::Top => row_y,
                VAlign::Center => row_y + row.height_pt * 0.5,
                VAlign::Bottom => row_y + row.height_pt,
            };
            if let Some(content) = grid_to_absolute(&el.content, span_width) {
                elements.push(AbsoluteElement { x, y, content });
            }
        }
        row_y += row.height_pt;
    }

    AbsolutePage {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements,
    }
}

fn grid_to_absolute(content: &GridContent, span_width: f32) -> Option<AbsoluteContent> {
    match content {
        GridContent::NoteHead { pitch, octave, dotted } => Some(AbsoluteContent::NoteHead {
            pitch: pitch.clone(),
            octave: *octave,
            dotted: *dotted,
        }),
        GridContent::Rest { dotted } => Some(AbsoluteContent::Rest { dotted: *dotted }),
        GridContent::NoteDash => Some(AbsoluteContent::Text {
            content: "—".to_string(),
            font_size: 12.0,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        }),
        GridContent::OctaveDot => None, // spacing only; renderer handles via NoteHead.octave
        GridContent::ChordSymbol(s) => Some(AbsoluteContent::ChordSymbol(s.clone())),
        GridContent::Underline { level } => Some(AbsoluteContent::Underline {
            width: span_width,
            level: *level,
        }),
        GridContent::TieOrSlur => Some(AbsoluteContent::TieOrSlur { width: span_width }),
        GridContent::TieOrSlurClose => {
            Some(AbsoluteContent::TieOrSlur { width: span_width * 0.5 })
        }
        GridContent::BarLine { height_pt } => Some(AbsoluteContent::BarLine { height: *height_pt }),
        GridContent::HorizontalLine => {
            Some(AbsoluteContent::HorizontalLine { width: span_width })
        }
        GridContent::RowLabel(s) => Some(AbsoluteContent::Text {
            content: s.clone(),
            font_size: 12.0,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            italic: false,
        }),
        GridContent::LyricSyllable(s) => Some(AbsoluteContent::Lyric(s.clone())),
        GridContent::Bpm(bpm) => Some(AbsoluteContent::Text {
            content: format!("♩={bpm}"),
            font_size: 12.0,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            italic: false,
        }),
        GridContent::TimeSignature { numerator, denominator } => Some(AbsoluteContent::Text {
            content: format!("{numerator}/{denominator}"),
            font_size: 12.0,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            italic: false,
        }),
        GridContent::SectionLabel(s) => Some(AbsoluteContent::Text {
            content: s.clone(),
            font_size: 12.0,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::SansSerif,
            weight: FontWeight::Bold,
            italic: true,
        }),
        GridContent::BarNumber(n) => Some(AbsoluteContent::Text {
            content: n.to_string(),
            font_size: 10.0,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Ideographic,
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            italic: false,
        }),
        GridContent::Text { content, font_size, bold, italic } => {
            Some(AbsoluteContent::Text {
                content: content.clone(),
                font_size: *font_size,
                anchor: TextAnchor::Middle, // halign already applied to x; anchor matches
                baseline: DominantBaseline::Middle,
                font: FontFamily::SansSerif,
                weight: if *bold { FontWeight::Bold } else { FontWeight::Normal },
                italic: *italic,
            })
        }
    }
}
```

**Note on `Text` anchor:** The resolver applies `HAlign` to produce the correct `x`. The `TextAnchor` should then be `Start` for all text (since x is already the left edge). However, to preserve alignment behaviour with the renderer (which uses anchor for final offset), use `Middle` for Center-aligned text and `End` for End-aligned text. Update `grid_to_absolute` for the `Text` variant to pass anchor based on the element's halign.

To do this properly, pass `halign` into `grid_to_absolute`:

Replace the `grid_to_absolute` signature with:
```rust
fn grid_to_absolute(content: &GridContent, span_width: f32, halign: HAlign) -> Option<AbsoluteContent>
```

And in the `Text` arm:
```rust
GridContent::Text { content, font_size, bold, italic } => {
    let anchor = match halign {
        HAlign::Start => TextAnchor::Start,
        HAlign::Center => TextAnchor::Middle,
        HAlign::End => TextAnchor::End,
    };
    Some(AbsoluteContent::Text {
        content: content.clone(),
        font_size: *font_size,
        anchor,
        baseline: DominantBaseline::Middle,
        font: FontFamily::SansSerif,
        weight: if *bold { FontWeight::Bold } else { FontWeight::Normal },
        italic: *italic,
    })
}
```

Update the call site in `resolve_page`:
```rust
if let Some(content) = grid_to_absolute(&el.content, span_width, el.halign) {
```

Also do the same for `RowLabel`: use `TextAnchor::Middle` (it's always `HAlign::Center`).

- [ ] **Step 4: Update `src/coordinate_resolver/mod.rs`**

```rust
pub mod resolve;
pub use resolve::resolve;

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run tests**

```bash
cargo test coordinate_resolver
```
Expected: all 5 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/coordinate_resolver/resolve.rs src/coordinate_resolver/mod.rs src/coordinate_resolver/tests.rs
git commit -m "feat: implement coordinate_resolver::resolve()"
```

---

## Task 8: Wire new pipeline and delete old code

Swap `render_svgs` to use `grid_layout::layout` + `coordinate_resolver::resolve`, then delete the old layout logic and compositor logic.

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/compositor/mod.rs`
- Delete: `src/compositor/header_footer.rs`, `src/compositor/tests.rs`
- Delete: `src/layout/new_layout.rs`, `src/layout/new_types.rs`, `src/layout/tests.rs`
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Update `render_svgs` in `src/lib.rs`**

Replace the `render_svgs` function:
```rust
pub fn render_svgs(score: &Score) -> Vec<String> {
    use layout::new_types::Header;
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };
    let blocks = compiler::compile(score);
    let grid_pages = grid_layout::layout(&blocks, &config, &header, 595.0, 842.0);
    let abs = coordinate_resolver::resolve(&grid_pages);
    let docs = renderer::new_renderer::render_new(&abs, &config);
    serializer::serialize(&docs)
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo check
```
Expected: no errors. (Old modules still compile, `render_svgs` now uses the new path.)

- [ ] **Step 3: Run full test suite**

```bash
cargo test
```
Expected: all tests pass (old compositor tests still run; will be deleted next step).

- [ ] **Step 4: Strip compositor to types-only**

Replace `src/compositor/mod.rs` with:
```rust
pub mod types;
pub use types::*;
```

Delete `src/compositor/header_footer.rs` and `src/compositor/tests.rs`:
```bash
rm src/compositor/header_footer.rs src/compositor/tests.rs
```

- [ ] **Step 5: Delete old layout module content**

```bash
rm src/layout/new_layout.rs src/layout/new_types.rs
```

Remove the tests directory:
```bash
rm -rf src/layout/tests
```

Replace `src/layout/mod.rs` with:
```rust
pub(crate) const PAGE_MARGIN: f32 = 25.0;
```

(Keep PAGE_MARGIN for now in case any code still references `layout::PAGE_MARGIN`; clean up after verifying.)

- [ ] **Step 6: Fix any remaining imports referencing deleted modules**

```bash
cargo check 2>&1 | grep "error"
```

Fix any `use crate::layout::new_types::Header` references in `src/lib.rs` — replace with a local struct or define `Header` directly in `grid_layout`:

In `src/lib.rs`, if `Header` is still referenced from `layout::new_types`, define it inline or re-export it from `grid_layout`. The cleanest fix: move the `Header` struct into `src/grid_layout/types.rs` and update `layout.rs` to use it from there instead of `layout::new_types`.

Update `src/grid_layout/types.rs` — add at the bottom:
```rust
#[derive(Debug, Clone)]
pub struct Header {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
}
```

Update `src/grid_layout/layout.rs` — remove `use crate::layout::new_types::Header;` and replace with:
```rust
use crate::grid_layout::types::Header;
```

Update `src/lib.rs` — replace `use layout::new_types::Header;` with:
```rust
use grid_layout::types::Header;
```

- [ ] **Step 7: Run full test suite**

```bash
cargo test
```
Expected: all tests pass. Count should be ≥ 316 (the baseline) minus compositor tests that were deleted, plus the new grid_layout and coordinate_resolver tests.

- [ ] **Step 8: Verify rendered output looks correct**

```bash
cargo run -- demo.jianpu 2>&1 | tail -5
```
(Or whatever the CLI command is. Verify at least one SVG is produced without panics.)

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat: wire grid_layout + coordinate_resolver pipeline; remove old layout and compositor logic"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** GridPage types ✓ | height table ✓ | system packing ✓ | sub-row expansion ✓ | header/footer rows ✓ | system separator ✓ | decoration rows ✓ | coordinate resolver ✓ | pipeline wiring ✓
- [x] **Pragmatic deviations documented** at top of plan
- [x] **No placeholders** — all code shown in full
- [x] **Type consistency** — `GridRow`, `GridElement`, `GridContent`, `HAlign`, `VAlign` used consistently across tasks 2-8
- [x] **`block_column_width`** defined once in Task 3, referenced in Tasks 5 and 6
- [x] **`LABEL_COLS = 4`** defined once in Task 5, used throughout
- [x] **`Header` struct** — moved to `grid_layout/types.rs` in Task 8 to avoid keeping the old `layout::new_types` module
