# Pipeline Refactor — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 2-function pipeline (`compile` + `render_svgs`) with six clean stages (compiler → layout → compositor → renderer → serializer), each with a single responsibility.

**Architecture:** New modules `src/compiler/`, `src/compositor/`, and `src/serializer/` are added alongside refactored `src/layout/` and `src/renderer/`. The new pipeline is built in parallel with the old one, then `lib.rs` is switched to the new pipeline, and old code is deleted.

**Tech Stack:** Rust, existing `src/ast/grouped::Score`, `src/ast/parsed::*` types.

---

## Pre-flight

Before starting, check that all tests pass:

```bash
cargo test
```

If anything fails, stop and fix it first.

---

### Task 1: RenderConfig

**Files:**
- Create: `src/render_config.rs`
- Modify: `src/lib.rs` (add `pub mod render_config;`)
- Modify: `src/ast/grouped.rs` — read to understand `Metadata` fields

- [ ] **Step 1: Check `Metadata` fields used in `render_svgs`**

Read `src/lib.rs:42-47` and `src/ast/grouped.rs`. Note: current `render_svgs` uses `score.metadata.row_height` and `score.metadata.note_number_width`. The spec adds `label_width`, `note_number_width`, `max_columns` — check if those exist on `Metadata`.

Run:
```bash
grep -n "row_height\|label_width\|note_number_width\|max_columns" src/ast/grouped.rs
```

- [ ] **Step 2: Write failing test for `RenderConfig::from_metadata`**

Create `src/render_config.rs`:

```rust
use crate::ast::grouped::Metadata;

pub struct RenderConfig {
    pub row_height: u32,
    pub label_width: u32,
    pub note_number_width: u32,
    pub max_columns: u32,
}

impl RenderConfig {
    pub fn from_metadata(meta: &Metadata) -> Self {
        RenderConfig {
            row_height: meta.row_height,
            label_width: meta.label_width,
            note_number_width: meta.note_number_width,
            max_columns: meta.max_columns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_metadata_copies_fields() {
        let meta = Metadata {
            row_height: 30,
            label_width: 20,
            note_number_width: 12,
            max_columns: 48,
            ..Metadata::default()
        };
        let cfg = RenderConfig::from_metadata(&meta);
        assert_eq!(cfg.row_height, 30);
        assert_eq!(cfg.label_width, 20);
        assert_eq!(cfg.note_number_width, 12);
        assert_eq!(cfg.max_columns, 48);
    }
}
```

> **Note:** If `Metadata` is missing `label_width` or `max_columns` fields (from the grep above), add them with sensible defaults (e.g. `label_width: u32 = 0`, `max_columns: u32 = 48`) to `Metadata` in `src/ast/grouped.rs` first.

- [ ] **Step 3: Run test to verify it fails (compile error or test failure)**

```bash
cargo test render_config
```

- [ ] **Step 4: Add `pub mod render_config;` to `src/lib.rs`**

