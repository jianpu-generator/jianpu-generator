# Design: Merge Ditto Part Labels onto Source Row

**Date:** 2026-06-11

## Problem

When a part is dittoed (copies a preceding part), the renderer omits its row entirely. The row label for the source part therefore only shows the source's own abbreviation (e.g. `S`), giving no indication that another part (e.g. `A`) shares the same notes in that measure.

## Goal

When a dittoed part is omitted from rendering, its label is appended to the label of the row it was copied from, separated by `, `. Example: Soprano (`S`) with Alto (`A`) dittoing it renders a single row labelled `S, A`.

## Change

**File:** `src/compiler/mod.rs`, function `compile_measure`.

Replace the `filter_map` iterator with a `for` loop:

1. `PartRow::Timed` → compile the slice, push a new `MeasureRow` with that part's label.
2. `PartRow::Ditto` → if `rows` is non-empty and the ditto label is non-empty, append `, <label>` to `rows.last_mut().label`.

## Edge Cases

- Empty ditto label (anonymous part) → skip appending, no change to source label.
- Ditto with no preceding timed row → skip (existing validation prevents this from occurring in practice).
- Multiple consecutive dittos on the same source → each appends in order, yielding e.g. `S, A, T`.

## Tests

Add a test in `src/tests/ditto.rs` asserting that when Alto dittos Soprano the compiled `MeasureRow` for Soprano has label `"S, A"` (or whatever the abbreviations are).
