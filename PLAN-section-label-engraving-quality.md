# Plan: bring section-label box sizing up to engraving-software quality

## Background

The section-label bounding box (added in a prior change: a bordered `<rect>`
drawn behind the `label="..."` text on a directive line — see
`TransparentRectRole::SectionLabelBackground` in
`src/renderer/new_types.rs`) is currently sized and positioned with
heuristics, not real measurement:

- `char_advance_width` / `section_label_box_width` in
  `src/renderer/new_renderer.rs` estimate the label's rendered width by
  bucketing each character into one of three fixed point-widths (CJK / ASCII
  letter / ASCII digit-punctuation-space), rather than measuring actual
  glyph advances.
- The label text is one `tspan` inside a single shared `<text>` element that
  also contains the bar number, key, bpm, time signature, and navigation
  markers (`directive_line_content` in
  `src/coordinate_resolver/content_conversion.rs`). A `label_x_offset: f32`
  field was added to `AbsoluteContent::DirectiveLine`
  (`src/compositor/types.rs`) to estimate — again by character-bucket
  heuristic via `estimate_span_width` — how far into that shared string the
  label starts, so the box doesn't overlap the bar number. This works but is
  structurally fragile: any future span inserted before the label needs its
  own width estimate threaded through the same way.
- The SVG's directive-line text has no pinned font
  (`font-family="sans-serif"`) — actual glyph shapes/widths depend on
  whatever font the viewer (browser, PDF viewer) substitutes, so even a
  perfect Rust-side width calculation can't guarantee pixel-accuracy.

This plan captures 5 follow-up tasks, adapted from how real engraving
software (Dorico, Sibelius, Finale, MuseScore) solves the same class of
problem, discussed in the conversation that produced this file. Each task
is independently valuable and can be picked up on its own — read this file
top to bottom for context, then execute one task at a time.

**Suggested execution order** (later tasks build on earlier ones, but each
has its own acceptance criteria and can be done standalone if needed):

1. Pin/embed a specific font for directive-line text
2. Structurally separate the section label from the shared directive-line text run
3. Replace the character-bucket heuristic with real font-metrics measurement
4. Two-pass layout for the whole directive line
5. Express box padding in relative (font-size-scaled) units, not fixed points

---

## Task 1: Pin/embed a specific font for directive-line text

**Problem:** `directive_line_content`/`grid_text_to_absolute` in
`src/coordinate_resolver/content_conversion.rs` emit
`font: FontFamily::SansSerif` (see `src/compositor/types.rs` for
`FontFamily`), which the serializer (`src/serializer/mod.rs`) writes out as
`font-family="sans-serif"`. Any measurement done Rust-side can only ever be
an approximation of what a *specific* viewer substitutes for "sans-serif" —
Chrome, Firefox, a PDF viewer, and `wkhtmltopdf`-style renderers can all
pick different fonts.

**Goal:** Decide on and embed/reference one specific font file for
directive-line text (bar numbers, section labels, key/bpm/time signature,
navigation markers) so that whatever computes glyph widths (see Task 3) is
measuring the *same* font that actually renders. This is a prerequisite for
Task 3's measurement to be trustworthy.

**Notes:**
- Check `fonts/` at the repo root — there may already be a bundled font
  used elsewhere (e.g. for note heads or PDF export via `printpdf`) that
  should be reused here for consistency rather than introducing a second
  font.
- Check how PDF export (`--features pdf`, search for `printpdf` usage) picks
  its font today — the SVG and PDF renderers should probably agree on the
  same font family for directive-line text.
- For the SVG output, "pinning" means embedding the font (as a `<style>`
  `@font-face` with base64 data, or referencing a `font-family` name that's
  guaranteed present) rather than a generic `sans-serif` alias — otherwise
  the viewer can still substitute.
- Out of scope: changing the font used for note heads/lyrics/other text —
  keep this scoped to directive-line text unless the codebase already
  shares one font for everything.

---

## Task 2: Structurally separate the section label from the shared directive-line text run