Insert after the existing `pub mod renderer;` line in `src/lib.rs`.

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test render_config
```

Expected: `test render_config::tests::from_metadata_copies_fields ... ok`

- [ ] **Step 6: Commit**

```bash
git add src/render_config.rs src/lib.rs src/ast/grouped.rs
git commit -m "feat: add RenderConfig struct"
```

---

### Task 2: Compiler Types

**Files:**
- Create: `src/compiler/mod.rs`
- Create: `src/compiler/types.rs`
- Modify: `src/lib.rs` (add `pub mod compiler;`)

- [ ] **Step 1: Write test asserting the types compile**

Create `src/compiler/types.rs`:

```rust
use crate::ast::parsed::JianPuPitch;

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRow {
    pub id: RowId,
    pub label: String,
    pub elements: Vec<ColumnElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnElement {
    pub column: u32,
    pub content: ElementContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContent {
    NoteHead { pitch: JianPuPitch, octave: i8, dotted: bool },
    Rest { dotted: bool },
    ChordSymbol(String),
    Underline { from_column: u32, to_column: u32, level: u32 },
    TieOrSlur { from_column: u32, to_column: u32 },
    BarLine,
    Lyric(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decoration {
    Bpm(u32),
    TimeSignature { numerator: u32, denominator: u32 },
    SectionLabel(String),
    BarNumber(u32),
}
```

Create `src/compiler/mod.rs`:

```rust
pub mod types;

pub use types::*;

use crate::ast::grouped::Score;

pub fn compile(_score: &Score) -> Vec<MeasureBlock> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::types::*;

    #[test]
    fn types_are_constructible() {
        let _block = MeasureBlock {
            rows: vec![MeasureRow {
                id: RowId("S".to_string()),
                label: "Soprano".to_string(),
                elements: vec![ColumnElement {
                    column: 0,
                    content: ElementContent::Rest { dotted: false },
                }],
            }],
            decorations: vec![Decoration::Bpm(120)],
        };
    }
}
```

- [ ] **Step 2: Add `pub mod compiler;` to `src/lib.rs`**

Insert after `pub mod combiner;`.

- [ ] **Step 3: Run test**

```bash
cargo test compiler::tests
```

Expected: `test compiler::tests::types_are_constructible ... ok`

- [ ] **Step 4: Commit**

```bash
git add src/compiler/mod.rs src/compiler/types.rs src/lib.rs
git commit -m "feat: add compiler module types"
```

---

### Task 3: Compiler Implementation

**Files:**
- Modify: `src/compiler/mod.rs` (implement `compile`)
- Create: `src/compiler/tests.rs`

This task extracts musical knowledge currently in `src/layout/mod.rs` into the new compiler. Read `src/layout/mod.rs` carefully before implementing.

The compiler converts each `MultiPartMeasure` in `score.measures` into a `MeasureBlock`. Each `PartRow` becomes a `MeasureRow`. Musical knowledge (duration → column, underlines, chord symbols, ties/slurs, octave dots, bar line placement) lives here.

- [ ] **Step 1: Read the current layout to understand what to extract**

```bash
grep -n "fn \|pub fn " src/layout/mod.rs src/layout/part_emit.rs
```

Also read `src/ast/grouped.rs` to understand `Score`, `MultiPartMeasure`, `PartRow`, `PartSlice`, `NoteEvent`.

- [ ] **Step 2: Write failing tests for `compile`**

Create `src/compiler/tests.rs`. Reference `src/layout/tests/` for how test helpers and fixtures work in this codebase (read them first with `cat -n src/layout/tests/notes.rs`).

```rust
use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test").unwrap();
    group(doc).unwrap()
}

#[test]
fn single_quarter_note_produces_one_note_head_element() {
    let score = score_from(
        "[parts]\nS = Soprano\n[score]\n1 |",
    );
    let blocks = compile(&score);
    assert!(!blocks.is_empty(), "should produce at least one block");
    let block = &blocks[0];
    assert!(!block.rows.is_empty());
    let row = &block.rows[0];
    let note_heads: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteHead { .. }))
        .collect();
    assert_eq!(note_heads.len(), 1);
}

#[test]
fn bar_line_is_last_element_in_row() {
    let score = score_from("[parts]\nS = Soprano\n[score]\n1 |");
    let blocks = compile(&score);
    let row = &blocks[0].rows[0];
    let last = row.elements.last().unwrap();
    assert_eq!(last.content, ElementContent::BarLine);
}

