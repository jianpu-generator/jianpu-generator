/// Returns the 7 sub-row_heights for a Note/Chord part, in order:
/// [tuplet_bracket, arc, above_dot, note_head, below_dot, half_ul, quarter_ul]
pub(crate) fn note_part_sub_row_heights(base: f32) -> [f32; 7] {
    [
        base * 1.0,  // tuplet bracket (label + short bracket path)
        base * 0.30, // tie/slur arc
        base * 0.25, // above-octave dots
        base,        // note head (main)
        base * 0.25, // below-octave dots
        base * 0.15, // half-beat underline
        base * 0.15, // quarter-beat underline
    ]
}

/// Total height in points for a Note/Chord part's musical sub-rows, omitting
/// the `tuplet_bracket` band's height when the part has no tuplet in this
/// system — so a part without a tuplet doesn't reserve dead space for a
/// bracket it never draws (see `expand_note_part`, which mirrors this by
/// skipping the sub-row itself).
pub(crate) fn note_part_height_pt(base: f32, has_tuplet_bracket: bool) -> f32 {
    let heights = note_part_sub_row_heights(base);
    if has_tuplet_bracket {
        heights.iter().sum()
    } else {
        heights[1..].iter().sum()
    }
}

/// Returns the 4 sub-row_heights for a Chord-symbol-only part, in order:
/// [arc, chord_main, half_ul, quarter_ul]
pub(crate) fn chord_part_sub_row_heights(base: f32) -> [f32; 4] {
    [
        base * 0.30, // tie/slur arc
        base * 0.75, // chord symbol (main)
        base * 0.15, // half-beat underline
        base * 0.15, // quarter-beat underline
    ]
}

/// Extra vertical padding (points) added on top of the lyric font's own
/// measured ascender+descender span, so a syllable's click-target box
/// leaves a small margin around the glyph instead of fitting it exactly
/// flush. An explicit UX choice, not a font-geometry fact — additive (not
/// multiplicative) so it doesn't overstate padding at large font sizes the
/// way a ratio would.
const LYRIC_CLICK_TARGET_VERTICAL_PADDING: f32 = 6.0;

/// Height for a lyric verse row (and its click-target rect — see
/// `coordinate_resolver::highlights::resolve_lyric_click_target`).
/// `row_height * 1.5` is the row's normal vertical rhythm — unchanged from
/// before, so a row with no CJK text and no `lyrics_font_size` override
/// renders at exactly the height it always has. The other side of the
/// `max` is a floor sized to actually contain the glyph: the pinned lyric
/// font's real ascender+descender span at `font_size`, measured at runtime
/// via `ttf_parser` (`font_metrics::lyric_vertical_extent`) rather than a
/// hardcoded ratio, plus `LYRIC_CLICK_TARGET_VERTICAL_PADDING` for a small
/// click-target margin — only the larger of the two applies, so this floor
/// only kicks in once `font_size` (the largest resolved size — Latin or CJK,
/// see `RenderConfig::lyric_font_size`/`lyric_cjk_font_size` — among the
/// row's own syllables) grows large enough, relative to `row_height`, to
/// actually risk overflowing the row's normal height.
pub(crate) fn lyric_row_height(row_height: f32, font_size: f32) -> f32 {
    (row_height * 1.5).max(
        crate::font_metrics::lyric_vertical_extent(font_size) + LYRIC_CLICK_TARGET_VERTICAL_PADDING,
    )
}

pub(crate) fn decoration_row_height(base: f32) -> f32 {
    base * 1.5
}

pub(crate) fn separator_row_height() -> f32 {
    4.0
}

/// One row of blank vertical space, used above the sequence line.
pub(crate) fn header_gap_row_height(base: f32) -> f32 {
    base
}

pub(crate) fn header_title_row_height(base: f32) -> f32 {
    base * 0.80
}

pub(crate) fn header_subtitle_author_row_height(base: f32) -> f32 {
    base * 2.625
}

pub(crate) fn header_part_list_row_height(base: f32) -> f32 {
    base * 0.9
}