**Problem:** Right now the bar number, section label, key, bpm, time
signature, and navigation markers are all `TextSpan`s inside one `<text>`
element (`directive_line_content` in
`src/coordinate_resolver/content_conversion.rs`, consumed by
`render_directive_line` in `src/renderer/new_renderer.rs`). Because they're
one text run, positioning the label's *own* box requires knowing how wide
everything before it is — hence the `label_x_offset` field threaded through
`AbsoluteContent::DirectiveLine`. In real engraving software, a rehearsal
mark (the boxed section label) is an independent, separately-positioned
object from the measure number — they don't share a text run, so neither
needs to know the other's width.

**Goal:** Render the section label as its own `<text>`/group, positioned
independently of the bar-number/key/bpm/markers text run, rather than as a
`tspan` inside it. This removes the need for `label_x_offset` entirely (or
reduces it to a small fixed gap, not a measured offset into a shared
string).

**Notes:**
- `AbsoluteContent::DirectiveLine` (`src/compositor/types.rs`) currently
  bundles `label: Option<String>` + `spans: Vec<TextSpan>` +
  `segno_icon_offset` + `label_x_offset` + `apply_row_offset` into one
  variant. Consider whether the label deserves its own
  `AbsoluteContent::SectionLabel { label, x, y }`-style variant instead,
  rendered as a sibling element rather than folded into the directive-line
  text spans.
- Watch `directive_row_offset` handling (`RenderConfig.directive_row_offset`
  in `src/render_config.rs`, documented in `syntax.md`) — both the label and
  the rest of the directive line need to keep moving together under that
  offset; don't lose that behavior when splitting them apart. See the
  existing test `labeled_directive_line_moves_label_background_text_and_segno_together`
  in `src/renderer/new_tests.rs`.
- Decide where the now-independent label element sits horizontally relative
  to the rest of the line — e.g. still visually first, immediately after
  where the bar number ends, but computed via a small fixed gap constant
  rather than a measured `label_x_offset`.