#[test]
fn bpm_appears_as_decoration_on_first_block() {
    let score = score_from("[parts]\nS = Soprano\n[meta]\nbpm = 100\n[score]\n1 |");
    let blocks = compile(&score);
    let has_bpm = blocks[0]
        .decorations
        .iter()
        .any(|d| matches!(d, Decoration::Bpm(100)));
    assert!(has_bpm, "first block should have BPM decoration");
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test compiler::tests
```

Expected: all 3 tests fail (compile returns empty vec).

- [ ] **Step 4: Implement `compile`**

In `src/compiler/mod.rs`, implement the full `compile` function. Key logic to port:

**Duration to column:** Look at `measure_beat_width` and `event_duration` in `src/layout/mod.rs`. Each note/rest/chord event occupies `event.duration` columns. Column of event N = sum of durations of events 0..N.

**BarLine column:** column = total duration of the measure (sum of all event durations in any part — they must be equal).

**Underlines (beams):** Look at `flush_beam_buffer` and `compute_underline_levels` in `src/layout/mod.rs`. Notes with `underline_count > 0` form beam groups. Use `Underline { from_column, to_column, level }` (level 0 = first underline closest to note heads).

**Chord symbol formatting:** Port `format_chord_symbol` from `src/layout/mod.rs`.

**Octave dots:** Notes with `octave != 0` produce `NoteHead` elements with `octave` field. The renderer will draw dots above (octave > 0) or below (octave < 0). No separate element needed — embedded in `NoteHead`.

**Ties/slurs:** Port the slur chain logic from `src/layout/mod.rs`. Look for `SlurKey` and slur tracking. Produce `TieOrSlur { from_column, to_column }` elements.

**Lyrics:** Produce `Lyric(text)` elements at the correct column.

**Decorations:** BPM, TimeSignature, SectionLabel, BarNumber come from `score.metadata` (first block only) and `measure.directives`.

**Row label:** Use `part.name()` or abbreviation for `RowId` and `label`.

Add `mod tests;` to `src/compiler/mod.rs`.

- [ ] **Step 5: Run tests**

```bash
cargo test compiler::tests
```

Expected: all 3 pass.

- [ ] **Step 6: Run full test suite**

```bash
cargo test
```

No regressions allowed. Fix any that appear.

- [ ] **Step 7: Commit**

```bash
git add src/compiler/mod.rs src/compiler/tests.rs
git commit -m "feat: implement compiler stage (music knowledge extraction)"
```

---

### Task 4: New Layout Types

**Files:**
- Create: `src/layout/new_types.rs`

The new `Page`/`System`/`RowLabel`/`Header`/`Footer` types (spec §Stage 2). These coexist with the old `types.rs` until Task 8.

- [ ] **Step 1: Write the new layout types**

Create `src/layout/new_types.rs`:

```rust
use crate::compiler::types::{MeasureBlock, RowId};

#[derive(Debug, Clone)]
pub struct Page {
    pub header: Header,
    pub footer: Footer,
    pub systems: Vec<System>,
    pub page_width_pt: f32,
    pub page_height_pt: f32,
}

#[derive(Debug, Clone)]
pub struct System {
    pub row_labels: Vec<RowLabel>,
    pub measures: Vec<MeasureBlock>,
}

#[derive(Debug, Clone)]
pub struct RowLabel {
    pub id: RowId,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
}

#[derive(Debug, Clone)]
pub struct Footer {
    pub page: u32,
    pub total: u32,
}
```

Add to `src/layout/mod.rs`:

```rust
pub mod new_types;
```

- [ ] **Step 2: Run check**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add src/layout/new_types.rs src/layout/mod.rs
git commit -m "feat: add new layout types (Page, System, RowLabel)"
```

---

### Task 5: New Layout Engine

**Files:**
- Create: `src/layout/new_layout.rs`

Implements `layout_new(blocks, config, page_width_pt, page_height_pt) -> Vec<new_types::Page>`. Pure geometry: breaks `MeasureBlock`s into `System`s and `System`s into `Page`s. No musical knowledge.

Key rules from spec:
- System-compatibility: all `MeasureBlock`s in a `System` must have identical `RowId` sets (same parts, same order).
- Column budget: a system fits measures while the total column count ≤ `config.max_columns` (or fits page width — use whichever the old engine uses; read `src/layout/layout_engine.rs` to see how the old row-breaking works).
- Header/footer: page 1 has the full header (title, subtitle, author); subsequent pages may repeat or omit (follow old behavior from `src/layout/layout_engine.rs`).

- [ ] **Step 1: Read old layout engine to understand row-breaking logic**

```bash
cat -n src/layout/layout_engine.rs | head -100
```

Note: the old engine decides how many measures fit per system. Replicate that logic.

- [ ] **Step 2: Write failing test**

Create `src/layout/tests/new_layout.rs`:

```rust
use crate::layout::new_layout::layout_new;
use crate::compiler::types::{MeasureBlock, MeasureRow, RowId, ColumnElement, ElementContent, Decoration};
use crate::render_config::RenderConfig;

fn make_block(row_id: &str, col_count: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![ColumnElement {
                column: col_count - 1,
                content: ElementContent::BarLine,
            }],
        }],
        decorations: vec![],
    }
}

fn config() -> RenderConfig {
    RenderConfig { row_height: 30, label_width: 20, note_number_width: 12, max_columns: 16 }
}

#[test]
fn two_blocks_fit_on_one_page() {
    let blocks = vec![make_block("S", 4), make_block("S", 4)];
    let pages = layout_new(&blocks, &config(), 595.0, 842.0);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].systems.iter().map(|s| s.measures.len()).sum::<usize>(), 2);
}

#[test]
fn page_footer_has_correct_totals() {
    let blocks: Vec<_> = (0..20).map(|_| make_block("S", 16)).collect();
    let pages = layout_new(&blocks, &config(), 595.0, 842.0);
    let total = pages.len() as u32;
    for (i, page) in pages.iter().enumerate() {
        assert_eq!(page.footer.total, total);
        assert_eq!(page.footer.page, i as u32 + 1);
    }
}
```

Add `mod new_layout;` to `src/layout/tests/mod.rs`.

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test layout::tests::new_layout
```

- [ ] **Step 4: Implement `layout_new`**

Create `src/layout/new_layout.rs`. Port the page-breaking logic from `src/layout/layout_engine.rs`, adapted to accept `&[MeasureBlock]` and produce `Vec<new_types::Page>`.

Column width of a block = column of its `BarLine` element (find the element with `content == ElementContent::BarLine` in any row; they all have the same column count).

```rust
use crate::compiler::types::{MeasureBlock, ElementContent};
use crate::layout::new_types::{Footer, Header, Page, RowLabel, System};
use crate::render_config::RenderConfig;

pub fn layout_new(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
    page_width_pt: f32,
    page_height_pt: f32,
) -> Vec<Page> {
    // ... implementation
}

fn block_column_width(block: &MeasureBlock) -> u32 {
    block
        .rows
        .first()
        .and_then(|row| {
            row.elements
                .iter()
                .find(|e| e.content == ElementContent::BarLine)
                .map(|e| e.column + 1)
        })
        .unwrap_or(1)
}
```

Add `pub mod new_layout;` to `src/layout/mod.rs`.

- [ ] **Step 5: Run tests**

```bash
cargo test layout::tests::new_layout
```

- [ ] **Step 6: Run full suite**

```bash
cargo test
```

- [ ] **Step 7: Commit**

```bash
git add src/layout/new_layout.rs src/layout/mod.rs src/layout/tests/new_layout.rs src/layout/tests/mod.rs
git commit -m "feat: implement new layout engine (pure geometry)"
```

---

### Task 6: Compositor Types

**Files:**
- Create: `src/compositor/mod.rs`
- Create: `src/compositor/types.rs`
- Modify: `src/lib.rs` (add `pub mod compositor;`)

- [ ] **Step 1: Write the compositor types**

Create `src/compositor/types.rs`:

```rust
#[derive(Debug, Clone)]
pub struct AbsolutePage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<AbsoluteElement>,
}

