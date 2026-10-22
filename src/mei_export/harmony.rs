//! Chord parts as MEI `<harm>` symbols with letter names.

use super::escape;
use super::pitch::{pitch_class_name, spell};
use crate::ast::grouped::{GroupedChordNote, NoteEvent, PartSlice};
use crate::ast::parsed::{Extension, KeyChange, TriadQuality};
use itertools::Itertools;

/// Where a measure's `<harm>` elements go.
pub(super) struct HarmonyPlacement<'a> {
    pub(super) key: &'a KeyChange,
    /// Time signature denominator, which `@tstamp` counts in.
    pub(super) meter_unit: u8,
    /// 1-based staff number the symbols hang over.
    pub(super) staff: usize,
}

pub(super) fn harm_xml(slice: &PartSlice, placement: &HarmonyPlacement) -> String {
    let multiplier = slice.resolution_multiplier.max(1);
    let sixteenths_per_unit = 16 / u32::from(placement.meter_unit.max(1));
    slice
        .notes
        .events
        .iter()
        .scan(0u32, |column, event| {
            let start = *column;
            *column += event.duration() / multiplier;
            Some(TimedEvent { start, event })
        })
        .filter_map(|timed| match timed.event {
            NoteEvent::Chord(chord) => Some(format!(
                "<harm staff=\"{}\" tstamp=\"{}\">{}</harm>\n",
                placement.staff,
                1.0 + timed.start as f32 / sixteenths_per_unit as f32,
                escape(&chord_name(chord, placement.key))
            )),
            NoteEvent::Note(_) | NoteEvent::Rest(_) | NoteEvent::Percussion(_) => None,
        })
        .join("")
}

struct TimedEvent<'a> {
    start: u32,
    event: &'a NoteEvent,
}

/// A chord's letter-name symbol in the key, e.g. `6m` in F is `Dm`.
pub(super) fn chord_name(chord: &GroupedChordNote, key: &KeyChange) -> String {
    let root = spell(&chord.degree, &chord.accidental, 0, key);
    let quality = match chord.triad {
        TriadQuality::Major => "",
        TriadQuality::Minor => "m",
        TriadQuality::Diminished => "dim",
        TriadQuality::Augmented => "aug",
        TriadQuality::Sus2 => "sus2",
        TriadQuality::Sus4 => "sus4",
    };
    let extension = match chord.extension {
        Some(Extension::DominantSeventh) => "7",
        Some(Extension::MajorSeventh) => "maj7",
        None => "",
    };
    let bass = chord
        .bass
        .as_ref()
        .map(|bass| {
            let pitch = spell(&bass.degree, &bass.accidental, 0, key);
            format!("/{}", pitch_class_name(pitch))
        })
        .unwrap_or_default();
    format!("{}{quality}{extension}{bass}", pitch_class_name(root))
}
