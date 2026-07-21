# Tuplet syntax implementation plan

Branch: `feature/tuplets`. This file tracks atomic, sequential steps. Each step is meant to
be implemented in its own fresh-context session/commit. **After finishing a step, check its
box, commit, and stop** (unless told to continue) — the next session picks up from here.

Full background/design rationale lives in git history of this file's first commit and in the
original plan below. Read the "Full plan" section at the bottom for design details not
repeated in the step descriptions.

## Steps

- [x] **Step 1 — Lexer: `{`/`}` tokens**
  `src/parser/score/timed_parser/timed_lexer.rs`, `timed_lexer/directive_lexing.rs`:
  add `LBrace { num: u32, den: Option<u32> }` / `RBrace` tokens (mirror `LParen`/`RParen`).
  Add `try_lex_tuplet_open` (parallel to `try_lex_time_signature`) recognizing `N:{` or
  `N:M:{` at a word boundary, consuming digits/colons/brace together. Update
  `timed_recursive_descent_parser.rs::parse_timed_unit`'s `text.find(...)` / `unit_end_abs`
  boundary scanners to also stop at `{`/`}`. Add lexer unit tests in
  `timed_lexer_tests.rs`.

- [ ] **Step 2 — Groups: `TupletStack`, ratio resolution, errors**
  `src/parser/score/timed_parser/groups.rs`: add `TupletStack`/`TupletFrame { note_count,
  segment_start, num, den }` alongside `GroupStack`, with `open_tuplet`/`close_tuplet`.
  `src/parser/score/timed_parser/timed_recursive_descent_parser.rs` /
  `timed_recursive_descent_parser/group_and_repeat.rs`: generalize `parse_atoms`'s
  `stop_at_rparen: bool` to also stop at `RBrace` when inside a tuplet. No cross-line tuplets:
  unclosed `{` at end of line is a hard parse error (no `finalize_open_frames` equivalent).
  Implicit ratio table `2→3, 3→2, 4→3, 5→4, 6→4, 7→4, 9→8`; anything else without explicit
  `:M` is `RecoverableError::tuplet_ambiguous_ratio`. Add `RecoverableErrorKind` variants
  `TupletAmbiguousRatio`, `TupletNoteCountMismatch` in `src/error/recoverable_kind.rs` +
  constructors in `recoverable_error.rs` (style of `duration_unexpected_char`). Note-count
  mismatch on close is a recoverable error. Add tests in a new
  `src/parser/score/timed_parser/tuplet_tests.rs`.

- [ ] **Step 3 — AST: `TupletInfo` on parsed nodes**
  `src/ast/parsed.rs`: add `pub struct TupletInfo { pub num: u32, pub den: u32 }` (no tuples)
  and `pub tuplet: Option<TupletInfo>` field to `ParsedNote`, `ParsedChordNote`,
  `ParsedPercussionHit`, `ParsedRest`. Wire it up in
  `timed_recursive_descent_parser.rs::parse_timed_unit` right after `duration_meta.duration`
  is computed, before `H::to_event(...)`, and in
  `group_and_repeat.rs::parse_repeat_unit` for repeat atoms opened inside a tuplet. Update all
  other construction sites of these structs (test helpers, desugar.rs, etc.) to set
  `tuplet: None` — grep for struct-literal construction of these 4 types.

- [ ] **Step 4 — Grouped IR + new rescale pass**
  `src/ast/grouped_notes.rs`: add `resolution_multiplier: u32` (default 1) to
  `GroupedMeasure` (find this type — may need to check `src/grouper/` for where measures are
  assembled if not in this file), and `tuplet: Option<TupletInfo>` on `GroupedNote`,
  `GroupedRest`, `GroupedChordNote`, `GroupedPercussionHit`.
  New file `src/grouper/tuplet_rescale.rs` (check actual grouper module path first): a pass
  that runs on one line's `Vec<Spanned<ScoreEvent>>` *before* `PartGrouper` consumes them.
  Scans for `TupletInfo` tags (including nested), computes `multiplier = LCM(all N's found)`,
  multiplies every event's `duration` by `multiplier` (plain notes too), and for
  tuplet-tagged events further multiplies by `den/num`. Returns the multiplier alongside the
  rescaled events for attachment to the produced `GroupedMeasure`. Add grouper-level tests:
  a tuplet correctly filling a beat (e.g. `{3:1_1_1_}` filling exactly 1 beat in 4/4).