#[derive(Debug, Clone)]
pub struct AbsoluteElement {
    pub x: f32,
    pub y: f32,
    pub content: AbsoluteContent,
}

#[derive(Debug, Clone)]
pub enum AbsoluteContent {
    NoteHead { pitch: crate::ast::parsed::JianPuPitch, octave: i8, dotted: bool },
    Rest { dotted: bool },
    ChordSymbol(String),
    Underline { width: f32, level: u32 },
    TieOrSlur { width: f32 },
    BarLine { height: f32 },
    Lyric(String),
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchor,
        baseline: DominantBaseline,
        font: FontFamily,
        weight: FontWeight,
        italic: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAnchor { Start, Middle, End }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DominantBaseline { Middle, Hanging, Ideographic }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontFamily { Monospace, SansSerif }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight { Normal, Bold }
```

Create `src/compositor/mod.rs`:

```rust
pub mod types;
pub use types::*;

use crate::layout::new_types::Page;
use crate::render_config::RenderConfig;

pub fn compose(_pages: &[Page], _config: &RenderConfig) -> Vec<AbsolutePage> {
    vec![]
}
```

Add `pub mod compositor;` to `src/lib.rs`.

- [ ] **Step 2: Run check**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add src/compositor/mod.rs src/compositor/types.rs src/lib.rs
git commit -m "feat: add compositor module types"
```

---

### Task 7: Compositor Implementation

**Files:**
- Modify: `src/compositor/mod.rs`
- Create: `src/compositor/tests.rs`

The compositor converts grid-positioned elements to absolute `(x, y)` coordinates. Port the geometry logic from `src/renderer/mod.rs`.

Key responsibilities:
- **Column justification:** `column_width = (page_width_pt - 2*PAGE_MARGIN - config.label_width) / total_columns_in_system`
- **Row stacking:** y position of row N = `PAGE_MARGIN + header_height + system_offset + row_n * config.row_height`
- **Decoration placement:** BPM, TimeSignature, SectionLabel, BarNumber go above each system's first row
- **Header/footer:** title/author at top, page number at bottom — derive positions from `config.row_height`
- **Style resolution:** Determine `font_size`, `anchor`, `baseline`, `font`, `weight`, `italic` for every `Text` element

Read `src/renderer/mod.rs` lines 1-350 for the geometry formulas.

- [ ] **Step 1: Write failing tests**

Create `src/compositor/tests.rs`:

```rust
use super::*;
use crate::layout::new_types::{Footer, Header, Page, System};
use crate::render_config::RenderConfig;
use crate::compiler::types::{MeasureBlock, MeasureRow, RowId, ColumnElement, ElementContent, Decoration};

fn cfg() -> RenderConfig {
    RenderConfig { row_height: 30, label_width: 0, note_number_width: 12, max_columns: 16 }
}

fn empty_page() -> Page {
    Page {
        header: Header { title: "T".to_string(), subtitle: None, author: "A".to_string() },
        footer: Footer { page: 1, total: 1 },
        systems: vec![System {
            row_labels: vec![],
            measures: vec![MeasureBlock {
                rows: vec![MeasureRow {
                    id: RowId("S".to_string()),
                    label: "S".to_string(),
                    elements: vec![
                        ColumnElement { column: 0, content: ElementContent::NoteHead {
                            pitch: crate::ast::parsed::JianPuPitch::One,
                            octave: 0,
                            dotted: false,
                        }},
                        ColumnElement { column: 1, content: ElementContent::BarLine },
                    ],
                }],
                decorations: vec![],
            }],
        }],
        page_width_pt: 595.0,
        page_height_pt: 842.0,
    }
}

#[test]
fn compose_produces_one_page_per_input_page() {
    let pages = compose(&[empty_page()], &cfg());
    assert_eq!(pages.len(), 1);
}

#[test]
fn note_head_element_has_positive_x_and_y() {
    let pages = compose(&[empty_page()], &cfg());
    let abs = &pages[0];
    let note = abs.elements.iter().find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }));
    let note = note.expect("should have a NoteHead");
    assert!(note.x > 0.0, "x should be positive, got {}", note.x);
    assert!(note.y > 0.0, "y should be positive, got {}", note.y);
}
```

Add `mod tests;` to `src/compositor/mod.rs`.

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test compositor::tests
```

- [ ] **Step 3: Implement `compose`**

In `src/compositor/mod.rs`. Port geometry from `src/renderer/mod.rs`:

```rust
const PAGE_MARGIN: f32 = 25.0;

pub fn compose(pages: &[Page], config: &RenderConfig) -> Vec<AbsolutePage> {
    pages.iter().map(|p| compose_page(p, config)).collect()
}

fn compose_page(page: &Page, config: &RenderConfig) -> AbsolutePage {
    let mut elements = Vec::new();
    emit_header(page, config, &mut elements);
    emit_systems(page, config, &mut elements);
    emit_footer(page, config, &mut elements);
    AbsolutePage {
        width_pt: page.page_width_pt,
        height_pt: page.page_height_pt,
        elements,
    }
}
```

Key formula for column width in a system (parallel to `render_row_groups` in renderer):
```rust
let usable_width = page.page_width_pt - 2.0 * PAGE_MARGIN - config.label_width as f32;
let total_columns: u32 = system.measures.iter().map(|b| block_column_width(b)).sum();
let column_width = usable_width / total_columns as f32;
```

For each `ColumnElement` in each measure row, compute absolute `(x, y)`:
- `x = PAGE_MARGIN + config.label_width + (measure_start_column + element.column) * column_width`
- `y = PAGE_MARGIN + header_height + system_y_offset + row_index * config.row_height`

Style decisions (port from renderer):
- NoteHead text: `font_size = config.row_height * 0.6`, monospace, anchor middle, baseline middle
- Lyric: sans-serif, `font_size = row_height * 0.6 * 1.2` for CJK (check `is_cjk` by checking if any char is in CJK unicode range `\u{4E00}..=\u{9FFF}`)
- Part labels: sans-serif, small
- Title: `font_size = row_height * 1.5`, center-anchored
- Author/subtitle: `font_size = row_height * 0.8`

- [ ] **Step 4: Run tests**

```bash
cargo test compositor::tests
```

- [ ] **Step 5: Run full suite**

```bash
cargo test
```

- [ ] **Step 6: Commit**

```bash
git add src/compositor/mod.rs src/compositor/tests.rs
git commit -m "feat: implement compositor (absolute positioning + style resolution)"
```

---

### Task 7b: Compositor — CJK Lyric Detection Helper

The compositor needs to decide if a lyric string is CJK to pick font size. Extract this as a small helper.

- [ ] **Step 1: Write failing test**

In `src/compositor/tests.rs`, add:

```rust
use super::is_cjk;

#[test]
fn cjk_detection() {
    assert!(is_cjk("你好"));
    assert!(!is_cjk("hello"));
    assert!(is_cjk("一"));
}
```

- [ ] **Step 2: Implement `is_cjk`**

In `src/compositor/mod.rs`:

```rust
pub(crate) fn is_cjk(s: &str) -> bool {
    s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c))
}
```

- [ ] **Step 3: Run test**

```bash
cargo test compositor::tests::cjk_detection
```

- [ ] **Step 4: Commit**

```bash
git add src/compositor/mod.rs src/compositor/tests.rs
git commit -m "feat: add CJK detection helper in compositor"
```

---

### Task 8: Renderer New Types

**Files:**
- Create: `src/renderer/new_types.rs`

- [ ] **Step 1: Write the renderer types**

Create `src/renderer/new_types.rs`:

```rust
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};

