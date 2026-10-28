//! Bodies for the `Guest` methods that edit or describe the editor's
//! selected byte ranges (octave shift, slur and tie toggles, the pitch
//! description). Kept at the exact by-value parameter types the `Guest`
//! trait requires (see `mod.rs`'s doc comment), so `needless_pass_by_value`
//! is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;

pub(super) fn shift_range_octave(
    source: String,
    ranges: Vec<ByteRange>,
    delta: i32,
) -> RangeEditResponse {
    let ranges = to_source_edit_ranges(ranges);
    let result = jianpu_generator::source_edit::shift_range_octave(&source, &ranges, delta as i8);
    to_range_edit_response(result)
}

pub(super) fn toggle_range_slur(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    let ranges = to_source_edit_ranges(ranges);
    let result = jianpu_generator::source_edit::toggle_range_slur(&source, &ranges);
    to_range_edit_response(result)
}

pub(super) fn toggle_range_tie(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    let ranges = to_source_edit_ranges(ranges);
    let result = jianpu_generator::source_edit::toggle_range_tie(&source, &ranges);
    to_range_edit_response(result)
}

pub(super) fn describe_selection(
    source: String,
    ranges: Vec<ByteRange>,
) -> Option<PitchDescription> {
    use jianpu_generator::pitch_description as description;
    let ranges = to_source_edit_ranges(ranges);
    description::describe_selection(&source, &ranges).map(|described| match described {
        description::PitchDescription::Note(note) => PitchDescription::Note(NoteDescription {
            letter_name: note.letter_name,
        }),
        description::PitchDescription::Chord(chord) => PitchDescription::Chord(ChordDescription {
            chord_name: chord.chord_name,
            tone_names: chord.tone_names,
            bass_note: chord.bass_note,
            guitar_diagram_svgs: chord.guitar_diagram_svgs,
            piano_diagram_svg: chord.piano_diagram_svg,
        }),
    })
}

fn to_source_edit_ranges(ranges: Vec<ByteRange>) -> Vec<jianpu_generator::source_edit::ByteRange> {
    ranges
        .into_iter()
        .map(|range| jianpu_generator::source_edit::ByteRange {
            start_byte: range.start_byte,
            end_byte: range.end_byte,
        })
        .collect()
}

fn to_range_edit_response(
    result: jianpu_generator::source_edit::RangeEditResult,
) -> RangeEditResponse {
    RangeEditResponse {
        source: result.source,
        ranges: result
            .ranges
            .into_iter()
            .map(|range| ByteRange {
                start_byte: range.start_byte,
                end_byte: range.end_byte,
            })
            .collect(),
    }
}
