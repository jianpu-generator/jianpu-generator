# Handoff: multi-verse lyrics in Unzipped view

Continuation of the plan in the original task (full plan text is in the
conversation that started this work — search chat history for "Support
multi-verse lyrics in the Unzipped view" if you need the original spec).
This file is the resume point after a context clear.

Branch: `feat/solo-edit-part-view`. Nothing has been committed yet — all
changes below are uncommitted working-tree edits.

## Done

1. **`src/unzipped_edit/mod.rs`** — fully rewritten for multi-verse support:
   - `extract_part_line` takes `(role, occurrence)` instead of just matching
     first slot for `target_index`.
   - `extract_unzipped_text` emits tagged `[Abbrev:lyrics:N]` blocks after
     each lyrics-bearing part's primary block; returns new
     `lyrics_verse_ranges: Vec<Vec<LyricsVerseRanges>>` field on
     `UnzippedExtractOutput` (new `LyricsVerseRanges { verse_number,
     measure_ranges }` struct).
   - `parse_unzipped_header` returns `ParsedUnzippedHeader { abbreviation,
     verse_number: Option<usize> }`, parsing both `[Abbrev]` and
     `[Abbrev:lyrics:N]`.
   - `split_unzipped_blocks` returns new `UnzippedBlocks { primary:
     HashMap<String,String>, lyrics_verses: HashMap<String,
     HashMap<usize,String>> }`.
   - `UnzippedEditError` gained `UnexpectedLyricsBlock` variant (tagged
     block names a part that isn't `NotesWithLyrics`/`Lyrics` kind).
   - `merge_unzipped_text` refactored into helpers to stay under the
     80-line clippy limit: `validate_unzipped_blocks`, `repack_all_parts`
     (+ `RepackedParts` type alias), `reconcile_bucket_lengths`,
     `build_raw_groups_for_desugar`, `render_score_lines`. New
     `VerseBuckets { verse_number, buckets }` struct, plus
     `part_is_blank_at_measure` / `lines_for_part_at_measure` helpers that
     implement the positional-backfill rule (a measure can't have verse 3
     without verses 1 and 2 — lower gaps get `desugar::implicit_fill`
     placeholders).
   - `scan_measure_token_counts` now takes extra `role: ScoreLineRole,
     occurrence: usize` params (was hardcoded to occurrence 0 of whatever
     role).
   - **Verified clean**: `cargo test --lib unzipped_edit` (20/20 pass) and
     `cargo clippy --workspace --all-targets -- -D warnings` — core crate
     (`jianpu-generator`) has **zero** clippy errors from this file as of
     last check. (Note: before this session started, `cargo clippy` on this
     file's *pre-existing* HEAD version — verified via `git stash` — already
     had ~15 clippy violations of its own; those are now fixed as a side
     effect of the refactor, not scope creep — don't revert them.)
   - `src/unzipped_edit/tests_capacity.rs` — updated the two existing calls
     to `scan_measure_token_counts` for the new `(role, occurrence)` params
     (pass `ScoreLineRole::Lyrics, 0`). No new tests added here yet (see
     Remaining #3).

2. **WASM boundary** — `crates/jianpu-wasm/src/types.rs` and
   `.../unzipped_edit.rs`:
   - New `LyricsVerseRangesOut { abbreviation, verse_number, ranges }`
     (`#[serde(rename_all = "camelCase")]` → TS field is `verseNumber`).
   - `UnzippedEditResponse::Ok` gained `lyrics_verse_ranges:
     Vec<LyricsVerseRangesOut>`.
   - `extract_unzipped_text_response`/`merge_unzipped_text_response` build
     it via new `lyrics_verse_ranges_out()` helper (flattens per-part
     `Vec<Vec<LyricsVerseRanges>>` into a flat list); both fold
     `UnexpectedLyricsBlock` into the generic `Err` response, same pattern as
     `MalformedHeader`.
   - `crates/jianpu-wasm/src/tests_unzipped_edit.rs` — updated existing
     tests for the new field, added
     `extract_unzipped_text_response_flattens_multiple_verses_into_tagged_blocks`
     and `merge_unzipped_text_response_returns_err_for_a_lyrics_tag_on_a_non_lyrics_part`.
   - **Verified**: `cargo test -p jianpu-wasm --features midi,wav,pdf` — 36
     passed (2 suites), 0 failures. WASM crate compiles clean against the
     new `mod.rs` shapes. (Clippy on the wasm crate specifically was not
     re-checked after this — the workspace-wide clippy run in Remaining #5
     will cover it.)

3. **Frontend, partial** — `web/src/hooks/workerHelpers.ts`:
   - Added `LyricsVerseRangesLike` interface and `UnzippedTextBlock` type.
   - Replaced `findPartIndexForOffset` with `collectUnzippedTextBlocks`
     (merges `partMeasureRanges` + `lyricsVerseRanges` into one
     offset-sorted block list, since a part's blocks are no longer
     necessarily contiguous — a `notes+lyrics` part now emits its primary
     block plus N verse blocks, interleaved in emission order with other
     parts' blocks) + `findBlockForOffset` (same "extends to next block's
     start" logic as before, just generalized from parts to blocks).
   - `measureRangeInUnzippedText` signature changed: now takes an optional
     third param `lyricsVerseRanges: LyricsVerseRangesLike[] = []`.
   - **This file's edit is done and self-consistent**, but nothing that
     calls `measureRangeInUnzippedText` has been updated to pass the new
     third argument yet (they still compile since it defaults to `[]`, but
     verse-block clicks won't resolve to the right measure until callers are
     threaded — see Remaining #1).

## Remaining (in original task-list order)

### 1. Finish frontend threading (was in progress when interrupted)

- **`web/src/hooks/useJianpuWorker.ts`**: mirrors `partMeasureRanges` state
  today (`useState<PartMeasureRangesOut[]>`, `partMeasureRangesRef`, set
  inside `useJianpuWorkerRenderRequests`, returned in `JianpuWorkerState`).
  Add a parallel `lyricsVerseRanges: LyricsVerseRangesOut[]` state +
  `lyricsVerseRangesRef`, following the exact same pattern at every line
  `partMeasureRanges`/`partMeasureRangesRef` currently appears (lines ~86-97,
  173, 315, 415 as of last read — re-check line numbers since the file may
  have shifted). Import `LyricsVerseRangesOut` from `jianpu-wasm` alongside
  `PartMeasureRangesOut`.
- **`web/src/hooks/useJianpuWorkerRenderRequests.ts`**: this hook currently
  calls `extract_unzipped_text(source)` once (~line 114) and does
  `setPartMeasureRanges(result.status === 'ok' ? result.part_measure_ranges
  : [])`. Add the parallel `setLyricsVerseRanges(result.status === 'ok' ?
  result.lyrics_verse_ranges : [])` right next to it, threading new
  `lyricsVerseRangesRef`/`setLyricsVerseRanges` params through the hook's
  params interface (mirror every `partMeasureRangesRef`/`setPartMeasureRanges`
  occurrence — there are 3 call sites of `measureRangeInUnzippedText` in
  this file, all currently passing only `partMeasureRangesRef.current`; each
  needs `, lyricsVerseRangesRef.current` appended as the third arg).
- **`web/src/hooks/useJianpuWorkerTypes.ts`**: add `lyricsVerseRanges:
  LyricsVerseRangesOut[]` to the `JianpuWorkerState` interface, import type
  from `jianpu-wasm`, doc comment mirroring the existing
  `partMeasureRanges` one.
- Double check nothing downstream (e.g. a Unzipped-view React component)
  needs the new field directly — as of last check, `partMeasureRanges` in
  `JianpuWorkerState` appeared to only be consumed inside this hook chain
  itself (not re-grepped exhaustively — verify with `grep -rn
  "\.partMeasureRanges\b" web/src` before assuming `lyricsVerseRanges` needs
  no further plumbing beyond the hook return value).
- **`web/src/monacoJianpuLanguage.ts`**: **no change needed** — already
  verified. Its Monarch tag pattern is `[/\[[^\]\n]*\]/, 'tag']`, which
  matches any bracket content except `]`/newline, so `[Abbrev:lyrics:2]`
  already highlights like `[Abbrev]` today. Don't "fix" this.

### 2. ARCHITECTURE.md (required by CLAUDE.md — key type/signature changes)

Two spots to update in `/Users/wongjiahau/personal-repos/jianpu-generator/ARCHITECTURE.md`:
- Glossary entry **"Unzipped View"** (~line 131): mention multi-verse
  `[Abbrev:lyrics:N]` blocks.
- Section **"Unzipped Edit (source-level)"** (~lines 177-186): rewrite to
  describe:
  - `[Abbrev]` = slot occurrence 0 of the part's first static role;
    `[Abbrev:lyrics:N]` = additional Lyrics-role occurrences.
  - New `LyricsVerseRanges`/`lyrics_verse_ranges` field on
    `UnzippedExtractOutput`.
  - New `UnzippedEditError::UnexpectedLyricsBlock` variant.
  - The positional-backfill merge-back rule (can't have verse 3 without
    verses 1-2; force-filled with `implicit_fill`).
  - Update the WASM export table rows (~lines 197-198) for
    `extract_unzipped_text`/`merge_unzipped_text` to mention
    `lyrics_verse_ranges: LyricsVerseRangesOut[]`.
- `syntax.md`: **do not touch** — unchanged by design (Unzipped text is a
  derived view, never the on-disk grammar).

### 3. Rust tests (per original plan's "Tests" section)

- **`src/unzipped_edit/tests_extract.rs`**: add cases for `NotesWithLyrics`/
  `Lyrics` parts with 1, 2, and measure-varying verse counts — assert
  correct block headers (`[Abbrev:lyrics:N]`), correct byte ranges into
  `lyrics_verse_ranges`, correct `_`-filled content when a verse is
  implicit for a given measure.
- **`src/unzipped_edit/tests_merge.rs`**: editing only a verse block
  round-trips independently of notes/other verses; deleting a verse block
  (omitting it from input) removes it, with correct positional backfill of
  lower verses when a higher one still has content; `UnexpectedLyricsBlock`
  when a `[Abbrev:lyrics:N]` header names a non-lyrics-bearing part;
  malformed verse tag (`N=0`, non-numeric, e.g. `[Abbrev:lyrics:abc]`) →
  `MalformedHeader`; **dedicated regression test**: verse 3 nonempty, verse
  2 empty, notes empty at the same measure → forces notes line filled with
  `implicit_fill(Notes, ..)` ("0 0 0 0" for 4/4) and verse 2 filled with
  `implicit_fill(Lyrics, ..)` ("_").
- **`src/unzipped_edit/tests_capacity.rs`**: optionally add a case
  exercising `scan_measure_token_counts` with `occurrence > 0` (verse 2+)
  and a measure where that occurrence doesn't exist (expect `0`, not `1`
  like the existing "missing part entirely" test — these are different
  cases, don't conflate them).
- **`src/unzipped_edit/tests_quickcheck.rs`**: extend
  `RandomJianpuDocument`'s generator so `NotesWithLyrics` parts are
  representable (currently `GeneratedPartKind` deliberately excludes
  `NotesWithLyrics` — see the comment above `enum GeneratedPartKind` in that
  file, ~line 58-62, explaining why) and so both `NotesWithLyrics` and
  `Lyrics` parts can randomly emit 1-3 verse lines per measure,
  independently per measure, to fuzz the positional-backfill logic under
  the existing extract→merge→extract→merge fixed-point property
  (`prop_extract_merge_round_trip_is_idempotent`). This is the highest-value
  test to get right since it's the one that would have caught the original
  bug — but also the fiddliest, since the generator currently only produces
  one score line per part per measure and needs to grow multi-line
  generation for lyrics-bearing kinds specifically.

### 4. e2e Playwright spec

Add to (or add a sibling of)
`web/e2e/unzipped-view-selection-highlight.spec.ts`: a `notes+lyrics` part
with 2+ verses in different measures (e.g. verse 1+2 in measure 0, verse 1
only in measure 1) — verify both verse blocks render as separate
`[Abbrev:lyrics:N]` sections in the Unzipped editor, and that clicking a
token in each verse block highlights the correct measure (reuse the
`clickAtPosition`/`waitForUnzippedText`/`toggleUnzippedView` helpers
already in that file).

### 5. Final verification + commit

- `cargo test --lib unzipped_edit` and `cargo test -p jianpu-wasm
  --features midi,wav,pdf` (or whatever feature set matches
  `web/package.json`'s `build:wasm` script — check it, don't assume).
- `cargo clippy --workspace --all-targets -- -D warnings` (this is the
  **actual** pre-commit invocation, from `lefthook.yml`'s `cargo-checks`
  job — use this exact command, not `--lib --tests -p <crate>`, since a few
  `-D`-vs-`forbid` lints behave slightly differently and the workspace vs.
  single-crate scope matters).
- `cd web && pnpm exec tsc -b` to typecheck the frontend changes (the
  `LyricsVerseRangesOut` type only exists after `pnpm run build:wasm`
  regenerates `jianpu-wasm`'s TS bindings from the Rust `Tsify` derives —
  run `pnpm run build:wasm` first if `tsc` complains the export doesn't
  exist).
- Manually exercise in browser per original plan's Verification section
  (create a `notes+lyrics` part with 2 verses in one measure, 1 in the
  next; confirm both verse blocks render/edit independently in Unzipped
  view and Zipped view reflects it correctly).
- Commit with `git commit` and an explicit `timeout: 480000` (8 min) on the
  Bash call — the pre-commit hook runs the full e2e suite + cargo checks and
  will exceed the default 2-minute tool timeout. Do not run
  `cargo test`/`cargo clippy`/e2e manually as a pre-commit double-check
  beyond what's listed above (CLAUDE.md: the hook already gates on commit).
- Commit message: scope should be `unzipped-edit` (or similar — the module
  doing the most work), per this repo's `<type>(<scope>): <description>`
  convention documented in CLAUDE.md. Something like `feat(unzipped-edit):
  support multi-verse lyrics in Unzipped view`.
