# Replace Solo Edit with Zipped ⇄ Unzipped whole-document view

## Current implementation state (as of last session pause)

**Git working tree** (uncommitted, not yet staged beyond the renames below):
- `git mv src/solo_edit src/unzipped_edit` done.
- `git mv src/unzipped_edit/tests.rs src/unzipped_edit/tests_merge_legacy.rs` done — this file still contains the **old** Solo Edit tests (`extract_solo_text`/`merge_solo_text` API) and has **not** been rewritten yet. It must be replaced by the Phase 5 test files (`tests_capacity.rs`, `tests_extract.rs`, `tests_merge.rs`) and then deleted, or its content fully rewritten against the new API — do not leave it referencing the old functions once `mod.rs` no longer defines them.
- `git mv crates/jianpu-wasm/src/solo_edit.rs crates/jianpu-wasm/src/unzipped_edit.rs` done — contents not yet rewritten (still calls old `jianpu_generator::solo_edit::{extract_solo_text, merge_solo_text}`).
- `src/unzipped_edit/mod.rs` itself: **not yet edited** — still contains the old `SoloExtractOutput`/`SoloEditError`/`extract_solo_text`/`merge_solo_text`/`resolve_context` (single-abbreviation) implementation verbatim (just moved). This is the main file to rewrite for Phases 0/2/3.
- No Rust code for the new API has been written yet. No frontend files touched yet. `ARCHITECTURE.md`/`crates/jianpu-wasm/src/{lib.rs,types.rs}` not yet touched.
- A `TaskCreate` list (task ids #1–#9) mirrors the phase checklist below in this same session — a fresh session should recreate an equivalent todo list since task state doesn't carry over.

**Key facts discovered while reading the existing code** (needed to resume without re-deriving them):

- `crate::desugar::SourceLine` (note: `desugar` not `solo_edit`) is `{ content: String, offset: usize, group: Option<String> }`, `pub(crate)`. `crate::desugar::desugar_groups(groups: Vec<Vec<(String,usize)>>, declarations: &[PartDecl], resolved_groups: &[ResolvedGroup], base_offset: usize) -> Result<(Vec<Vec<SourceLine>>, Vec<Vec<ScoreLineSlot>>, Vec<Option<RecoverableError>>, Vec<AbbreviationReference>), IrrecoverableError>` is `pub(crate)`, already does the "carry current time_num forward + fill implicit rests" desugaring the whole plan builds on. `crate::desugar::parse_key_prefix(line) -> Option<(&str, &str)>` also `pub(crate)`.
- `parser::score::measure_group::collect_groups(content: &str) -> Vec<Vec<(String, usize)>>` (plain tuples, `pub`) splits on blank lines — this is exactly the blank-line/paragraph splitter needed for Phase 3a's unzipped-text block splitting too (reuse directly on `unzipped_text`, don't hand-roll another splitter). `is_directive_line`/`directive_line_count` also live here, `pub`.
- `parser::score::interleaved_directives.rs` has `pub(super) fn split_directive(lines: &[SourceLine], base_offset: usize) -> (Vec<Spanned<ScoreEvent>>, &[SourceLine], Vec<RecoverableError>)` — `super` here is `interleaved_parser` (declared via `#[path=...] mod directives;` inside `interleaved_parser.rs`, which is `pub mod` in `parser/score/mod.rs`). Plan: add a `pub(crate) fn` in `interleaved_parser.rs` itself (sibling to the private `mod directives`) that calls `directives::split_directive(...)` and returns just the events — e.g. `pub(crate) fn directive_events_for_group(group: &[SourceLine], base_offset: usize) -> Vec<Spanned<ScoreEvent>>`. This is how `interleaved_parser::parse` itself currently tracks `time_num`/`time_den` across groups (see `process_bar_group`, ~line 236-245) — mirror that loop exactly in the new `scan_measure_capacities`.
- `interleaved_beat_padding::beats_per_measure(num: u8, den: u8) -> u32` is `pub(super)` (`(num as u32) * (16 / den as u32)`) — not reusable directly; just reimplement the one-line formula locally in the new capacity module (trivial, not worth a visibility change).
- `token_parser::parse_notes_line/parse_chord_line/parse_percussion_line(line: &str, base_offset: usize, stack: &mut GroupStack) -> Result<TimedLineParse, IrrecoverableError>` are `pub`. `TimedLineParse { events: Vec<Spanned<ScoreEvent>>, chord_errors: Vec<Diagnostic>, lex_errors: Vec<RecoverableError> }`, `pub` fields, `pub use`d from `token_parser`.
- `ScoreEvent` variants relevant to repacking: `Note(ParsedNote)`, `Chord(ParsedChordNote)`, `PercussionHit(ParsedPercussionHit)`, `Rest(ParsedRest)` all have a `.duration: u32` field (quarter-beats); `Extension { dotted: bool }` (4 or 6 beats, folds into previous cluster, does not start a new one); `TieMarker` (0 beats, also folds into previous, only sets `slur`). Directive variants (`BpmChange`, `KeyChange`, `TimeSignatureChange`, `LabelChange`, `MergeDuplicateMeasuresAcrossPartsChange`, `HideRestingPartsChange`) never appear in a per-part flat-text repack (directives are stripped out / stay pinned per the design) so the repack fold only needs to handle the six timed variants above.
- `PartGrouper::handle_extension` (`src/grouper/part_grouper.rs:180`) is the reference implementation for "extension adds beats to previous event" — confirmed it just adds `4`/`6` (dotted) beats to whichever `NoteEvent` variant is last, no multiplier logic needed at this layer (multiplier/tuplet rescaling is explicitly out of scope per the plan). Write a small local fold in `unzipped_edit` rather than depending on the grouper's internal `NoteEvent` type (which lives in a different, later pipeline stage).
- `ast::parsed::PartDecl { abbreviation, abbreviation_span, display_name, kind: PartKind, follow_target, soundfont, volume, octave_offset }`; `PartKind { Chords, Notes, NotesWithLyrics, Percussion, Lyrics }`; `ScoreLineRole { Chord, Notes, Lyrics }`; `ScoreLineSlot { track_index, role }`; `PartDecl::score_line_roles()` maps kind → static role slice.
- `crates/jianpu-wasm/src/types.rs` already has `SpanOut { start: usize, end: usize }` (`Tsify`+`Serialize`+`PartialEq`+`Eq`, `#[tsify(into_wasm_abi)]`) — **reuse this instead of adding a new `RangeOut`** (identical shape, avoids a redundant duplicate type per CLAUDE.md's no-premature-abstraction rule). Only need to add `PartMeasureRangesOut { abbreviation: String, ranges: Vec<SpanOut> }` and the `UnzippedEditResponse` enum (replacing `SoloEditResponse`).
- No `regex` crate dependency exists in `Cargo.toml` (only `itertools = "0.13"`). Header-line matching (`^\[(\w+)\]\s*$`) for Phase 3a must be done with manual `str` parsing (e.g. `strip_prefix('[')`/`strip_suffix(']')`/trim, matching `desugar::parse_key_prefix`'s style), not a regex dependency.
- Still undecided/needs a call when resuming: exact `UnzippedEditError` variant for a **malformed** header line (doesn't even look like `[...]`) vs. `UnknownPart` for a well-formed-but-undeclared abbreviation — plan text only explicitly calls out `UnknownPart` for the latter; lean towards a shared `UnzippedEditError::ParseFailed` (or a new `MalformedHeader` variant if wasm response shape needs to distinguish) for the former, and write the "malformed header" test in Phase 5 against whichever is chosen.
- `scan_measure_token_counts`'s signature in the plan text omits a `resolved_groups: &[ResolvedGroup]` parameter, but `desugar::desugar_groups` requires one — add it when implementing (the plan's signature was abbreviated, not literal).

## Progress checklist

- [ ] Phase 0: Rust capacity scanning utility (`scan_measure_capacities`, `scan_measure_token_counts`)
- [ ] Phase 1: rename `src/solo_edit/` → `src/unzipped_edit/` (lib.rs, ARCHITECTURE.md, wasm imports)
- [ ] Phase 2: Rust extraction (`extract_unzipped_text`, `UnzippedExtractOutput` w/ `part_measure_ranges`)
- [ ] Phase 3: Rust merge-back (`merge_unzipped_text`, repack-by-capacity algorithm)
- [ ] Phase 4: WASM boundary (`crates/jianpu-wasm/src/unzipped_edit.rs`, `types.rs`, `lib.rs` exports)
- [ ] Phase 5: Rust tests (tests_capacity, tests_extract, tests_merge, wasm tests_unzipped_edit)
- [ ] Phase 6: Frontend
  - [ ] `usePartToggles.ts` — remove `soloEditPart`/`toggleSoloEdit`
  - [ ] `App.tsx` — add `unzippedView` state (reset on file switch, not persisted)
  - [ ] `PartToggles.tsx` — delete pencil "Solo Edit" segment
  - [ ] `AppWorkspace.tsx` — swap imports, props, write-back branch, Editor path, toggle button
  - [ ] `useJianpuWorkerRenderRequests.ts` — move `extract_unzipped_text` call here, `notifyUnzippedSelection`
  - [ ] `workerHelpers.ts` — `measureRangeInUnzippedText` replacing `measureRangeInSoloText`
  - [ ] `Editor.tsx` — add `onSelectionOffsetChange` optional callback via `model.getOffsetAt`
  - [ ] `e2e/unzipped-view-selection-highlight.spec.ts` replacing solo-edit spec
- [ ] Docs: `ARCHITECTURE.md` Unzipped View section (state `syntax.md` unchanged deliberately)

## Context

Solo Edit (branch `feat/solo-edit-part-view`, commits `1714290`, `12f63fe`) currently isolates one part at a time, with measures still delimited one-line-per-measure. The user wants a bigger pivot: a whole-document **Unzipped** view showing *all* parts at once, each under a `[Abbrev]` header, with the part's notes flattened into one continuous token stream — no per-measure line breaks. Newlines inside a part's block become purely cosmetic wrapping. The point is to let the user freely shift/insert/delete notes across what used to be measure boundaries without manually managing bar lines; on write-back, the flat stream is automatically re-barred into measures by beat capacity.

**Zipped** (today's canonical, blank-line-delimited, `[Abbrev]`-per-line format) remains the only format ever stored on disk. Unzipped is a derived, editable projection — extract on view-enter, merge back into Zipped on every edit (live write-back), exactly like Solo Edit does today but for the whole score instead of one part.

This fully replaces Solo Edit: the per-part pencil toggle goes away, replaced by one Zipped/Unzipped toggle for the whole editor.

## Design confirmed with user

- Measure boundaries in Unzipped view are *not* explicit — they're recomputed automatically from beat capacity (time signature), so the user can shift content freely.
- Directive lines (`bpm=`, `time=`, `key=`, `label=`) stay pinned to their original measure index; they are never re-derived from Unzipped text and never move.
- Total measure count after merge = max across parts of their repacked measure count (never fewer than the original document's count, so no directive is orphaned). Parts that repack shorter are padded with rest measures at the tail.
- Tuplet `resolution_multiplier` capacity scaling is explicitly deferred (known v1 limitation, not to be fixed now).

## Phase 0 — Rust: capacity scanning utility

New file, part of the renamed module (see Phase 1): a function that scans the **original** `# score` content once and returns one beat-capacity per original measure-group index (quarter-beat units, `numerator*16/denominator`), carrying the active `time=` directive forward across groups (same "current time signature" logic `PartGrouper`/`DirectiveGrouper` already use elsewhere — do not reimplement `time=` parsing from scratch).

```rust
fn scan_measure_capacities(score_content: &str) -> Vec<u32>
```

Built on `parser::score::measure_group::collect_groups`. For directive parsing, reuse the existing directive-line parsing in `src/parser/score/interleaved_directives.rs` (`split_directive` et al.) — if it's not `pub(crate)`, add a small `pub(crate)` wrapper in `interleaved_parser.rs` (which is already `pub mod`) rather than broadening the whole module's visibility.

Also needed: a **Lyrics-kind variant**, since lyric parts have no beat/duration grammar — their "capacity" is a token (syllable) count per original measure, not quarter-beats:

```rust
fn scan_measure_token_counts(score_content: &str, declarations: &[PartDecl], target_index: usize) -> Vec<u32>
```
(reuses `extract_part_line` + `.split_whitespace().count()` per measure group, same desugar pass already used elsewhere in this module).

## Phase 1 — Rust: rename `src/solo_edit/` → `src/unzipped_edit/`

- Update `src/lib.rs` re-export, `ARCHITECTURE.md` glossary/section, `crates/jianpu-wasm` imports.
- Keep `extract_part_line` and the `resolve_context`-style parsing helpers (parts/groups resolution) — still needed internally, just no longer the public entry points.
- Delete the old public `extract_solo_text`/`merge_solo_text` (nothing else will call them after Phase 4).

## Phase 2 — Rust: extraction

```rust
pub struct UnzippedExtractOutput {
    pub text: String,
    /// Per declared part (declaration order), per measure index: byte range
    /// [start, end) within `text` covering that measure's tokens.
    pub part_measure_ranges: Vec<Vec<(usize, usize)>>,
}
pub fn extract_unzipped_text(source: &str) -> Result<UnzippedExtractOutput, UnzippedEditError>
```

Parse `# parts` once (all declarations, in order — no single-abbreviation param anymore), run `collect_groups` + `desugar_groups` once, shared across all parts (not re-run per part like today's per-abbreviation `resolve_context`). For each declaration index `i`:
- For each measure group, `extract_part_line(group, slots, i)` (verbatim reuse).
- Join per-measure lines with `" "` (not `\n`), recording each measure's `[start, end)` byte offset in the growing `text` as it's built (no separate re-scan pass).
- Emit `[{abbrev}]\n` header, then the flattened line, then `"\n\n"` before the next part (no trailing separator after the last part).

## Phase 3 — Rust: merge-back (the new algorithm)

```rust
pub fn merge_unzipped_text(source: &str, unzipped_text: &str) -> Result<String, UnzippedEditError>
```

**3a. Split `unzipped_text` into per-part blocks.** Split on blank-line-delimited paragraphs; first line of each block matching `^\[(\w+)\]\s*$` is the header, the rest — with internal newlines collapsed to a single space (newline is insignificant) — is that part's flat token text. Build `HashMap<abbrev, String>`.
- A header that doesn't match any declared part → `UnzippedEditError::UnknownPart`.
- A declared part with **no** block present → treated as empty text (valid way to blank out a part), not an error.

**3b. Repack each part's flat text into measures**, dispatching on `PartKind`:
- `Notes | NotesWithLyrics` → `token_parser::parse_notes_line(flat_text, 0, &mut GroupStack::default())`
- `Chords` → `token_parser::parse_chord_line(...)`
- `Percussion` → `token_parser::parse_percussion_line(...)`
- `Lyrics` → tokenize on whitespace directly (no duration grammar); capacity is a token count (Phase 0's `scan_measure_token_counts`), not quarter-beats.

For Notes/Chords/Percussion, each parsed `Spanned<ScoreEvent>` has a `.span` (byte range into `flat_text`, since `base_offset=0`) and a duration in quarter-beats (already computed internally via `parse_duration_suffixes`). `ScoreEvent::Extension`/tie events add their beats to the *previous* event rather than starting a new one — fold these the same way `PartGrouper::handle_extension` does (small local helper, reuse the logic, don't hand-roll a divergent copy) to produce a flat `Vec<(Span, u32 /*duration*/)>` per non-extension token.

Greedy-fill against the capacity list from Phase 0 (`capacities[m]`, extending with `capacities.last()` once past the original measure count — this is the "shift content, auto re-bar" behavior):

```
current_beat = 0; m = 0; buckets = [[]]
for (span, dur) in events:
    if current_beat >= capacity(m): m += 1; current_beat = 0; buckets.push([])
    buckets[m].push(span); current_beat += dur
```
Overflow within a bucket (a single token pushes `current_beat` past capacity) is **not** split or specially handled — it spills into that measure's text and surfaces as the existing beat-overflow diagnostic on next parse, matching current UX for over-full measures elsewhere in the app.

Each part's output: `Vec<String>`, one joined (`" "`-joined raw substrings sliced from `flat_text` by span) entry per new measure index.

**3c. Reconcile across parts:**
```
new_total = max(original_measure_count, max over parts of buckets.len())
pad every part's bucket list to new_total with "" (empty)
```

**3d. Reassemble.** Directive lines are never regenerated — reuse `collect_groups(original_score_content)` to fetch each original group's directive line text for measure index `m < original_measure_count`; measures `m >= original_measure_count` get no directive line. For each `m in 0..new_total`, emit the directive line (if any) followed by `[Abbrev] <tokens>` per declared part in order. For parts with `""` at index `m` (either padded-tail or genuinely empty edited block), don't try to hand-synthesize the correct number of rest tokens — leave the line blank/omit it and run one final `desugar_groups` pass over the assembled groups at the end, exactly as `merge_solo_text` does today via its `padded_complemented` → `desugared` re-run, letting the existing implicit-rest-fill machinery produce the right rest tokens. Splice the reassembled `# score` content into `source` using the same byte-range replace `merge_solo_text` already does (verbatim reuse of that tail).

## Phase 4 — WASM boundary

`crates/jianpu-wasm/src/unzipped_edit.rs` (renamed from `solo_edit.rs`):
```rust
pub(crate) fn extract_unzipped_text_response(source: &str) -> UnzippedEditResponse
pub(crate) fn merge_unzipped_text_response(source: &str, unzipped_text: &str) -> UnzippedEditResponse
```

`crates/jianpu-wasm/src/types.rs`:
```rust
pub struct RangeOut { pub start: usize, pub end: usize }
pub struct PartMeasureRangesOut { pub abbreviation: String, pub ranges: Vec<RangeOut> }

#[serde(tag = "status", rename_all = "camelCase")]
pub enum UnzippedEditResponse {
    Ok { text: String, part_measure_ranges: Vec<PartMeasureRangesOut> },
    UnknownPart,
    Err,
}
```
(Implementation note: reuse the existing `SpanOut { start, end }` type instead of adding a new identical `RangeOut`, since one already exists in `types.rs` with the same shape.)

`crates/jianpu-wasm/src/lib.rs`: replace the `extract_solo_text`/`merge_solo_text` exports with:
```rust
#[wasm_bindgen] pub fn extract_unzipped_text(source: &str) -> UnzippedEditResponse
#[wasm_bindgen] pub fn merge_unzipped_text(source: &str, unzipped_text: &str) -> UnzippedEditResponse
```

## Phase 5 — Tests (separate files per CLAUDE.md convention, not inlined)

- `src/unzipped_edit/tests_capacity.rs` — time-signature changes mid-document, capacity list correctness, Lyrics token-count variant.
- `src/unzipped_edit/tests_extract.rs` — multi-part flattening, byte-range correctness for `part_measure_ranges`.
- `src/unzipped_edit/tests_merge.rs` — round-trip; tail growth beyond original measure count; tail growth using last-known capacity after a `time=` change mid-document; multi-part reconciliation with uneven growth/shrink; blank-part-block deletion; unknown-header error; malformed header; a tuplet measure (asserts documented limitation, not exactness).
- `crates/jianpu-wasm/src/tests_unzipped_edit.rs` — wasm response shape incl. `part_measure_ranges`, replacing old solo-edit wasm tests.

## Phase 6 — Frontend

**`web/src/hooks/usePartToggles.ts`**: remove `soloEditPart` state and `toggleSoloEdit`. Keep everything else unchanged.

**`web/src/App.tsx`**: add local state (same pattern as `editorCollapsed`, which already lives directly in `App.tsx`):
```ts
const [unzippedView, setUnzippedView] = useState(false)
```
Do **not** persist across file switches (unlike part toggles) — reset to `false` on file switch, same as `soloEditPart` was reset today. Remove `soloEditPart`/`toggleSoloEdit` wiring into `PartToggles`/`AppWorkspace`; pass `unzippedView`/`setUnzippedView` to `AppWorkspace` instead.

**`web/src/components/PartToggles.tsx`**: delete the pencil "Solo Edit" segment and `soloEditPart`/`onSoloEditToggle` props entirely.

**`web/src/components/AppWorkspace.tsx`**:
- Swap `extract_solo_text`/`merge_solo_text` imports for `extract_unzipped_text`/`merge_unzipped_text`.
- Replace `soloEditPart: string | null` prop with `unzippedView: boolean` + `onToggleUnzippedView: () => void`.
- Extraction effect now fires whenever `unzippedView` is true (was: whenever `soloEditPart` set), storing `{ text, partMeasureRanges }` instead of `soloText`.
- `handleEditorChange`: branch on `unzippedView` instead of `soloEditPart`; call `merge_unzipped_text(source, value)`.
- `Editor` `path` prop: `unzippedView ? \`${fileId}::unzipped\` : fileId`.
- Add a Zipped/Unzipped toggle control near the existing pane-divider toggle button / editor toolbar (mirror the existing `pane-divider-toggle` button pattern) — a small icon button (e.g. lucide `Columns2`) that calls `onToggleUnzippedView` and reflects current mode.

**`web/src/hooks/useJianpuWorkerRenderRequests.ts`**: this hook already owns the debounced `listMeasureSpans` re-fetch on `source` change — move the `extract_unzipped_text` call here too (replacing the ad hoc effect currently proposed for `AppWorkspace`), so `partMeasureRanges` lives alongside `measureSpans`/`measureSpansRef`. Replace `notifySoloSelection(startLine, endLine)` with `notifyUnzippedSelection(startOffset, endOffset)` — line numbers no longer map to anything meaningful in Unzipped view.

**`web/src/hooks/workerHelpers.ts`**: replace `measureRangeInSoloText` with:
```ts
export function measureRangeInUnzippedText(
  partMeasureRanges: { abbreviation: string; ranges: { start: number; end: number }[] }[],
  cursorOffset: number,
): { start: number; end: number } | null
```
Find the part block containing `cursorOffset` (blocks are contiguous in the generated text — precompute block bounds alongside ranges at extraction time), then binary-search that part's `ranges` for the measure index containing (or nearest, for clicks in inter-token whitespace) `cursorOffset`.

**`web/src/components/Editor.tsx`**: `onSelectionChange` currently reports `(startLineNumber, endLineNumber)` from Monaco (`notifyCursor`, ~line 268-280). Both `model` and `selection` are already in scope there — extend the callback to also compute and pass byte offsets via `model.getOffsetAt(selection.getStartPosition())` / `getOffsetAt(selection.getEndPosition())`, used only by the Unzipped-view caller. Smallest viable change: add an optional second callback prop (e.g. `onSelectionOffsetChange?: (startOffset: number, endOffset: number) => void`) fired alongside the existing line-based one, rather than changing the existing callback's signature (avoids touching the Zipped-view call site).

**`web/e2e/solo-edit-selection-highlight.spec.ts`** → replace with `web/e2e/unzipped-view-selection-highlight.spec.ts`: multi-part source, toggle Unzipped view, click within a specific part's token, assert the same SVG-highlight behavior; add a case confirming a part's tokens wrapped across two visual lines still map correctly (proves newline-insignificance).

## Documentation

- `ARCHITECTURE.md`: replace the Solo Edit section/glossary entry with a "Unzipped View" entry describing `extract_unzipped_text`/`merge_unzipped_text`, the capacity-based repack algorithm, and the tuplet-capacity limitation.
- `syntax.md`: **no changes** — Unzipped text is never stored, the canonical `.jianpu` grammar is unchanged. State this explicitly in the `ARCHITECTURE.md` entry so it's not mistaken for an oversight.

## Known risks / judgment calls to revisit during implementation

1. Lyrics-kind capacity is token-count-based, a real divergence from the beat-capacity model used elsewhere — needs its own scan function and dedicated tests.
2. Beat overflow inside one repacked measure is not split/specially surfaced at merge time — deferred to the next full parse's existing diagnostics.
3. `interleaved_directives.rs` directive-parsing visibility: prefer a thin `pub(crate)` wrapper in `interleaved_parser.rs` over broadening the whole module.
4. Cursor-offset plumbing through `Editor.tsx` is the largest frontend unknown — confirm `model.getOffsetAt` behaves as expected once implemented (should be uncontroversial, Monaco's standard API).
5. Tuplet `resolution_multiplier` capacity scaling is explicitly out of scope for v1.

## Verification

- `cargo test` in the core crate covers Phases 0-3 (capacity scanning, extraction, merge round-trips including growth/shrink/tuplet-limitation cases).
- `cargo test` in `crates/jianpu-wasm` covers Phase 4's response shape.
- Manual: `cargo run -- generate svg simple.jianpu` unaffected (Unzipped view is editor-only, no renderer changes).
- `web/e2e/unzipped-view-selection-highlight.spec.ts` (Playwright) covers the end-to-end UI flow: toggle to Unzipped, edit, verify Zipped source updates and SVG highlight follows cursor.
- Manual smoke test in dev server: open a multi-part file, toggle Unzipped view, shift a note across what was a measure boundary, toggle back to Zipped, confirm the shift produced the expected re-barring and other parts padded correctly.