pub struct SvgDocument {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<SvgElement>,
}

pub struct SvgElement {
    pub x: f32,
    pub y: f32,
    pub kind: SvgKind,
}

pub enum SvgKind {
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchor,
        baseline: DominantBaseline,
        font: FontFamily,
        weight: FontWeight,
        italic: bool,
    },
    Line {
        x2: f32,
        y2: f32,
        stroke_width: f32,
    },
    Circle {
        r: f32,
    },
    Path {
        control_x: f32,
        control_y: f32,
        end_x: f32,
        end_y: f32,
        stroke_width: f32,
    },
}
```

Add `pub mod new_types;` to `src/renderer/mod.rs`.

- [ ] **Step 2: Run check**

```bash
cargo check
```

- [ ] **Step 3: Commit**

```bash
git add src/renderer/new_types.rs src/renderer/mod.rs
git commit -m "feat: add SVG AST types to renderer"
```

---

### Task 9: New Renderer Implementation

**Files:**
- Create: `src/renderer/new_renderer.rs`
- Create: `src/renderer/new_tests.rs`

Implements `render_new(pages: &[AbsolutePage]) -> Vec<SvgDocument>`. Pure mechanical translation from semantic content to SVG primitives.

Key translations:
- `AbsoluteContent::NoteHead { pitch, octave, dotted }` → multiple `SvgElement`s:
  - `SvgKind::Text` for the digit (pitch → '1'..'7'; '0' for rest)
  - For `octave > 0`: `octave` `SvgKind::Circle` elements above the note
  - For `octave < 0`: `|octave|` `SvgKind::Circle` elements below the note
  - For `dotted`: one `SvgKind::Circle` to the right of the digit
- `AbsoluteContent::Rest { dotted }` → `SvgKind::Text` ("0") + optional dot circle
- `AbsoluteContent::ChordSymbol(s)` → `SvgKind::Text`
- `AbsoluteContent::Underline { width, level }` → `SvgKind::Line` (x to x+width)
- `AbsoluteContent::TieOrSlur { width }` → `SvgKind::Path` (quadratic bezier arc)
- `AbsoluteContent::BarLine { height }` → `SvgKind::Line` (vertical)
- `AbsoluteContent::Lyric(s)` → `SvgKind::Text`
- `AbsoluteContent::Text { .. }` → `SvgKind::Text`

- [ ] **Step 1: Write failing tests**

Create `src/renderer/new_tests.rs`:

```rust
use crate::renderer::new_renderer::render_new;
use crate::compositor::types::{AbsolutePage, AbsoluteElement, AbsoluteContent, TextAnchor, DominantBaseline, FontFamily, FontWeight};
use crate::renderer::new_types::SvgKind;
use crate::ast::parsed::JianPuPitch;

