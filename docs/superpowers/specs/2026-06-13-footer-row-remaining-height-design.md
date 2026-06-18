# Design: Footer Row Takes Remaining Page Height

**Date:** 2026-06-13

## Problem

The footer row currently has a fixed height (`base * 0.40`) and `valign: Center`. This means the page number text floats near the last system rather than anchoring to the bottom of the page.

## Goal

The footer row should expand to fill whatever vertical space remains on the page after all body rows are placed, and its content should be vertically aligned to the bottom edge of that space — so the page number appears at the very bottom of each page.

## Changes

### 1. `src/grid_layout/layout.rs` — `layout()` function

After `build_page_rows()` returns the body rows, compute the remaining height:

```
remaining_height = page_height_pt - 2 * PAGE_MARGIN - sum(body_rows heights)
```

Pass `remaining_height` into `make_footer_row` instead of deriving the height from `base` inside that function.

### 2. `src/grid_layout/layout.rs` — `make_footer_row()`

- Accept `height_pt: f32` as a parameter instead of computing it internally.
- Change the footer element's `valign` from `VAlign::Center` to `VAlign::Bottom`.

### 3. No changes needed elsewhere

- `coordinate_resolver/resolve.rs` already handles `VAlign::Bottom` as `row_y + row.height_pt` (line 34). The footer text will land at the bottom of the expanded row, which coincides with `page_height_pt - PAGE_MARGIN` — exactly the bottom margin line.
- `footer_row_height()` helper should be deleted; after this change it will have no callers.

## Invariants

- The remaining height will always be positive because `layout()` already subtracts `footer_h` from `usable_h` when packing systems, guaranteeing that body content never overflows into footer space.
- No user-facing `.jianpu` syntax is affected.
- No test fixtures need updating unless snapshot tests capture absolute y-coordinates of the footer element.
