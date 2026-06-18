# Bar Line Height Includes Lyric Rows

**Date:** 2026-06-13

## Problem

Bar lines in each system are rendered with a `height_pt` that is computed by
`compute_bar_height`, which sums sub-row heights for non-lyric rows only. When a
system contains lyric rows (either lyric-only parts, or note parts with attached
lyrics), those lyric rows are appended as `GridRow`s after the musical sub-rows in
`expand_system_to_rows`, but they are not accounted for in the bar line height.
The result is that bar lines stop short of the bottom of the system whenever lyrics
are present.

## Fix

Replace the body of `compute_bar_height` in `src/grid_layout/layout.rs` with:

```rust
fn compute_bar_height(first: &MeasureBlock, base: f32) -> f32 {
    system_musical_height_pt(first, base) + system_lyric_height_pt(first, base)
}
```

`system_lyric_height_pt` already counts every row for which `has_lyrics` is true
(both lyric-only rows and note rows with attached lyrics) and multiplies by
`lyric_row_height(base)`. This matches exactly what `expand_system_to_rows` emits,
so the bar line height will equal the full music + lyrics height of the system.

## Scope

- **Changed:** `compute_bar_height` in `src/grid_layout/layout.rs` (2 lines).
- **Unchanged:** The decoration row (BPM, time signature, bar number) sits above the
  system and is not part of the bar span — this stays correct because
  `system_lyric_height_pt` does not include the decoration row.
- **All bar lines affected:** both the left-edge bar (placed in `expand_note_part`
  when `part_idx == 0`) and every measure bar line (placed in
  `expand_measure_elements` when `part_idx == 0`) share the same `bar_height` value,
  so both are fixed by this change.

## Testing

Add a test asserting that when the last part of a system has lyrics, the
`BarLine { height_pt }` elements in the resulting `GridPage` rows equal
`system_musical_height_pt + system_lyric_height_pt`.
