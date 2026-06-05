# Label Directive Design

**Date:** 2026-06-05
**Status:** Approved

## Overview

Add a `label="..."` directive that can appear inside parenthesised directive lines alongside `bpm=`, `key=`, and `time=`. The label is rendered as italic text above the row group where it is declared, left-aligned at the note start column, sitting just above the note heads.

## Syntax

```
(bpm=92 key=C4 time=4/4 label="Verse 1")
(label="Chorus")
```

- The value must be double-quoted.
- Quotes are stripped at parse time.
- An unclosed or missing quote is a parse error.
- `label=` is optional; any directive line may include it alongside or without other directives.

## Data Flow (6 files, in pipeline order)

### 1. `src/ast/parsed.rs`
Add variant:
```rust
ScoreEvent::LabelChange(String)
```

### 2. `src/parser/score/interleaved_parser.rs`
In `parse_directive_line()`, handle tokens matching `label="..."`:
- Strip the `label=` prefix.
- Require the remaining string to start and end with `"`.
- Extract the inner text and emit `ScoreEvent::LabelChange(text)`.
- Error if the quote is not closed.

### 3. `src/ast/grouped.rs`
Add field to `MultiPartMeasure`:
```rust
pub label: Option<String>,
```

### 4. `src/grouper.rs`
When iterating `ScoreEvent`s to build `GroupedMeasure`, detect `LabelChange(text)` and set `label = Some(text)` on the measure being built. Propagate into `MultiPartMeasure.label`.

### 5. `src/layout/types.rs`
Add variant to `GridContent`:
```rust
SectionLabel { text: String },
```

### 6. `src/layout/mod.rs`
When laying out a measure, if `measure.label` is `Some(text)`, emit:
```
GridElement {
    position: GridPosition { column: label_cols, row: current_row_offset + 0 },
    horizontal_alignment: HorizontalAlignment::Left,
    vertical_alignment: VerticalAlignment::Bottom,
    content: GridContent::SectionLabel { text },
}
```
Emitted once per measure (not per part).

### 7. `src/renderer.rs`
Render `SectionLabel` as SVG `<text>`:
- Font size: `base_font_size * 0.7`
- Font style: italic
- `text-anchor="start"`, `dominant-baseline="ideographic"`
- `font-family="sans-serif"`

## Visual Result

- Label sits in row `+0` (the bar-number row), just above the note heads.
- Bar numbers (tiny, 0.6× font, same row) coexist without collision — they are visually distinct.
- The label is not repeated on subsequent system lines; it appears only at the measure where it is declared.

## Non-Goals

- No label wrapping or truncation — the label is rendered as-is and may overlap if very long.
- No MIDI effect — labels are purely visual.
- No label on the `1=X` (inline key change) token syntax; only the directive-line form is supported.