- [ ] **Step 5 — Thread `multiplier` through grouping/compiler**
  `PartGrouper` (find in `src/grouper/`), `src/grouper/grouping.rs`,
  `src/compiler/part_slice.rs`, `src/compiler/part_slice_unit.rs`: every literal that
  hardcodes "quarter-beat = 4 sixteenths" (`4`, `8`, `16`, `3`, `1`) becomes
  `BASE_CONST * multiplier`, with `multiplier` threaded in as a parameter/field. Both
  `handle_extension` (`-` suffix) in `PartGrouper` and the equivalent `+= 4` in
  `grouping.rs` become `+= 4 * multiplier`. `column: u32` in `part_slice_unit.rs`/
  `part_slice.rs` is a pure additive grid index — safe to scale uniformly per-measure. Add
  half-bar-boundary / dotted-eighth-tail tests around a tuplet to confirm rules still fire
  correctly.

- [ ] **Step 6 — Grid width + MIDI ticks divide back out**
  `src/grid_layout/layout_spacing.rs`: `column_weight` (~line 24) and any other grid-width
  literal (`MULTI_MEASURE_REST_WIDTH` etc. in `compiler/types.rs`/`layout_spacing.rs`) must
  divide by that measure's `multiplier` so pixel width reflects real musical content, not
  inflated column count.
  `src/midi/midi_notes.rs::duration_to_ticks`: becomes `quarter_beats * TPQ / (4 *
  multiplier)`. Known limitation: multipliers whose prime factors don't divide 480 (e.g.
  involving 7 or 11) round — document, don't bump TPQ.
  Add a MIDI test: tick output for a triplet measure sums to the same total ticks as the
  equivalent non-tuplet measure.
  **Verify**: existing demo files (none use tuplets) render pixel-identically before/after —
  multiplier defaults to 1, should be a no-op diff. Diff generated SVG output for
  `demo/01-pitches.jianpu` etc. before/after this step.

- [ ] **Step 7 — Rendering: tuplet brackets**
  Follow the slur-arc pipeline as template (`SlurSpan` → `resolve_slur_spans` in
  `src/grid_layout/slur_placement.rs` → `resolve_span_marking` in
  `src/coordinate_resolver/resolve.rs` → `render_tie_or_slur` in
  `src/renderer/new_renderer/glyph_renderers.rs`):
  1. `grid_layout`: derive `TupletSpan { from_column, to_column, part_index, label: String }`
     per contiguous tuplet group (label = `N` digit e.g. `"3"`), as a `GridElement` with new
     `GridContent::TupletBracket { label }` variant.
  2. `src/grid_layout/layout.rs::note_part_sub_row_heights`: prepend `tuplet_bracket` to the
     row order (`[arc, above_dot, note_head, below_dot, half_ul, quarter_ul]` →
     `[tuplet_bracket, arc, ...]`) so brackets stack above slur arcs.
  3. `src/coordinate_resolver/resolve.rs`: resolve grid columns to pixel x/width like
     `resolve_span_marking` does for `TieOrSlur`, producing `AbsoluteContent::TupletBracket {
     label, width }`.
  4. `src/renderer/new_renderer/glyph_renderers.rs`: add `render_tuplet_bracket` — two short
     vertical ticks + horizontal line (or shallow bracket path) spanning `width`, with an
     `SvgKind::Text` label centered above the midpoint (copy `SvgKind::Text` construction from
     `render_note_head`). Dispatch alongside `render_tie_or_slur` in
     `src/renderer/new_renderer.rs`.
  Tuplets don't span line/system breaks, so no `TieOrSlurTail`/`Head` cross-system-break
  equivalent is needed.

- [ ] **Step 8 — Docs**
  `syntax.md`: new "Tuplets" subsection under Duration suffixes — `{N:notes}`,
  `{N:M:notes}`, implicit-ratio table, nesting with `(...)`, no-cross-line-tuplets rule.
  `ARCHITECTURE.md`: document `TupletInfo`, `resolution_multiplier` on `GroupedMeasure`,
  `GridContent::TupletBracket`/`AbsoluteContent::TupletBracket`, and the tuplet-rescale pass
  in the pipeline-layers section + glossary.

