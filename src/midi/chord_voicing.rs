//! Voice leading for chord playback: every chord sounds a bass note (its root, or the
//! slash-chord bass) in a low register, plus its chord tones stacked in close position
//! above it. Which inversion of those upper voices sounds is chosen to move as little
//! as possible from the part's previous chord, e.g. `1 → 5` in C plays `C E G → B D G`
//! (keeping the common tone G) rather than `C E G → G B D`.

use itertools::Itertools;

/// The lowest upper voice is always placed in the octave starting here (G3..=F#4), so
/// voice leading can never drift the chord up or down indefinitely.
const UPPER_VOICES_LOWEST_NOTE_MIN: i32 = 55;
/// The bass note is always placed in the octave starting here (G2..=F#3), just below
/// the upper voices' range.
const BASS_NOTE_MIN: i32 = 43;
/// Among equally smooth voicings, prefer the one whose lowest upper voice is nearest
/// middle C.
const HOME_NOTE: i32 = 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChordVoicing {
    pub(crate) bass: u8,
    /// Ascending.
    pub(crate) upper_voices: Vec<u8>,
}

impl ChordVoicing {
    pub(crate) fn midi_notes(&self) -> Vec<u8> {
        std::iter::once(self.bass)
            .chain(self.upper_voices.iter().copied())
            .collect()
    }
}

pub(crate) struct ChordPitchClasses {
    /// Root first, then each chord tone in stacking order (third, fifth, seventh).
    pub(crate) chord_tones: Vec<i32>,
    pub(crate) bass: i32,
}

/// Voices `chord` as close to `previous` as possible, or in root position when there's
/// no previous chord to lead from.
pub(crate) fn voice_chord(
    chord: &ChordPitchClasses,
    previous: Option<&ChordVoicing>,
) -> ChordVoicing {
    let bass = place_at_or_above(chord.bass, BASS_NOTE_MIN) as u8;
    let upper_voices = match previous {
        None => close_position(&chord.chord_tones),
        Some(previous) => (0..chord.chord_tones.len())
            .map(|inversion| {
                let rotated = chord
                    .chord_tones
                    .iter()
                    .cycle()
                    .skip(inversion)
                    .take(chord.chord_tones.len())
                    .copied()
                    .collect_vec();
                close_position(&rotated)
            })
            .min_by_key(|candidate| {
                (
                    movement(&previous.upper_voices, candidate),
                    candidate
                        .first()
                        .map_or(0, |&lowest| (i32::from(lowest) - HOME_NOTE).abs()),
                )
            })
            .unwrap_or_default(),
    };
    ChordVoicing { bass, upper_voices }
}

/// Stacks `pitch_classes` upward in the given order, each tone the nearest one above
/// the previous, starting within the upper voices' range.
fn close_position(pitch_classes: &[i32]) -> Vec<u8> {
    pitch_classes
        .iter()
        .scan(UPPER_VOICES_LOWEST_NOTE_MIN, |minimum, &pitch_class| {
            let note = place_at_or_above(pitch_class, *minimum);
            *minimum = note + 1;
            Some(note as u8)
        })
        .collect()
}

fn place_at_or_above(pitch_class: i32, minimum: i32) -> i32 {
    minimum + (pitch_class - minimum).rem_euclid(12)
}

/// Total semitones travelled between two voicings: each note's distance to the nearest
/// note of the other voicing, counted in both directions so chords with different
/// numbers of voices (e.g. a triad into a seventh chord) still compare fairly.
fn movement(from: &[u8], to: &[u8]) -> u32 {
    nearest_distances(from, to) + nearest_distances(to, from)
}

fn nearest_distances(notes: &[u8], others: &[u8]) -> u32 {
    notes
        .iter()
        .filter_map(|&note| others.iter().map(|&other| note.abs_diff(other)).min())
        .map(u32::from)
        .sum()
}
