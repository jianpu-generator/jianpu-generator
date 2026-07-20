pub(super) use super::*;

use midly::{MidiMessage, Smf, TrackEventKind};

mod lyrics;
mod measure_audio;
mod navigation_sequence_entry_range;
mod render;

pub(super) fn count_note_on_events(midi_bytes: &[u8]) -> usize {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter(|e| {
            matches!(
                e.kind,
                TrackEventKind::Midi {
                    message: MidiMessage::NoteOn { vel, .. },
                    ..
                } if vel.as_int() > 0
            )
        })
        .count()
}
