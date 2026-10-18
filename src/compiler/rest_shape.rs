//! How a rest is drawn beyond its `0` glyph: its underlines and the dashes
//! that extend it. Shared by a notes line's rests (`part_slice::compile_rest`)
//! and a rest bracket in a chord-lyric line (`lyric_rest`), so both draw a
//! given duration identically.

/// Underlines drawn under a rest of `duration` columns: two for a rest one
/// beat-division long (`0=`), one for `0_`, none otherwise.
pub(super) fn rest_underline_count(duration: u32, multiplier: u32) -> u32 {
    if duration == multiplier {
        2
    } else if duration == 2 * multiplier {
        1
    } else {
        0
    }
}

/// Column offsets, from the rest's own column, of the `NoteDash`es that extend
/// it. A dotted (or double-dotted) rest's own written duration is one full
/// dotted beat, so its `-.`/`-..` extensions land on 6-/7-quarter-beat
/// boundaries rather than the 4-quarter-beat ones a plain `-` uses (see the
/// matching comment in `part_slice_unit::compile_unit`).
pub(super) fn rest_dash_offsets(
    duration: u32,
    dotted: bool,
    double_dotted: bool,
    multiplier: u32,
) -> Vec<u32> {
    let beat = if double_dotted {
        7 * multiplier
    } else if dotted {
        6 * multiplier
    } else {
        4 * multiplier
    };
    (beat..duration).step_by(beat as usize).collect()
}