fn make_page(content: AbsoluteContent) -> AbsolutePage {
    AbsolutePage {
        width_pt: 595.0,
        height_pt: 842.0,
        elements: vec![AbsoluteElement { x: 100.0, y: 200.0, content }],
    }
}

#[test]
fn note_head_produces_text_element() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One, octave: 0, dotted: false,
    });
    let docs = render_new(&[page]);
    assert_eq!(docs.len(), 1);
    let has_text = docs[0].elements.iter().any(|e| matches!(e.kind, SvgKind::Text { .. }));
    assert!(has_text);
}

#[test]
fn bar_line_produces_line_element() {
    let page = make_page(AbsoluteContent::BarLine { height: 60.0 });
    let docs = render_new(&[page]);
    let has_line = docs[0].elements.iter().any(|e| matches!(e.kind, SvgKind::Line { .. }));
    assert!(has_line);
}

#[test]
fn tie_produces_path_element() {
    let page = make_page(AbsoluteContent::TieOrSlur { width: 40.0 });
    let docs = render_new(&[page]);
    let has_path = docs[0].elements.iter().any(|e| matches!(e.kind, SvgKind::Path { .. }));
    assert!(has_path);
}
```

Add `mod new_tests;` to `src/renderer/mod.rs` (behind `#[cfg(test)]`).

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test renderer::new_tests
```

- [ ] **Step 3: Implement `render_new`**

Create `src/renderer/new_renderer.rs`:

```rust
use crate::ast::parsed::JianPuPitch;
use crate::compositor::types::{AbsoluteContent, AbsolutePage};
use crate::renderer::new_types::{SvgDocument, SvgElement, SvgKind};

pub fn render_new(pages: &[AbsolutePage]) -> Vec<SvgDocument> {
    pages.iter().map(render_page).collect()
}

fn render_page(page: &AbsolutePage) -> SvgDocument {
    let mut elements = Vec::new();
    for abs_el in &page.elements {
        emit_elements(abs_el.x, abs_el.y, &abs_el.content, &mut elements);
    }
    SvgDocument {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements,
    }
}

fn emit_elements(x: f32, y: f32, content: &AbsoluteContent, out: &mut Vec<SvgElement>) {
    match content {
        AbsoluteContent::NoteHead { pitch, octave, dotted } => {
            let digit = pitch_digit(*pitch);
            // ... push Text, Circle for octave dots, Circle for dotted
        }
        AbsoluteContent::Rest { dotted } => {
            // push Text "0"; optional dot circle
        }
        AbsoluteContent::BarLine { height } => {
            out.push(SvgElement {
                x, y,
                kind: SvgKind::Line { x2: x, y2: y + height, stroke_width: 1.0 },
            });
        }
        AbsoluteContent::TieOrSlur { width } => {
            let cx = x + width / 2.0;
            let cy = y - 8.0; // control point above the tie
            out.push(SvgElement {
                x, y,
                kind: SvgKind::Path {
                    control_x: cx,
                    control_y: cy,
                    end_x: x + width,
                    end_y: y,
                    stroke_width: 1.5,
                },
            });
        }
        AbsoluteContent::Underline { width, .. } => {
            out.push(SvgElement {
                x, y,
                kind: SvgKind::Line { x2: x + width, y2: y, stroke_width: 1.0 },
            });
        }
        AbsoluteContent::ChordSymbol(s) | AbsoluteContent::Lyric(s) => {
            // push Text
        }
        AbsoluteContent::Text { content, font_size, anchor, baseline, font, weight, italic } => {
            out.push(SvgElement {
                x, y,
                kind: SvgKind::Text {
                    content: content.clone(),
                    font_size: *font_size,
                    anchor: *anchor,
                    baseline: *baseline,
                    font: *font,
                    weight: *weight,
                    italic: *italic,
                },
            });
        }
    }
}

