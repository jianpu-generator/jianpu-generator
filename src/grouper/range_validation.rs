use std::collections::HashMap;

use itertools::Itertools;

use crate::ast::grouped::{GroupedNote, NoteEvent, Score};
use crate::ast::parsed::{KeyChange, PartDecl, PitchRange, RangeInstrument};
use crate::error::{Diagnostic, Span, Warning};
use crate::pitch_description::spelling::SpelledPitch;
use crate::sounding_pitch::resolve_midi_note_with_accidental;

const SHARP_NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

struct OutOfRangeNote {
    measure_index: usize,
    span: Span,
    part_abbreviation: String,
    warning: Warning,
}

/// Warns on each note whose sounding pitch (key plus the part's octave
/// offset) falls outside its part's [`RangeInstrument`], attaching the
/// warning to the note's measure. A note shared by several measures or parts
/// (ditto, follow) is warned on once per part.
pub(super) fn validate_ranges(score: &mut Score, declarations: &[PartDecl]) {
    let instruments: HashMap<&str, RangeInstrument> = declarations
        .iter()
        .filter_map(|declaration| {
            declaration
                .range_instrument
                .map(|instrument| (declaration.abbreviation.as_str(), instrument))
        })
        .collect();
    if instruments.is_empty() {
        return;
    }

    let keys = score
        .measures
        .iter()
        .scan(KeyChange::default(), |key, measure| {
            if let Some(changed) = &measure.key {
                *key = changed.clone();
            }
            Some(key.clone())
        })
        .collect::<Vec<_>>();

    let out_of_range_notes = score
        .measures
        .iter()
        .zip(&keys)
        .enumerate()
        .flat_map(|(measure_index, (measure, key))| {
            let instruments = &instruments;
            measure.parts.iter().flat_map(move |part_row| {
                let slice = part_row.slice();
                let abbreviation = slice.name.as_deref().unwrap_or_default();
                let instrument = instruments.get(abbreviation).copied();
                slice.notes.events.iter().filter_map(move |event| {
                    let NoteEvent::Note(note) = event else {
                        return None;
                    };
                    let problem =
                        out_of_range_problem(note, slice.octave_offset, instrument?, key)?;
                    Some(OutOfRangeNote {
                        measure_index,
                        span: note.event_span,
                        part_abbreviation: abbreviation.to_string(),
                        warning: Warning::note_out_of_range(
                            note.event_span,
                            format!("{abbreviation}: {problem}"),
                        ),
                    })
                })
            })
        })
        .unique_by(|note| (note.span, note.part_abbreviation.clone()))
        .collect::<Vec<_>>();

    for note in out_of_range_notes {
        if let Some(measure) = score.measures.get_mut(note.measure_index) {
            measure.diagnostics.push(Diagnostic::Warning(note.warning));
        }
    }
}

/// Why `note` (shifted by `octave_offset` octaves, in `key`) can't be played
/// on `instrument`, or `None` when it is in range.
fn out_of_range_problem(
    note: &GroupedNote,
    octave_offset: i8,
    instrument: RangeInstrument,
    key: &KeyChange,
) -> Option<String> {
    let octave = note.octave.saturating_add(octave_offset);
    let midi = resolve_midi_note_with_accidental(&note.pitch, &note.accidental, octave, key);
    let PitchRange { lowest, highest } = instrument.range();
    let side = if midi < lowest {
        format!(
            "below the {}'s lowest note {}",
            instrument.keyword(),
            natural_note_name(lowest)
        )
    } else if midi > highest {
        format!(
            "above the {}'s highest note {}",
            instrument.keyword(),
            natural_note_name(highest)
        )
    } else {
        return None;
    };
    let sounding = SpelledPitch::key_tonic(key)
        .scale_degree(&note.pitch, &note.accidental)
        .name_with_octave(midi);
    Some(format!("note sounds as {sounding}, {side}"))
}

/// `midi`'s name with its octave (C4 = 60), spelling black keys as sharps.
fn natural_note_name(midi: u8) -> String {
    let name = SHARP_NOTE_NAMES
        .get(usize::from(midi % 12))
        .copied()
        .unwrap_or_default();
    let octave = i32::from(midi / 12) - 1;
    format!("{name}{octave}")
}

#[cfg(test)]
#[path = "range_validation_tests.rs"]
mod tests;
