use crate::ast::parsed::{Extension, TriadQuality};

use super::spelling::Interval;

/// What a chord symbol's triad quality + extension means: which tones it
/// stacks on the root, the suffix to print after the root letter, and the
/// matching chords-db suffix (`None` when chords-db has no voicing for it).
///
/// The tones match what MIDI playback sounds (`midi::event_processing::
/// chord_midi_notes`) — in particular `o7` stacks a minor 7th on a
/// diminished triad, i.e. half-diminished `m7b5`.
pub(crate) struct ChordTemplate {
    pub(crate) intervals: Vec<Interval>,
    pub(crate) name_suffix: &'static str,
    pub(crate) chords_db_suffix: Option<&'static str>,
}

const ROOT: Interval = interval(0, 0);
const MAJOR_SECOND: Interval = interval(1, 2);
const MINOR_THIRD: Interval = interval(2, 3);
const MAJOR_THIRD: Interval = interval(2, 4);
const PERFECT_FOURTH: Interval = interval(3, 5);
const DIMINISHED_FIFTH: Interval = interval(4, 6);
const PERFECT_FIFTH: Interval = interval(4, 7);
const AUGMENTED_FIFTH: Interval = interval(4, 8);
const MINOR_SEVENTH: Interval = interval(6, 10);
const MAJOR_SEVENTH: Interval = interval(6, 11);

const fn interval(letter_steps: u8, semitones: i8) -> Interval {
    Interval {
        letter_steps,
        semitones,
    }
}

pub(crate) fn chord_template(triad: &TriadQuality, extension: Option<&Extension>) -> ChordTemplate {
    let triad_intervals = match triad {
        TriadQuality::Major => [ROOT, MAJOR_THIRD, PERFECT_FIFTH],
        TriadQuality::Minor => [ROOT, MINOR_THIRD, PERFECT_FIFTH],
        TriadQuality::Diminished => [ROOT, MINOR_THIRD, DIMINISHED_FIFTH],
        TriadQuality::Augmented => [ROOT, MAJOR_THIRD, AUGMENTED_FIFTH],
        TriadQuality::Sus2 => [ROOT, MAJOR_SECOND, PERFECT_FIFTH],
        TriadQuality::Sus4 => [ROOT, PERFECT_FOURTH, PERFECT_FIFTH],
    };
    let seventh = extension.map(|extension| match extension {
        Extension::DominantSeventh => MINOR_SEVENTH,
        Extension::MajorSeventh => MAJOR_SEVENTH,
    });
    let suffix = chord_suffix(triad, extension);
    ChordTemplate {
        intervals: triad_intervals.into_iter().chain(seventh).collect(),
        name_suffix: suffix.name,
        chords_db_suffix: suffix.chords_db,
    }
}

struct ChordSuffix {
    name: &'static str,
    chords_db: Option<&'static str>,
}

const fn suffix(name: &'static str, chords_db: Option<&'static str>) -> ChordSuffix {
    ChordSuffix { name, chords_db }
}

fn chord_suffix(triad: &TriadQuality, extension: Option<&Extension>) -> ChordSuffix {
    use Extension::{DominantSeventh, MajorSeventh};
    use TriadQuality::{Augmented, Diminished, Major, Minor, Sus2, Sus4};
    match (triad, extension) {
        (Major, None) => suffix("", Some("major")),
        (Minor, None) => suffix("m", Some("minor")),
        (Diminished, None) => suffix("dim", Some("dim")),
        (Augmented, None) => suffix("aug", Some("aug")),
        (Sus2, None) => suffix("sus2", Some("sus2")),
        (Sus4, None) => suffix("sus4", Some("sus4")),
        (Major, Some(DominantSeventh)) => suffix("7", Some("7")),
        (Major, Some(MajorSeventh)) => suffix("maj7", Some("maj7")),
        (Minor, Some(DominantSeventh)) => suffix("m7", Some("m7")),
        (Minor, Some(MajorSeventh)) => suffix("mmaj7", Some("mmaj7")),
        (Diminished, Some(DominantSeventh)) => suffix("m7b5", Some("m7b5")),
        (Diminished, Some(MajorSeventh)) => suffix("dim(maj7)", None),
        (Augmented, Some(DominantSeventh)) => suffix("aug7", Some("aug7")),
        (Augmented, Some(MajorSeventh)) => suffix("maj7#5", Some("maj7#5")),
        (Sus2, Some(DominantSeventh)) => suffix("7sus2", None),
        (Sus2, Some(MajorSeventh)) => suffix("maj7sus2", None),
        (Sus4, Some(DominantSeventh)) => suffix("7sus4", Some("7sus4")),
        (Sus4, Some(MajorSeventh)) => suffix("maj7sus4", None),
    }
}