- [ ] **Step 9 — E2E fixture + final verification**
  New `demo/12-tuplets.jianpu` (check existing numbered `demo/*.jianpu` files for the next
  free number first) exercising a triplet, a duplet, and a quintuplet.
  `cargo run -- generate svg demo/12-tuplets.jianpu` — visually confirm brackets render above
  the note row without colliding with slur arcs, octave dots, or beam underlines.
  If the project has SVG snapshot/golden-file tests under `tests/`, add one for this fixture.
  Run full `cargo test` and the e2e suite. Delete this file (`TUPLET_PLAN.md`) once all steps
  are checked off and merged, or leave it — check with the user.

## Full plan

(Original approved plan, kept verbatim for reference — see step descriptions above for the
condensed, sequenced version of the same content.)

### Context

The `.jianpu` grammar has no way to notate tuplets (triplets, duplets, quintuplets...).
Duration is currently a plain `u32` "quarter-beat" count (1 = a sixteenth note) used
identically across parsing, measure-capacity validation, beat-grouping rules, SVG
grid-column layout, and MIDI tick export. Tuplets need fractional beat subdivisions (e.g. an
eighth-note triplet = 4/3 quarter-beats), which this integer grid cannot represent today.

Agreed syntax:
- `{N:notes}` — N notes in the time normally taken by the implied "against" count (standard
  tuplet meaning: 3-in-2, 2-in-3, 5-in-4, 6-in-4, 7-in-4, 9-in-8, ...).
- `{N:M:notes}` — explicit override, N notes in the time of M notes of the same written
  value.
- Any `N` without a standard implied ratio and without an explicit `:M` is a parse error
  ("ambiguous tuplet ratio, use `{N:M:...}`").
- Scope: fully general N-tuplets, not just triplets/duplets. Internal grid resolution is
  rescaled dynamically per measure (quintuplet measure gets a ×5 grid, combined
  triplet+quintuplet gets ×15, etc.) without hardcoding a single global multiplier.

### Design: per-measure resolution rescaling (no whole-file pre-scan)

`PartGrouper` already processes one measure/line at a time and the recursive-descent parser
materializes a full list of `ScoreEvent`s per line before grouping happens. Insert a new
rescaling pass per measure, between "parser produces `ScoreEvent`s for this line" and
"`PartGrouper` consumes them":

1. Parse the line normally. `duration.rs`'s suffix parser (`_`, `=`, `.`, `-`) stays
   untouched — it keeps producing base-scale quarter-beats exactly as today, independent of
   tuplets.
2. A tuplet-carrying event gets an attached `TupletInfo { num, den }` set by the parser when
   inside an open `{...}` bracket — no arithmetic yet, just the ratio tag.
3. Before handing the line's events to `PartGrouper`, a new pass scans for any `TupletInfo`,
   computes `multiplier = LCM(all N's found, including nested tuplets)`, and:
   - Multiplies every event's `duration` in that line by `multiplier` (plain notes too).
   - For tuplet-tagged events, further multiplies by `den/num` (guaranteed to divide evenly).
   - Attaches the resulting `multiplier` to the produced `GroupedMeasure`.
4. `PartGrouper`, `grouping.rs`, `compiler/part_slice*.rs` keep existing logic unchanged, but
   every literal hardcoding "quarter-beat = 4 sixteenths" is expressed as
   `BASE_CONST * multiplier`.
5. Two places divide back out using the measure's `multiplier` so non-tuplet music renders
   pixel-identically to today: `layout_spacing.rs` column weight, and
   `midi_notes.rs::duration_to_ticks`.
6. `handle_extension` (`-` suffix) needs `+= 4 * multiplier`.

### Testing

Parser-level (implicit ratio table, explicit `{N:M:...}`, ambiguous-N error, note-count
mismatch error, nesting with `(...)` both directions, unclosed-tuplet error), grouper-level
(measure-capacity validation, half-bar-boundary/dotted-eighth-tail around a tuplet), e2e
(`demo/12-tuplets.jianpu` rendered via `cargo run -- generate svg`), MIDI export (tick sum
round-trip check).
