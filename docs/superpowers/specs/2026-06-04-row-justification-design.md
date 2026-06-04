# Row Justification Design

**Date:** 2026-06-04

## Problem

Each row is currently centered individually based on its own `width_in_columns`. The first row is wider than subsequent rows because it carries directive prefix columns (BPM, time signature), so rows end up with different left margins and appear misaligned.

## Goal

All rows are fully justified to the page width (stretched to fill edge-to-edge within margins), so every row shares the same left and right edges. This matches conventional music engraving practice.

---

## Changes

### 1. Dependencies (`Cargo.toml`)

Add the `nonempty` crate. This enforces the non-emptiness of `RowGroup.elements` at the type level, making the division-by-zero concern structurally impossible.

### 2. Metadata fields

Remove `cell_size`. Replace with two new fields:

| Field | `.jianpu` key | Type | Default | Purpose |
|---|---|---|---|---|
| `row_height` | `row height` | `u32` (points) | `24` | Height of each row; drives font sizes, dot radii, arc heights, and all vertical spacing |
| `max_columns` | `max columns` | `u32` | `28` | Maximum logical columns per row before wrapping to the next row |

`cell_size` previously drove both horizontal and vertical sizing. Splitting them reflects the new reality: after justification, horizontal column width is derived per row, not set globally.

**Affected files:** `ast/parsed.rs`, `ast/grouped.rs`, `src/parser/metadata_parser.rs`, `src/grouper.rs`, `demo.jianpu`

### 3. Layout (`src/layout/mod.rs`)

- Replace all uses of `cell_size` / `cell` with `row_height` for vertical and proportional calculations (label column count, row group heights, rows-per-page).
- Replace `columns_per_page = (usable_width / cell) as u32` with `score.metadata.max_columns` for the line-wrap check.
- Change `RowGroup.elements` from `Vec<GridElement>` to `NonEmpty<GridElement>` (`nonempty::NonEmpty`). The two push sites (wrap flush and end-of-score flush) already guard with `if !current_elements.is_empty()`; these become the construction sites for `NonEmpty::from_vec`.

**Affected files:** `src/layout/types.rs`, `src/layout/mod.rs`

### 4. Renderer (`src/renderer.rs`)

Two widths are in play per row group:

- **`row_height`** — global, passed in from metadata. Used for: font sizes, dot radii, arc heights, all `y` position calculations.
- **`column_width`** — per row group, computed as:
  ```
  column_width = (page.page_width_pt - 2 × PAGE_MARGIN) / row_group.width_in_columns as f32
  ```
  Used exclusively for all `x` position calculations.

`margin_x` becomes the constant `PAGE_MARGIN` (no per-row centering math; content fills edge-to-edge within margins).

The renderer function signature changes from `render(pages, cell_size: u32)` to `render(pages, row_height: u32)`.

**Affected files:** `src/renderer.rs`, `src/main.rs`, `src/pdf.rs`

---

## Invariants

- `RowGroup.elements` is `NonEmpty<GridElement>` — a row group with zero elements cannot be constructed.
- `row_group.width_in_columns` is always ≥ 1 because a row group is only created after at least one measure is processed, which always advances the column counter by at least 1 (the bar line alone adds 1). Combined with the `NonEmpty` type, division by zero in `column_width` computation is impossible.

---

## Out of Scope

- `Notes.events` and `MultiPartMeasure.parts` are not migrated to `NonEmpty` in this change (future work).
- The last row is not treated differently — every row is justified, including the final partial row.
- No changes to the `.jianpu` score syntax, parser, or grouper beyond the metadata field rename.
