//! Bodies for the `Guest` methods that edit or describe the editor's
//! selected byte ranges (octave shift, slur and tie toggles, the pitch
//! description). Kept at the exact by-value parameter types the `Guest`
//! trait requires (see `mod.rs`'s doc comment), so `needless_pass_by_value`
//! is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;
use poentaro::document::Document;
use poentaro::error::Span;

pub(super) fn shift_range_octave(
    source: String,
    ranges: Vec<ByteRange>,
    delta: i32,
) -> RangeEditResponse {
    edit_ranges(&source, ranges, |lowered, ranges| {
        poentaro::source_edit::shift_range_octave(lowered, ranges, delta as i8)
    })
}

pub(super) fn toggle_range_slur(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    edit_ranges(&source, ranges, poentaro::source_edit::toggle_range_slur)
}

pub(super) fn toggle_range_tie(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
    edit_ranges(&source, ranges, poentaro::source_edit::toggle_range_tie)
}

/// Runs a range edit on the lowered form, with ranges and result in the text's
/// positions.
fn edit_ranges(
    source: &str,
    ranges: Vec<ByteRange>,
    edit: impl FnOnce(
        &str,
        &[poentaro::source_edit::ByteRange],
    ) -> poentaro::source_edit::RangeEditResult,
) -> RangeEditResponse {
    let document = Document::open(source, &[]);
    let lowered_ranges = to_lowered_ranges(&document, ranges);
    let result = edit(document.lowered(), &lowered_ranges);
    let edited = document.with_lowered(&result.source, &[]);
    RangeEditResponse {
        source: edited.text().to_string(),
        ranges: result
            .ranges
            .into_iter()
            .map(|range| {
                let span = edited.span_to_text(Span::new(
                    range.start_byte as usize,
                    range.end_byte as usize,
                ));
                ByteRange {
                    start_byte: span.start as _,
                    end_byte: span.end as _,
                }
            })
            .collect(),
    }
}

pub(super) fn describe_selection(
    source: String,
    ranges: Vec<ByteRange>,
) -> Option<PitchDescription> {
    use poentaro::pitch_description as description;
    let document = Document::open(&source, &[]);
    let ranges = to_lowered_ranges(&document, ranges);
    description::describe_selection(document.lowered(), &ranges).map(|described| match described {
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

fn to_lowered_ranges(
    document: &Document,
    ranges: Vec<ByteRange>,
) -> Vec<poentaro::source_edit::ByteRange> {
    ranges
        .into_iter()
        .map(|range| {
            let span = document.span_to_lowered(Span::new(
                range.start_byte as usize,
                range.end_byte as usize,
            ));
            poentaro::source_edit::ByteRange {
                start_byte: span.start as _,
                end_byte: span.end as _,
            }
        })
        .collect()
}