- Update `ARCHITECTURE.md` per `CLAUDE.md`'s rule if this changes a key
  type (`AbsoluteContent::DirectiveLine`'s shape) or introduces a new one.
- Existing tests to update: `src/renderer/new_tests.rs` (all the
  `AbsoluteContent::DirectiveLine { .. }` literal constructions, plus
  `label_background_starts_past_a_preceding_bar_number` and the CJK-width
  test), `src/coordinate_resolver/tests_sequence_line.rs`.

---

## Task 3: Replace the character-bucket heuristic with real font-metrics measurement

**Problem:** `char_advance_width` in `src/renderer/new_renderer.rs` buckets
every character into one of three fixed widths (CJK ≈ 13pt, ASCII letter ≈
7pt, digit/punctuation/space ≈ 6pt). This is a rough approximation — real
proportional fonts vary glyph advance width per-character (e.g. `i` vs `M`),
and bold/italic styling (used for section labels specifically, see
`section_label_span` in `content_conversion.rs`) shifts widths further.

**Goal:** Measure the label's actual rendered width using real font metrics
from the font pinned in Task 1, rather than a heuristic. In Rust, this
means loading the font file with a font-metrics/shaping crate (e.g.
`ttf-parser`, `ab_glyph`, or `rustybuzz` for full shaping incl. kerning/CJK)
and summing real glyph advances for the label string at the label's actual
font size/weight/style (12pt bold italic, per `section_label_span`).

**Notes:**
- This depends on Task 1 (a pinned font file to load metrics from) — doing
  this before Task 1 just heuristic-optimizes against an assumption that
  may not match any actual renderer.
- `estimate_span_width` in `content_conversion.rs` (used for the Segno
  glyph's inline x-offset) has the same class of problem and could be
  upgraded to the same measurement approach in the same pass, though it's
  not required for this task — scope to the section-label box first, note
  the Segno offset as a possible follow-up.
- New dependency: check `Cargo.toml` for an existing font-parsing crate
  before adding a new one (search for `ttf`, `font`, `ab_glyph`, `rusttype`,
  `fontdue`, `harfbuzz`, `rustybuzz`).
- Existing tests to update:
  `cjk_label_gets_a_wider_background_than_an_equal_length_ascii_label` in
  `src/renderer/new_tests.rs` currently only asserts a relative comparison
  (CJK wider than ASCII) — once real metrics are used, consider asserting
  closer-to-exact widths for a known font/string.

---

## Task 4: Two-pass layout for the whole directive line

**Problem:** Currently, spans are built and positions/offsets estimated in
a single pass inside `directive_line_content`
(`src/coordinate_resolver/content_conversion.rs`) — width estimates
(`estimate_span_width`) are computed *while* spans are being assembled, not
after real shaping. This is the same "measure while building" pattern that
caused the original bar-number/label misalignment bug this plan follows
up on.

**Goal:** Once Task 3 exists (a real measurement function), restructure
`directive_line_content` into two explicit passes: (1) build the list of
logical elements for the line (bar number, label, key, bpm, time signature,
navigation markers, Segno) with their content/style but no positions; (2)
walk that list once, measuring each element with the Task 3 measurement
function and assigning each an x position based on the running total of
real measured widths before it. Both the Segno glyph's offset and the
section label's position (if not already fully decoupled by Task 2) should
come out of this same pass, rather than each having its own bespoke
estimate function.

**Notes:**
- This mostly subsumes/replaces `estimate_span_width` and
  `push_navigation_marker_spans`'s offset-tracking role — read both before
  starting, in `src/coordinate_resolver/content_conversion.rs`.
- If Task 2 (structural separation) is already done, this task's scope
  shrinks to just the remaining shared text run (bar number, key, bpm, time
  signature, markers, Segno) — the label no longer needs to participate.
- Good place to introduce a small internal struct (not a tuple, per
  `CLAUDE.md`'s "never use tuple in new data structures" rule) representing
  one measured, positioned element, e.g. `{ span: TextSpan, x_offset: f32 }`.

---

## Task 5: Express box padding in relative (font-size-scaled) units

**Problem:** `SECTION_LABEL_BOX_PADDING` in `src/renderer/new_renderer.rs`
is a fixed `4.0` (points), and `bg_height` is a fixed `18.0`. Real engraving
software expresses these kinds of margins relative to the notation's overall
scale (e.g. "staff spaces"), so the box scales correctly if the score is
ever rendered at a different size — this codebase's closest equivalent is
probably the label's own `font_size` (currently always `12.0`, hardcoded in
`section_label_span`).

**Goal:** Derive `SECTION_LABEL_BOX_PADDING` and `bg_height` from the
label's font size (e.g. `padding = font_size * 0.33`,
`height = font_size * 1.5`) instead of hardcoded point constants, so the
box would scale correctly if `section_label_span`'s `font_size` ever becomes
configurable instead of a hardcoded `12.0`.

**Notes:**
- Low risk, no new dependencies — this is a pure refactor of existing
  constants into ratios. Good candidate to do first if you want a quick win
  before tackling Tasks 1-4, even though it's listed last (it doesn't
  depend on the others).
- Check whether `section_label_span`'s `font_size: 12.0` is ever going to
  become configurable (e.g. via a metadata directive) — if genuinely never,
  this task's value is mostly about code clarity (documenting *why* the
  padding is `4.0`) rather than actual scaling behavior. Worth confirming
  intent before investing time here.
- Existing tests asserting exact point values (e.g.
  `assert_eq!(background.x, text.x - 4.0)` in `src/renderer/new_tests.rs`)
  will need updating to compute the expected value from the same ratio
  rather than a hardcoded literal.

---

## Out of scope for all 5 tasks

- Changing the visual style of the box itself (stroke color/width, corner
  radius) — that was already decided (black stroke, `rx="2"`) in the prior
  change.
- Any font changes to note heads, lyrics, or other non-directive-line text.
- PDF-specific rendering path, except where Task 1 explicitly calls out
  checking for font consistency with it.
