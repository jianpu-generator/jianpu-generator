# Pipeline Refactor — Clean 6-Stage Architecture

## Overview

Replace the current 2-function pipeline (`compile` + `render_svgs`) with six stages, each
with a single responsibility and no cross-layer knowledge leakage.

```
Score ──► Vec<MeasureBlock> ──► Vec<Page> ──► Vec<AbsolutePage> ──► Vec<SvgDocument> ──► Vec<String>
         compiler             layout        compositor             renderer              serializer
```

The existing `parser::parse` and `grouper::group` stages are unchanged.

---

## Shared Config

```rust
// src/render_config.rs  (or inlined into lib.rs)
pub struct RenderConfig {
    pub row_height: u32,          // points; controls font sizes and vertical spacing
    pub label_width: u32,         // points; left margin reserved for part labels
    pub note_number_width: u32,   // points; estimated rendered width of one digit
    pub max_columns: u32,         // max logical columns per system before wrapping
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
```

---

## Stage 1 — Measure Compiler

**Module:** `src/compiler/`  
**Input:** `&Score`  
**Output:** `Vec<MeasureBlock>`

Converts each `MultiPartMeasure` in the score into a `MeasureBlock`. This is where all
musical knowledge lives: duration → column position, ditto detection, underline grouping,
tie/slur chain resolution, octave dot placement. The layout engine receives opaque
grid-positioned elements and knows nothing about music.

### Function Signature

```rust
pub fn compile(score: &Score) -> Vec<MeasureBlock>
```

### Types

```rust
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
}

pub struct MeasureRow {
    pub id: RowId,                    // used by layout engine for system compatibility
    pub label: String,                // display text for the row label (e.g. "S", "Alto")
    pub elements: Vec<ColumnElement>,
}

pub struct RowId(pub String);        // typically the part abbreviation

pub struct ColumnElement {
    pub column: u32,                  // 0-indexed within this measure
    pub content: ElementContent,
}

pub enum ElementContent {
    NoteHead { pitch: JianPuPitch, octave: i8, dotted: bool },
    // octave > 0 → dots above; octave < 0 → dots below
    Rest { dotted: bool },
    ChordSymbol(String),
    Underline { from_column: u32, to_column: u32, level: u32 },
    // level 0 = closest to note head (first underline), 1 = second, etc.
    TieOrSlur { from_column: u32, to_column: u32 },
    BarLine,                          // always placed at column = sum of all durations
    Lyric(String),
}

// Rendered above the measure content but do not participate in column alignment.
pub enum Decoration {
    Bpm(u32),
    TimeSignature { numerator: u32, denominator: u32 },
    KeyChange(KeyChange),
    SectionLabel(String),
    BarNumber(u32),
}
```

**Derived property:** `width_in_columns` is not stored — the layout engine derives it as
the column of the `BarLine` element.

---

## Stage 2 — Layout Engine

**Module:** `src/layout/`  
**Input:** `&[MeasureBlock], &RenderConfig, page_width_pt: f32, page_height_pt: f32`  
**Output:** `Vec<Page>`

Pure geometry: breaks `MeasureBlock`s into `System`s (horizontal rows) and `System`s into
`Page`s. Has no musical knowledge. Enforces the system-compatibility invariant: all
`MeasureBlock`s within a `System` must have identical `RowId` sets (same parts, same order).

### Function Signature

```rust
pub fn layout(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
    page_width_pt: f32,
    page_height_pt: f32,
) -> Vec<Page>
```

### Types

```rust
pub struct Page {
    pub header: Header,
    pub footer: Footer,
    pub systems: Vec<System>,
    pub page_width_pt: f32,
    pub page_height_pt: f32,
}

pub struct System {
    pub row_labels: Vec<RowLabel>,    // one per row, same order as MeasureBlock.rows
    pub measures: Vec<MeasureBlock>,  // guaranteed identical RowId sets across all measures
}

pub struct RowLabel {
    pub id: RowId,
    pub text: String,                 // display name (from MeasureRow.label)
}

pub struct Header {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
}

pub struct Footer {
    pub page: u32,
    pub total: u32,
}
```

---

## Stage 3 — Compositor

**Module:** `src/compositor/`  
**Input:** `&[Page], &RenderConfig`  
**Output:** `Vec<AbsolutePage>`

Converts every grid-positioned element to an absolute `(x, y)` point in SVG/PDF space.
Responsibilities:
- Column justification: distributes horizontal space so systems fill `page_width_pt`.
- Row stacking: computes y positions from `config.row_height` and row heights.
- Decoration placement: positions BPM, time signature, key change, section label, bar number
  above each system.
