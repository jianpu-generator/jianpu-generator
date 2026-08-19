# Make dot and accidental column weights match their actual rendered geometry

## Context

`layout_spacing.rs`'s per-column weight/rod computation
(`column_weight`/`column_rod`, see **Rod and spring** in `ARCHITECTURE.md`)
is supposed to track what the renderer actually draws, so a column's
allocated width never falls short of its content's real footprint. Two of
its glyphs currently don't:

### The augmentation dot (`·`)

`dotted_extra_weight()` estimates the space needed for a dotted
note/rest/dash's dot(s) as a flat guess, unconnected to any font
measurement:

```rust
fn dotted_extra_weight(dotted: bool, double_dotted: bool, config: &RenderConfig) -> f32 {
    ...
    dot_count as f32 * 2.0 * config.row_height as f32 * 0.06
}
```

Its doc comment claims this "matches the dot's real rendered diameter
(`dot_radius = row_height * 0.06` in those renderers)" — but no such
`row_height`-based radius exists anywhere in
`renderer/new_renderer/glyph_renderers.rs` any more. The dot is drawn as
centered (`TextAnchor::Middle`) text, positioned by an offset formula that
has nothing to do with `row_height`:

- `render_note_head`/`render_rest`: first dot at
  `center + note_number_width * 1.5`, further dots spaced
  `note_number_width * DOT_SPACING_RATIO` (`0.5`) apart, at
  `config.notes_font_size()`.
- `render_note_dash`: same offset/spacing formula (in terms of
  `note_number_width`), but at the fixed `NOTE_DASH_FONT_SIZE` (`12.0`)
  instead of the scaled notes font size.
- `render_chord_symbol`: a different offset formula —
  first dot at `text_width + chords_font_size * 0.4`, spacing
  `chords_font_size * 0.4` — at `config.chords_font_size()`.

At the default config (`note_number_width: 12`), the first dot's anchor
alone sits `12 * 1.5 = 18pt` right of the glyph's center — i.e. `24pt`
right of the column's left edge — while `column_weight` +
`dotted_extra_weight` together allocate only ~`14.4pt` for a dotted note
(`note_glyph_weight` ~`10.8pt` + the guessed `3.6pt`). The estimate isn't
just the wrong *size* for the dot glyph, it doesn't even reach the
*position* the dot is actually drawn at. That gap is what's producing the
visible overlap/crowding.

### The accidental (`♯`/`♭`)

`accidental_extra_weight()` is mostly real — it measures the symbol's
actual font-advance width via `font_metrics::monospace_text_width` at the
same font/size the renderer uses:

```rust
let reach = config.note_number_width as f32
    * (ACCIDENTAL_LEFT_GAP_RATIO + ACCIDENTAL_RIGHT_PADDING_RATIO)
    + font_metrics::monospace_text_width(symbol, config.notes_font_size() * 1.25);
```

but the gap ratios bracketing that measured width
(`ACCIDENTAL_LEFT_GAP_RATIO = 0.2`, `ACCIDENTAL_RIGHT_PADDING_RATIO =
1.0`) are hand-tuned constants, not derived from the glyph's own metrics —
`ACCIDENTAL_RIGHT_PADDING_RATIO` alone reserves a full `note_number_width`
of trailing clearance regardless of the symbol's actual right-side
bearing. `render_note_head` positions the glyph's left edge at `elem.x +
note_number_width * ACCIDENTAL_LEFT_GAP_RATIO`, so the left ratio is at
least consistent with the renderer; there's no equivalent check that the
right ratio's `1.0` is actually needed rather than just generous.

`monospace_glyph_left_bearing()` (`font_metrics.rs`) already exists and is
already used this way for note heads/rests/chord symbols/dashes in
`coordinate_resolver::resolve.rs` (compensating each glyph's own built-in
left-side bearing) — the accidental path is the odd one out still using a
flat ratio instead.

## Approach

### 1. Give dots a position-and-size-aware reach, not a flat guess — `layout_spacing.rs`

Replace `dotted_extra_weight`'s single row_height-based guess with a
reach computation mirroring `accidental_extra_weight`'s pattern (`reach -
note_glyph_weight`), parameterized per call site so each matches its own
renderer offset formula:

```rust
/// How far right of a glyph's own left edge its rendered dot(s) reach,
/// mirroring the offset/spacing formula `render_note_head`/`render_rest`/
/// `render_note_dash` actually draw at (`center + note_number_width *
/// 1.5`, further dots `note_number_width * DOT_SPACING_RATIO` apart,
/// `TextAnchor::Middle` so a dot's right edge sits half its own advance
/// width past its anchor).
fn note_ish_dot_reach(
    dot_count: u32,
    note_number_width: f32,
    dot_font_size: f32,
) -> f32 {
    if dot_count == 0 {
        return 0.0;
    }
    let last_dot_anchor = note_number_width * 1.5
        + (dot_count - 1) as f32 * note_number_width * font_metrics::DOT_SPACING_RATIO;
    last_dot_anchor + font_metrics::monospace_char_advance_width('\u{b7}', dot_font_size) / 2.0
}
```

(The `/ 2.0` accounts for `TextAnchor::Middle` centering the glyph's
advance box on its anchor — the same reasoning `resolve.rs` already
applies via `monospace_glyph_left_bearing` for *left*-anchored glyphs,
mirrored here for a middle-anchored one.)

Then, at each of the three call sites currently sharing
`dotted_extra_weight`, compute `note_ish_dot_reach(...)` at that call
site's own font size and subtract what `column_weight`'s base term
already covers (same `.max(0.0)` floor `accidental_extra_weight` uses):

- `NoteHead`/`Rest`: `note_ish_dot_reach(dot_count, config.note_number_width as f32, config.notes_font_size())`,
  reach measured relative to `note_glyph_weight(config)`.
- `NoteDash`: same formula, but at `font_metrics::NOTE_DASH_FONT_SIZE`
  instead of `config.notes_font_size()`, reach measured relative to
  `dash_weight()`.
- `ChordSymbol`: keep its own separate formula (offset
  `text_width + chords_font_size * 0.4`, spacing `chords_font_size *
  0.4`) — it already has a distinct positioning scheme, don't force it
  through `note_ish_dot_reach`.

This turns `dotted_extra_weight` from one flat function into per-element
reach math threaded through `column_weight`'s existing match arms — check
whether keeping a single `dotted_extra_weight(kind, dotted, double_dotted,
config)` dispatcher (matching on `ElementContent`) or inlining the reach
call directly in each `column_weight` arm reads better once written; the
match arms already have every value (`note_number_width`, font size,
`text` for the chord case) in scope.

### 2. Base the accidental's gaps on the glyph's real metrics — `layout_spacing.rs`

Replace `ACCIDENTAL_LEFT_GAP_RATIO`'s contribution with the symbol's own
measured left-side bearing (matching what `render_note_head` actually
starts drawing at, once its `elem.x + note_number_width *
ACCIDENTAL_LEFT_GAP_RATIO` offset — if this plan changes it — reads off
`monospace_glyph_left_bearing` too, so the two stay in lockstep):

```rust
let symbol_font_size = config.notes_font_size() * 1.25;
let left_gap = font_metrics::monospace_glyph_left_bearing(symbol.chars().next().unwrap(), symbol_font_size);
let reach = left_gap
    + font_metrics::monospace_text_width(symbol, symbol_font_size)
    + config.note_number_width as f32 * ACCIDENTAL_RIGHT_PADDING_RATIO;
