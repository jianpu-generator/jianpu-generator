# Lyrics Toggle Should Reclaim Vertical Space — Design

**Date:** 2026-06-10
**Status:** Approved behavior, recommended approach selected

## Problem

In the web editor, deactivating a part's lyrics (the per-part "lyrics" checkbox)
hides the syllables but the rendered score still reserves the vertical space
where the lyric row used to be. The result is a blank gap under the notes of
that part.

## Root Cause

The lyrics toggle flows through `apply_lyrics_filter` (`src/lib.rs`), which
clears the syllables by setting `part_slice.lyrics = None` — but leaves the
part's `kind` as `PartKind::NotesWithLyrics`.

The layout engine sizes part rows purely from `kind` via `part_row_height`
(`src/layout/mod.rs`):

| Kind | Rows |
|------|------|
| `Chord` | 2 |
| `Notes` | 3 |
| `NotesWithLyrics` | 4 |

`row_group_height` (`src/layout/layout_engine.rs`) sums these per part from the
first measure, so the stale `NotesWithLyrics` kind keeps the 4th row reserved
even though no lyric elements are emitted into it.

## Desired Behavior

When lyrics are disabled for a part, the part occupies the same height as a
notes-only part (3 rows instead of 4). Lines move closer together and the score
may reflow onto fewer pages. This applies to every consumer of
`apply_lyrics_filter`: the web preview (SVG) and PDF export via
`crates/jianpu-wasm`.

## Approach

**Chosen: downgrade `kind` in the filter.**

In `apply_lyrics_filter`, alongside `part_slice.lyrics = None`, also set
`part_slice.kind = PartKind::Notes`. After the filter runs, the score is
indistinguishable from one authored without lyrics for that part, so the layout
engine needs no changes and all render paths (SVG, PDF) are fixed at once.

**Rejected alternative: make the layout lyrics-aware.** Teaching
`part_row_height` to return 3 when `lyrics` is `None` would make row height
vary per measure — parts can legitimately have lyric-less measures mid-score —
misaligning row cursors unless a score-level pass is added. More code, more
risk, no benefit over the data-level fix.

## Scope of Change

- `src/lib.rs` — `apply_lyrics_filter`: one added line setting
  `part_slice.kind = PartKind::Notes` (plus the `PartKind` import if not
  already in scope).
- No changes to the layout engine, renderer, web frontend, or `.jianpu` syntax
  (so `syntax.md` is unaffected).

## Error Handling

No new failure modes: the filter remains a pure in-memory transformation. Parts
without lyrics or not named in `disabled_lyrics` are untouched; `None` /
`Some([])` remain no-ops.

## Testing

- Rust unit test: build a score with a `notes lyrics` part, run
  `apply_lyrics_filter` for that part, run layout, and assert the row group
  height matches a notes-only part (3 rows, not 4). The existing test
  `render_with_disabled_lyrics_hides_lyrics_for_part`
  (`crates/jianpu-wasm/src/lib.rs`) continues to cover syllable removal.
- Verify untouched parts keep their lyric row (filter only affects listed
  abbreviations).
- Manual check in the web editor: toggling lyrics off visibly shrinks the gap;
  toggling back on restores it (the filter re-runs from the freshly parsed
  score each render, so no state is lost).

## Process Notes

Per project rules, run `gitnexus_impact` on `apply_lyrics_filter` before
editing and `gitnexus_detect_changes()` before committing.
