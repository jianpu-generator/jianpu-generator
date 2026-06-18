# Grid Layout Split Design

## Overview

Split the existing `layout` layer into two layers:

1. **Grid Layout** — takes `Vec<MeasureBlock>` and produces `Vec<GridPage>`, a flat column-and-row grid representation with no pixel coordinates.
2. **Coordinate Resolver** — takes `Vec<GridPage>` and produces `Vec<AbsolutePage>`, converting grid positions to pixel coordinates using pure arithmetic.

The existing `compositor` is replaced by the coordinate resolver. The `renderer` and `serializer` layers are unchanged.

### New Pipeline

```
compiler → grid_layout → coordinate_resolver → renderer → serializer
```

Previously:

```
compiler → layout → compositor → renderer → serializer
```

---

## Motivation

The existing `layout` and `compositor` layers are entangled: layout decides page/system/measure packing but leaves coordinate math to compositor, which also re-derives row heights and column widths from raw `MeasureBlock` data. This makes both layers harder to test in isolation.

The grid layout output is a self-contained, inspectable intermediate: all layout decisions are visible as grid positions, and the resolver needs no musical knowledge to produce coordinates.

---

## GridPage Types

These are the output of the grid layout layer and the input to the coordinate resolver.

```rust
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
}

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

pub struct GridElement {
    pub column: u32,
    pub column_span: u32,
    pub halign: HAlign,
    pub valign: VAlign,
    pub content: GridContent,
}

pub enum HAlign {
    Start,
    Center,
    End,
}

pub enum VAlign {
    Top,
    Center,
    Bottom,
}

pub enum GridContent {
    NoteHead { pitch: JianPuPitch, dotted: bool },
    Rest { dotted: bool },
    NoteDash,
    /// Stacked octave dot(s). Whether above or below is implicit from which row it occupies.
    OctaveDot { count: u32 },
    ChordSymbol(String),
    /// Rendered width = column_span × column_width_pt.
    Underline,
    /// Arc. Width = column_span × column_width_pt.
    TieOrSlur,
    /// Closing arc at measure start. Arc runs from left edge to center of column. column_span = 1.
    TieOrSlurClose,
    /// Vertical bar line. Height is baked in by the grid layout layer.
    BarLine { height_pt: f32 },
    /// Full-width horizontal system separator. Width = column_span × column_width_pt.
    HorizontalLine,
    /// Part name shown to the left of the row. Placed at column = 0, column_span = 4.
    RowLabel(String),
    LyricSyllable(String),
    Bpm(u32),
    TimeSignature { numerator: u32, denominator: u32 },
    SectionLabel(String),
    BarNumber(u32),
    /// Generic styled text used for header and footer rows.
    /// Font styling is fully resolved by the grid layout layer.
    Text {
        content: String,
        font_size: f32,
        bold: bool,
        italic: bool,
    },
}
```

---

## Row Structure Per Musical Part

The grid layout layer expands each logical part row from the compiler into a fixed sequence of flat `GridRow`s. All rows within the same system share the same `column_count`.

### Note/Chord Part (6 rows)

| # | Purpose | Height |
|---|---------|--------|
| 1 | Tie/slur arcs | minimal |
| 2 | Above-octave dots | minimal |
| 3 | Note head (main row) | largest |
| 4 | Below-octave dots | minimal |
| 5 | Half-beat underline | minimal |
| 6 | Quarter-beat underline | minimal |

The `RowLabel` element is placed in row 3 (the note head row) at `column=0, column_span=4`.

### Chord-Symbol-Only Part (4 rows)

| # | Purpose | Height |
|---|---------|--------|
| 1 | Tie/slur arcs | minimal |
| 2 | Chord symbol (main row) | largest |
| 3 | Half-beat underline | minimal |
| 4 | Quarter-beat underline | minimal |

No octave dot rows (chord symbols carry no octave notation).

### Lyrics Part (1 row)

| # | Purpose | Height |
|---|---------|--------|
| 1 | Lyric syllables | minimal |

This row immediately follows the 6 rows of its associated note/chord part.

### Decoration Row (1 row, per system when present)

| # | Purpose | Height |
|---|---------|--------|
| 1 | BPM, time signature, section label, bar number | minimal |

Appears as the first row of a system when the opening measure carries any decoration.

### System Separator Row (1 row, between systems)

| # | Purpose | Height |
|---|---------|--------|
| 1 | `HorizontalLine` spanning all columns | minimal |

Contains a single `HorizontalLine` element at `column=0, column_span=column_count`.

### Header Rows (2 rows, at page top)