- Header/footer placement: title/author at top, page number at bottom.
- Style resolution: determines `font_size`, `anchor`, `baseline`, `font`, `weight`, `italic`
  for every `Text` element so the SVG renderer has no font decisions to make.

### Function Signature

```rust
pub fn compose(pages: &[Page], config: &RenderConfig) -> Vec<AbsolutePage>
```

### Types

```rust
pub struct AbsolutePage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<AbsoluteElement>,
}

pub struct AbsoluteElement {
    pub x: f32,
    pub y: f32,
    pub content: AbsoluteContent,
}

pub enum AbsoluteContent {
    NoteHead { pitch: JianPuPitch, octave: i8, dotted: bool },
    Rest { dotted: bool },
    ChordSymbol(String),
    Underline { width: f32, level: u32 },   // x is the left edge
    TieOrSlur { width: f32 },               // x is the left anchor; arc is symmetric
    BarLine { height: f32 },                // x is the line x; vertical line downward
    Lyric(String),
    Text {                                  // titles, labels, page numbers, decorations
        content: String,
        font_size: f32,
        anchor: TextAnchor,
        baseline: DominantBaseline,
        font: FontFamily,
        weight: FontWeight,
        italic: bool,
    },
}

pub enum TextAnchor      { Start, Middle, End }
pub enum DominantBaseline { Middle, Hanging, Ideographic }
pub enum FontFamily       { Monospace, SansSerif }
pub enum FontWeight       { Normal, Bold }
```

---

## Stage 4 — SVG Renderer

**Module:** `src/renderer/`  
**Input:** `&[AbsolutePage]`  
**Output:** `Vec<SvgDocument>`

Converts each `AbsolutePage` into a typed SVG AST. No layout decisions, no styling
decisions — every measurement and style is already in the input. This stage is a pure
mechanical translation from semantic content types to SVG primitive types.

`NoteHead` expands into multiple `SvgElement`s (one `Text` for the digit, `Circle`s for
octave dots, an optional `Circle` for the dotted-note dot). All other variants map 1-to-1.

### Function Signature

```rust
pub fn render(pages: &[AbsolutePage]) -> Vec<SvgDocument>
```

### Types

```rust
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
        // Quadratic bezier: "M x y Q cx cy x2 y2"
        // x/y come from SvgElement.x/y; only the control and end points vary.
        control_x: f32,
        control_y: f32,
        end_x: f32,
        end_y: f32,
        stroke_width: f32,
    },
}
```

Note: `TextAnchor`, `DominantBaseline`, `FontFamily`, `FontWeight` are re-exported from
`compositor` (same types, shared module).

---

## Stage 5 — SVG Serializer

**Module:** `src/serializer/`  
**Input:** `&[SvgDocument]`  
**Output:** `Vec<String>`

Converts each `SvgDocument` to a valid SVG XML string. Pure string formatting — no
geometry, no style decisions. One string per document (= one page).

### Function Signature

```rust
pub fn serialize(documents: &[SvgDocument]) -> Vec<String>
```

Each `SvgElement` maps to one XML element:

| `SvgKind`  | XML element |
|------------|-------------|
| `Text`     | `<text>`    |
| `Line`     | `<line>`    |
| `Circle`   | `<circle>`  |
| `Path`     | `<path>`    |

---

## Updated `lib.rs` Entry Points

```rust
pub fn render_svgs(score: &Score) -> Vec<String> {
    let config = RenderConfig::from_metadata(&score.metadata);
    let blocks = compiler::compile(score);
    let pages  = layout::layout(&blocks, &config, 595.0, 842.0);
    let abs    = compositor::compose(&pages, &config);
    let docs   = renderer::render(&abs);
    serializer::serialize(&docs)
}
```

---

## Migration Notes

- `src/layout/mod.rs` — keep only pure row-breaking logic; remove `event_duration`,
  `measure_beat_width`, `measure_column_width`, `format_chord_symbol`, `part_row_height`,
  all chain/beam logic → move to `src/compiler/`.
- `src/renderer/mod.rs` — remove all `format!` SVG string building; replace with
  `SvgElement` construction → string formatting moves to `src/serializer/`.
- `src/layout/types.rs` — `Page`, `RowGroup`, `GridElement`, `GridContent` are replaced by
  the new types above; delete after migration.
- Existing `renderer::render` signature changes; update all call sites in `lib.rs` and tests.
