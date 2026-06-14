---
name: slur-span-design
description: Design for replacing per-measure TieOrSlur/TieOrSlurClose column elements with a SlurSpan data model that correctly renders same-system and cross-system slur/tie arcs
metadata:
  type: project
---

# Slur/Tie Rendering — SlurSpan Design

## Problem

Slur and tie arcs are currently rendered with two bugs:

1. **Position bug** (`coordinate_resolver/resolve.rs`): `TieOrSlur` grid elements use `HAlign::Center`, placing `elem.x` at the midpoint of the full column span. The renderer then draws from `elem.x` to `elem.x + width`, shifting both arc endpoints right by half a column width.

2. **Span bug** (`compiler/mod.rs`): cross-measure slur chains emit `TieOrSlur { to_column: barline_column }` — the arc ends at the barline rather than the closing note. Slur groups (as opposed to ties) never emit a `TieOrSlurClose` for the closing note, so the closing endpoint is entirely missing.

**Correct behavior:**
- Same-system slur/tie: single continuous arc from the center of the opening note to the center of the closing note.
- Cross-system slur/tie: one arc per system crossed — in the first system from the opening note center to the right edge, in the last system from the left edge to the closing note center.

## Approach

Replace the per-measure `ElementContent::TieOrSlur` / `TieOrSlurClose` column elements with a `SlurSpan` value that captures the full logical extent of each slur or tie. The layout stage, which already knows system boundaries, resolves each span into one or two grid arcs at the correct absolute columns.

## Data Model

### `SlurSpan` and `CompileResult` — `compiler/types.rs`

```rust
pub struct SlurSpan {
    pub part_index: usize,
    pub from_measure: usize,  // global measure index
    pub from_column: u32,     // measure-relative column of the opening note
    pub to_measure: usize,
    pub to_column: u32,       // measure-relative column of the closing note
}

pub struct CompileResult {
    pub blocks: Vec<MeasureBlock>,
    pub slur_spans: Vec<SlurSpan>,
}
```

`ElementContent::TieOrSlur` and `ElementContent::TieOrSlurClose` are removed.

### Grid arc variants — `grid_layout/types.rs`

`GridContent::TieOrSlur` and `GridContent::TieOrSlurClose` are replaced with:

| Variant | Meaning |
|---|---|
| `TieOrSlur` | Same-system arc: center of from-column to center of to-column |
| `TieOrSlurTail` | Cross-system, first system: center of from-column to right edge of system |
| `TieOrSlurHead` | Cross-system, last system: left edge of system to center of to-column |

## Compiler

### `PartCrossState`

```rust
struct PendingSlurOpen {
    measure_index: usize,
    from_column: u32,
}

struct PartCrossState {
    prev_tie: bool,
    prev_slur_key: Option<SlurKey>,
    pending_slur_opens: Vec<Option<PendingSlurOpen>>,  // indexed by chain depth
}
```

### `compiler::compile` return type

`compile()` returns `CompileResult` instead of `Vec<MeasureBlock>`.

### Slur emission rules

**Same-measure chain close** (`flush_chain`): emits a `SlurSpan` directly instead of a `ColumnElement::TieOrSlur`.

**Cross-measure chain open** (end of measure, chain length == 1): saves a `PendingSlurOpen { measure_index, from_column }` into `pending_slur_opens[depth]`. No column element is emitted.

**Cross-measure chain close** (next measure, note with `group_continuation < group_membership`): for each closing depth, if `pending_slur_opens[depth]` is set, emits a `SlurSpan` using that open's `measure_index` and `from_column` as the origin, then clears the slot.

**Cross-measure tie close** (the existing `TieOrSlurClose` path): emits a `SlurSpan` instead of a `ColumnElement::TieOrSlurClose`.

## Layout

### `MeasurePlacement`

After `pack_into_systems`, the layout builds:

```rust
struct MeasurePlacement {
    system_index: usize,
    column_offset: u32,  // measure's starting column within its system, after LABEL_COLS
}
```

A `Vec<MeasurePlacement>` indexed by global measure index is built by walking the packed systems in order and accumulating column widths.

### Span resolution

`grid_layout::layout()` accepts `&CompileResult` (for both `blocks` and `slur_spans`).

For each `SlurSpan`:

- Look up `from_placement` and `to_placement` by measure index.
- **Same system** (`from_placement.system_index == to_placement.system_index`): inject one `GridContent::TieOrSlur` element into sub-row 0 of the correct part in that system, spanning from `LABEL_COLS + from_placement.column_offset + from_column` to `LABEL_COLS + to_placement.column_offset + to_column`.
- **Different systems**: inject `GridContent::TieOrSlurTail` in the from-system, spanning from `LABEL_COLS + from_placement.column_offset + from_column` to the last column in that system (`LABEL_COLS + system_column_count - 1`); and inject `GridContent::TieOrSlurHead` in the to-system, spanning from `LABEL_COLS` to `LABEL_COLS + to_placement.column_offset + to_column`.

Arcs are injected inside `expand_note_part`, which already has direct access to sub-row 0.

## Coordinate Resolution

All three arc variants bypass the `halign`-based `elem.x` calculation (matching the existing `Underline` special case):

| Variant | `elem.x` | `width` |
|---|---|---|
| `TieOrSlur` | `x_start + col_width / 2` | `(column_span − 1) × col_width` |
| `TieOrSlurTail` | `x_start + col_width / 2` | `span_width − col_width / 2` |
| `TieOrSlurHead` | `x_start` | `(column_span − 1) × col_width + col_width / 2` |

## Renderer

`render_tie_or_slur` is unchanged structurally: it draws a quadratic bezier from `(elem.x, elem.y)` to `(elem.x + width, elem.y)` with a control point above the midpoint. All three arc variants share this renderer — their differences are fully captured by `elem.x` and `width` from the resolve step.

## Files Changed

| File | Change |
|---|---|
| `compiler/types.rs` | Add `SlurSpan`, `CompileResult`; remove `TieOrSlur`, `TieOrSlurClose` from `ElementContent` |
| `compiler/mod.rs` | Extend `PartCrossState` with `pending_slur_opens`; change `compile()` return type; update `flush_chain`, end-of-measure flush, and tie-close logic to emit `SlurSpan` |
| `grid_layout/types.rs` | Replace `TieOrSlur`, `TieOrSlurClose` with `TieOrSlur`, `TieOrSlurTail`, `TieOrSlurHead` in `GridContent` |
| `grid_layout/expand.rs` | Remove `TieOrSlur` / `TieOrSlurClose` handling from `expand_measure_elements` |
| `grid_layout/layout.rs` | Accept `&CompileResult`; add `MeasurePlacement` computation; resolve spans into grid arcs inside `expand_note_part` |
| `coordinate_resolver/resolve.rs` | Add special-case handling for all three arc variants (like existing `Underline` treatment) |
| `renderer/new_renderer.rs` | Add match arms for `TieOrSlurTail` and `TieOrSlurHead` pointing to the same renderer |
| `lib.rs` | Thread `CompileResult` through `render_svgs` |
