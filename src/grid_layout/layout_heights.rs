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

pub(crate) fn lyric_row_height(base: f32) -> f32 {
    base * 1.5
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