fn pitch_digit(pitch: JianPuPitch) -> &'static str {
    match pitch {
        JianPuPitch::One => "1",
        JianPuPitch::Two => "2",
        JianPuPitch::Three => "3",
        JianPuPitch::Four => "4",
        JianPuPitch::Five => "5",
        JianPuPitch::Six => "6",
        JianPuPitch::Seven => "7",
    }
}
```

Add `pub mod new_renderer;` to `src/renderer/mod.rs`.

- [ ] **Step 4: Run tests**

```bash
cargo test renderer::new_tests
```

- [ ] **Step 5: Commit**

```bash
git add src/renderer/new_renderer.rs src/renderer/new_types.rs src/renderer/mod.rs src/renderer/new_tests.rs
git commit -m "feat: implement new renderer (SVG AST translation)"
```

---

### Task 10: Serializer

**Files:**
- Create: `src/serializer/mod.rs`
- Modify: `src/lib.rs` (add `pub mod serializer;`)

Converts each `SvgDocument` to a valid SVG XML string. Pure string formatting.

- [ ] **Step 1: Write failing tests**

Create `src/serializer/mod.rs` with stub + tests:

```rust
use crate::renderer::new_types::{SvgDocument, SvgElement, SvgKind};
use crate::compositor::types::{TextAnchor, DominantBaseline, FontFamily, FontWeight};

pub fn serialize(documents: &[SvgDocument]) -> Vec<String> {
    documents.iter().map(serialize_doc).collect()
}

fn serialize_doc(doc: &SvgDocument) -> String {
    let mut body = String::new();
    for el in &doc.elements {
        serialize_element(el, &mut body);
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}pt" height="{}pt" viewBox="0 0 {} {}">{}</svg>"#,
        doc.width_pt, doc.height_pt, doc.width_pt, doc.height_pt, body
    )
}