| # | Purpose | Height |
|---|---------|--------|
| 1 | Title (`Text`, bold, large, `HAlign::Center`) | minimal |
| 2 | Subtitle (`Text`, `HAlign::Center`) + Author (`Text`, `HAlign::End`) — both in the same row | minimal |

Row 2 contains two `GridElement`s sharing the same row: subtitle at `HAlign::Center` and author at `HAlign::End`, both spanning the full column range. The subtitle element is omitted when no subtitle is present, leaving only the author.

### Footer Row (1 row, at page bottom)

| # | Purpose | Height |
|---|---------|--------|
| 1 | Page number (`Text`, e.g. `"1 / 3"`) | minimal |

---

## Row Height Assignment

All row heights are derived from one base constant: `RenderConfig::row_height` (in points, default 30 pt). This is the height reserved for a **main note-head row**. Every other row type is a fixed fraction of this base. The grid layout layer computes every `GridRow::height_pt` before emitting the page; the coordinate resolver never touches height arithmetic.

### Height Table

| Row type | `height_pt` formula | Notes |
|----------|---------------------|-------|
| Tie/slur arc | `row_height × 0.30` | Arc needs vertical room to curve |
| Above-octave dots | `row_height × 0.25` | Small dot, tight spacing |
| **Note head (main)** | `row_height` | Base unit |
| Below-octave dots | `row_height × 0.25` | |
| Half-beat underline | `row_height × 0.15` | Single rendered line |
| Quarter-beat underline | `row_height × 0.15` | |
| Chord tie/slur arc | `row_height × 0.30` | Same rule as note tie/slur |
| **Chord symbol (main)** | `row_height × 0.75` | Slightly shorter than note head |
| Chord half-beat underline | `row_height × 0.15` | |
| Chord quarter-beat underline | `row_height × 0.15` | |
| Lyrics | `row_height × 0.50` | Smaller font, single line |
| Decoration | `row_height × 0.50` | BPM / time signature / section label |
| System separator | `4.0` pt (fixed) | Thin rule + surrounding whitespace |
| Header title | `row_height × 0.80` | Large bold font |
| Header subtitle + author | `row_height × 0.50` | Smaller font, shared row |
| Footer (page number) | `row_height × 0.40` | |

### BarLine height_pt

A `BarLine` element spans every musical sub-row in its system — all six sub-rows of each Note/Chord part and all four sub-rows of each Chord-symbol-only part (lyrics rows are excluded; bar lines do not cross lyric rows). The grid layout layer computes this sum when it emits the `BarLine` element:

```
BarLine.height_pt = Σ height_pt of all note/chord sub-rows in the system
```

The system separator row, decoration row, and header/footer rows are not included in this sum.

---

## Column Layout

The label gutter is absorbed into the column grid. The first 4 columns (`column=0..3`) of each system are reserved for `RowLabel` elements. Musical content (notes, bar lines, decorations) begins at `column=4`.

`usable_width_pt = page_width_pt - 2 × PAGE_MARGIN`

`column_width_pt = usable_width_pt / column_count`

`x = PAGE_MARGIN + column × column_width_pt`  (for `HAlign::Start`)

The resolver adjusts `x` for `HAlign::Center` (+`column_span × column_width_pt × 0.5`) and `HAlign::End` (+`column_span × column_width_pt`).

---

## Coordinate Resolver

The resolver has no musical knowledge. Its only inputs are `Vec<GridPage>` and `usable_width_pt` (derived from `page_width_pt - 2 × PAGE_MARGIN`).

For each page:
1. Walk rows top-to-bottom, tracking `y`.
2. For each element in a row: compute `x` from `column`, `column_span`, `halign`, and `column_width_pt(usable_width_pt)`.
3. Compute `y` from row position and `valign` within `height_pt`.
4. Emit `AbsoluteElement { x, y, content }`.
5. Advance `y += row.height_pt`.

Special cases (purely geometric, no domain knowledge):
- `BarLine { height_pt }`: vertical line of given height downward from `y`.
- `HorizontalLine`: horizontal line of width `column_span × column_width_pt` at `y`.
- `TieOrSlurClose`: arc from `x` (left edge) to `x + column_width_pt × 0.5` (center).

---

## Impact on Existing Code

| Module | Change |
|--------|--------|
| `layout/` | Replaced by `grid_layout/` producing `GridPage` |
| `compositor/` | Replaced by `coordinate_resolver/` consuming `GridPage` |
| `renderer/` | Unchanged |
| `serializer/` | Unchanged |
| `compiler/` | Unchanged |
| `lib.rs` `render_svgs` | Updated pipeline call sites |