```

Whether `ACCIDENTAL_RIGHT_PADDING_RATIO` should also become
metrics-derived (e.g. the glyph's own right-side bearing, `advance_width -
bbox.x_max`) or stay a deliberate flat clearance constant needs a call at
implementation time — unlike the dot's diameter guess or the left gap,
this one isn't provably wrong, just unverified. Recommend leaving it as a
named, documented constant unless a visible crowding/looseness problem
shows up specifically on the trailing side.

If `render_note_head`'s draw-site offset (`ACCIDENTAL_LEFT_GAP_RATIO`)
also moves to `monospace_glyph_left_bearing`, that's a renderer-side
change in `glyph_renderers.rs`, not just a layout-weight one — confirm
both sides move together so the accidental's real position matches what
the reach was computed against, the same discipline this plan is applying
to the dot.

### 3. Tests

- `src/grid_layout/tests_measure_spacing.rs` (or a new
  `tests_dot_spacing.rs`/`tests_accidental_spacing.rs`-adjacent file,
  matching this codebase's "tests live in separate files" convention) —
  assert a dotted note's/rest's/dash's column rod is now large enough to
  clear the dot's actual rendered right edge (recompute the expected
  reach independently in the test, don't just assert against the
  production formula's own output).
- `src/grid_layout/tests_accidental_spacing.rs` (already exists) — extend
  with a case confirming the new left-bearing-based gap still clears the
  symbol without regressing the existing right-padding assertions.
- `src/font_metrics.rs` inline tests — if `note_ish_dot_reach` (or
  whatever it ends up named) lands in `font_metrics.rs` instead of
  `layout_spacing.rs`, give it the same kind of real-vs-fallback-glyph
  coverage `monospace_glyph_left_bearing`'s tests already have.

### 4. Verification

1. `cargo test` — updated/new spacing tests pass, no regression in
   existing measure-spacing/accidental-spacing suites.
2. Visual check:
   ```
   cargo run -- generate svg <a fixture with dotted notes and sharp/flat accidentals>
   ```
   Confirm a dotted note's dot no longer crowds/overlaps the following
   column, and an accidental's spacing looks unchanged (or intentionally
   tighter, if `ACCIDENTAL_RIGHT_PADDING_RATIO` is revisited). Delete
   generated SVGs afterward, don't commit them.

## Critical files

- `src/grid_layout/layout_spacing.rs` — `dotted_extra_weight`,
  `accidental_extra_weight`, `column_weight`, `column_rod`.
- `src/font_metrics.rs` — `monospace_char_advance_width`,
  `monospace_glyph_left_bearing`, `DOT_SPACING_RATIO`,
  `ACCIDENTAL_LEFT_GAP_RATIO`/`ACCIDENTAL_RIGHT_PADDING_RATIO`.
- `src/renderer/new_renderer/glyph_renderers.rs` — `render_note_head`,
  `render_rest`, `render_chord_symbol` (renderer-side offset formulas
  this plan's weight math must keep matching).
- `src/renderer/new_renderer/glyph_renderers_note_dash.rs` —
  `render_note_dash`.