fn serialize_element(el: &SvgElement, out: &mut String) {
    match &el.kind {
        SvgKind::Text { content, font_size, anchor, baseline, font, weight, italic } => {
            let anchor_str = match anchor {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            };
            let baseline_str = match baseline {
                DominantBaseline::Middle => "middle",
                DominantBaseline::Hanging => "hanging",
                DominantBaseline::Ideographic => "ideographic",
            };
            let font_str = match font {
                FontFamily::Monospace => "monospace",
                FontFamily::SansSerif => "sans-serif",
            };
            let weight_str = match weight {
                FontWeight::Normal => "normal",
                FontWeight::Bold => "bold",
            };
            let italic_str = if *italic { "italic" } else { "normal" };
            out.push_str(&format!(
                r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family="{}" font-weight="{}" font-style="{}">{}</text>"#,
                el.x, el.y, font_size, anchor_str, baseline_str, font_str, weight_str, italic_str,
                escape_xml(content)
            ));
        }
        SvgKind::Line { x2, y2, stroke_width } => {
            out.push_str(&format!(
                r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="black" stroke-width="{:.1}"/>"#,
                el.x, el.y, x2, y2, stroke_width
            ));
        }
        SvgKind::Circle { r } => {
            out.push_str(&format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="black"/>"#,
                el.x, el.y, r
            ));
        }
        SvgKind::Path { control_x, control_y, end_x, end_y, stroke_width } => {
            out.push_str(&format!(
                r#"<path d="M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}" stroke="black" stroke-width="{:.1}" fill="none"/>"#,
                el.x, el.y, control_x, control_y, end_x, end_y, stroke_width
            ));
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositor::types::{TextAnchor, DominantBaseline, FontFamily, FontWeight};

    fn make_text_doc(content: &str) -> SvgDocument {
        SvgDocument {
            width_pt: 100.0,
            height_pt: 100.0,
            elements: vec![SvgElement {
                x: 10.0, y: 20.0,
                kind: SvgKind::Text {
                    content: content.to_string(),
                    font_size: 12.0,
                    anchor: TextAnchor::Middle,
                    baseline: DominantBaseline::Middle,
                    font: FontFamily::SansSerif,
                    weight: FontWeight::Normal,
                    italic: false,
                },
            }],
        }
    }

    #[test]
    fn produces_valid_svg_wrapper() {
        let docs = serialize(&[make_text_doc("hello")]);
        assert_eq!(docs.len(), 1);
        assert!(docs[0].starts_with("<svg"), "should start with <svg");
        assert!(docs[0].ends_with("</svg>"), "should end with </svg>");
    }

    #[test]
    fn xml_special_chars_are_escaped() {
        let docs = serialize(&[make_text_doc("<b>&\"test\"</b>")]);
        assert!(docs[0].contains("&lt;b&gt;&amp;&quot;test&quot;&lt;/b&gt;"));
    }

    #[test]
    fn circle_element_serializes() {
        let doc = SvgDocument {
            width_pt: 100.0, height_pt: 100.0,
            elements: vec![SvgElement { x: 5.0, y: 5.0, kind: SvgKind::Circle { r: 3.0 } }],
        };
        let result = serialize(&[doc]);
        assert!(result[0].contains("<circle"), "should contain circle element");
    }
}
```

- [ ] **Step 2: Add `pub mod serializer;` to `src/lib.rs`**

- [ ] **Step 3: Run tests**

```bash
cargo test serializer::tests
```

Expected: all 3 pass.

- [ ] **Step 4: Commit**

```bash
git add src/serializer/mod.rs src/lib.rs
git commit -m "feat: implement serializer (SVG string formatting)"
```

---

### Task 11: Wire Up New Pipeline in `lib.rs`

**Files:**
- Modify: `src/lib.rs`

Wire the new pipeline into `render_svgs` and verify all existing integration tests still pass.

- [ ] **Step 1: Update `render_svgs` to use new pipeline**

In `src/lib.rs`, replace the existing `render_svgs`:

```rust
pub fn render_svgs(score: &Score) -> Vec<String> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let blocks = compiler::compile(score);
    let pages = layout::new_layout::layout_new(&blocks, &config, 595.0, 842.0);
    let abs = compositor::compose(&pages, &config);
    let docs = renderer::new_renderer::render_new(&abs);
    serializer::serialize(&docs)
}
```

- [ ] **Step 2: Run all tests**

```bash
cargo test
```

> **Important:** The integration tests in `src/tests/render.rs` compare SVG output. If they fail, the new pipeline's output differs from the old one. Fix the differences in the compositor and renderer until the output matches. The test failures will point to specific rendering differences.

- [ ] **Step 3: Fix any integration test failures**

If `src/tests/render.rs` or `src/tests/ditto.rs` fail, inspect the diff:
```bash
cargo test 2>&1 | head -100
```

Common issues to fix:
- Missing header/footer elements
- Wrong column width calculation
- Missing decorations (BPM, time signature)
- Wrong y positions for rows
- Missing bar numbers or section labels

Fix in the appropriate stage (compositor for positioning, renderer for element generation).

- [ ] **Step 4: Run all tests again (must be green)**

```bash
cargo test
```

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: switch render_svgs to new 6-stage pipeline"
```

---

### Task 12: Cleanup — Delete Old Code

**Files:**
- Delete: old `render` function in `src/renderer/mod.rs` (the one that produces `Vec<String>`)
- Delete or gut: `src/layout/types.rs` old types (`Page`, `RowGroup`, `GridElement`, `GridContent`) — but only after verifying nothing else uses them
- Remove: old `layout_engine.rs` if fully superseded

- [ ] **Step 1: Check what still uses old types**

```bash
grep -rn "GridContent\|GridElement\|RowGroup\|layout::types" src/ --include="*.rs" | grep -v "new_types\|#\[allow"
```

If anything still uses the old types, update it to use new types first.

- [ ] **Step 2: Check what uses old renderer `render` function**

```bash
grep -rn "renderer::render\b" src/ --include="*.rs"
```

If only tests reference it, update the tests to use `render_new`.

- [ ] **Step 3: Delete old code**

Remove the old `pub fn render(pages: &[Page], ...) -> Vec<String>` from `src/renderer/mod.rs`.

Remove or replace `src/layout/types.rs` old types. If `layout/tests/` reference them directly, update those tests to use compiler types or remove them (the layout tests tested the old mixed stage — now they are split between compiler tests and layout tests).

- [ ] **Step 4: Run all tests**

```bash
cargo test
```

Must be green.

- [ ] **Step 5: Run clippy**

```bash
cargo clippy -- -D warnings
```

Fix any warnings.

- [ ] **Step 6: Commit**

```bash
git add -u
git commit -m "refactor: remove old layout/renderer pipeline code"
```

---

## Final Verification

- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy -- -D warnings` — no warnings
- [ ] Render a real `.jianpu` file and visually inspect the output:

```bash
cargo run -- demo.jianpu
```

Open `demo.svg` and verify it looks correct.
