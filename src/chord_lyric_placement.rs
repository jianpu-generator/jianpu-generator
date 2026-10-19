//! Where the syllables of a lyric line under a chord part sit in the measure;
//! see "Lyrics under a chord part" in `syntax.md`.
//!
//! The Nth chord-bound entry sits at the column of the Nth chord attack (so the
//! chord symbol is drawn above it); the other syllables spread over the span
//! between neighbouring chord-bound entries. A chord-bound entry is a bracketed
//! one, or an unbracketed first syllable when the first attack is on beat 1
//! (see `chord_bound_indices`). `compile_lyrics_slice`, the lyric source
//! spans and the grouper's tally all read the attacks from
//! `lyric_slots::chord_attacks`, so they cannot disagree.
//!
//! Spreading needs enough distinct integer columns between two attacks; the
//! grouper guarantees that by raising the measure's resolution multiplier
//! (see `unbracketed_resolution_multiplier`).

use crate::ast::grouped::NoteEvent;
use crate::ast::parsed::Syllable;
use crate::error::{RecoverableError, Span};
use crate::lyric_slots::{chord_attacks, ChordAttack};

/// What a placed syllable is attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyllableAnchor {
    /// A chord-bound entry, sung on the chord at this event index.
    Chord { event_index: usize },
    /// An unbracketed syllable; `ordinal` counts the measure's unbracketed
    /// syllables from 0.
    Floating { ordinal: usize },
}

/// One syllable of the line with the measure-relative column it is drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlacedSyllable {
    /// Index into the line's syllables.
    pub(crate) syllable_index: usize,
    pub(crate) column: u32,
    pub(crate) anchor: SyllableAnchor,
}

/// Floating syllables have no note to borrow an id from. Their ids live far
/// above any real note id (and are unique per `(measure start id, ordinal)`)
/// so a click target keyed by note id never collides with a chord's.
const FLOATING_ID_BASE: usize = 1 << 28;
const FLOATING_ID_STRIDE: usize = 64;

impl PlacedSyllable {
    /// `first_note_id`: the note id of the measure's first event (the lyric
    /// part's running counter at the start of the measure).
    pub(crate) fn note_id(&self, first_note_id: usize) -> usize {
        match self.anchor {
            SyllableAnchor::Chord { event_index } => first_note_id + event_index,
            SyllableAnchor::Floating { ordinal } => FLOATING_ID_BASE
                .saturating_add(first_note_id.saturating_mul(FLOATING_ID_STRIDE))
                .saturating_add(ordinal.min(FLOATING_ID_STRIDE - 1)),
        }
    }
}

/// Whether the line's unbracketed first syllable takes the first chord attack:
/// it does when that attack is on beat 1, since nothing could precede it.
fn first_syllable_takes_beat_one(syllables: &[Syllable], first_attack_on_beat_one: bool) -> bool {
    first_attack_on_beat_one
        && syllables
            .first()
            .is_some_and(|syllable| !syllable.bracketed)
}

/// Indices into `syllables` of the entries paired with chord attacks, in
/// order: every bracketed one, plus an unbracketed first syllable when the
/// first attack is on beat 1.
fn chord_bound_indices(syllables: &[Syllable], first_attack_on_beat_one: bool) -> Vec<usize> {
    let implicit_first = first_syllable_takes_beat_one(syllables, first_attack_on_beat_one);
    syllables
        .iter()
        .enumerate()
        .filter(|(index, syllable)| syllable.bracketed || (implicit_first && *index == 0))
        .map(|(index, _)| index)
        .collect()
}

/// Validates a chord-lyric line against its measure's chord attacks: the
/// chord-bound entries (see `chord_bound_indices`) must number at most
/// `attack_count` (the Nth takes the Nth attack; trailing attacks may go
/// without). `_` (no syllables) passes. The grouper and the part stream editor
/// both run this tally.
pub(crate) fn chord_lyric_tally_error(
    syllables: &[Syllable],
    attack_count: usize,
    first_attack_on_beat_one: bool,
    span: Span,
) -> Option<RecoverableError> {
    let first_syllable_takes_beat_one =
        first_syllable_takes_beat_one(syllables, first_attack_on_beat_one);
    let brackets = syllables
        .iter()
        .filter(|syllable| syllable.bracketed)
        .count();
    (brackets + usize::from(first_syllable_takes_beat_one) > attack_count).then(|| {
        RecoverableError::lyrics_bracket_count_mismatch(
            span,
            brackets,
            attack_count,
            first_syllable_takes_beat_one,
        )
    })
}

/// The resolution multiplier a measure's lyric line needs so every gap between
/// chord attacks has room for its unbracketed syllables: `u + 1` for `u`
/// unbracketed syllables, `1` when there are none.
pub(crate) fn unbracketed_resolution_multiplier(syllables: &[Syllable]) -> u32 {
    let unbracketed = syllables.iter().filter(|s| !s.bracketed).count();
    if unbracketed == 0 {
        1
    } else {
        u32::try_from(unbracketed + 1).unwrap_or(1)
    }
}

/// A syllable's column before anchors (and floating ordinals) are assigned;
/// `chord_event` is `Some` for a bracketed entry.
struct Located {
    syllable_index: usize,
    column: u32,
    chord_event: Option<usize>,
}

/// The measure facts every placement step reads.
struct MeasureLine<'a> {
    syllables: &'a [Syllable],
    attacks: Vec<ChordAttack>,
    /// Indices into `syllables` of the chord-bound entries, in order.
    bracket_indices: Vec<usize>,
    measure_end: u32,
}

impl MeasureLine<'_> {
    /// Extra brackets are already an error; they just reuse the last attack
    /// rather than panic.
    fn attack_of(&self, bracket_ordinal: usize) -> Option<&ChordAttack> {
        self.attacks
            .get(bracket_ordinal)
            .or_else(|| self.attacks.last())
    }

    fn bracket_column(&self, bracket_ordinal: usize) -> u32 {
        self.attack_of(bracket_ordinal)
            .map_or(0, |attack| attack.column)
    }

    /// Unbracketed syllables before the first chord-bound entry, spread from
    /// the bar start up to its attack (only when that attack is past beat 1).
    fn pickup(&self) -> Vec<Located> {
        let first_bracket = self
            .bracket_indices
            .first()
            .copied()
            .unwrap_or(self.syllables.len());
        let end = if self.bracket_indices.is_empty() {
            self.measure_end
        } else {
            self.bracket_column(0)
        };
        spread(0, end, first_bracket, true)
            .into_iter()
            .enumerate()
            .map(|(syllable_index, column)| Located {
                syllable_index,
                column,
                chord_event: None,
            })
            .collect()
    }

    /// The bracket itself, then the unbracketed syllables after it up to the
    /// next bracket (or the bar line).
    fn bracket_and_run(&self, ordinal: usize, bracket_index: usize) -> Vec<Located> {
        let column = self.bracket_column(ordinal);
        let next_bracket = self.bracket_indices.get(ordinal + 1).copied();
        let run_end_column =
            next_bracket.map_or(self.measure_end, |_| self.bracket_column(ordinal + 1));
        let run_len = next_bracket.unwrap_or(self.syllables.len()) - bracket_index - 1;
        std::iter::once(Located {
            syllable_index: bracket_index,
            column,
            chord_event: self.attack_of(ordinal).map(|attack| attack.event_index),
        })
        .chain(
            spread(column, run_end_column, run_len, false)
                .into_iter()
                .enumerate()
                .map(|(offset, column)| Located {
                    syllable_index: bracket_index + 1 + offset,
                    column,
                    chord_event: None,
                }),
        )
        .collect()
    }
}

/// Places every syllable of `syllables` (a chord-lyric line, in order).
/// `events` are the chord part's measure events.
pub(crate) fn place_chord_syllables(
    events: &[NoteEvent],
    syllables: &[Syllable],
) -> Vec<PlacedSyllable> {
    let attacks = chord_attacks(events);
    let first_attack_on_beat_one = attacks.first().is_some_and(|attack| attack.column == 0);
    let line = MeasureLine {
        syllables,
        attacks,
        bracket_indices: chord_bound_indices(syllables, first_attack_on_beat_one),
        measure_end: events.iter().map(NoteEvent::duration).sum(),
    };
    let mut located: Vec<Located> = line
        .pickup()
        .into_iter()
        .chain(
            line.bracket_indices
                .iter()
                .enumerate()
                .flat_map(|(ordinal, &bracket_index)| line.bracket_and_run(ordinal, bracket_index)),
        )
        .collect();
    located.sort_by_key(|entry| entry.syllable_index);
    let mut floating_ordinal = 0;
    located
        .into_iter()
        .map(|entry| {
            let anchor = match entry.chord_event {
                Some(event_index) => SyllableAnchor::Chord { event_index },
                None => {
                    floating_ordinal += 1;
                    SyllableAnchor::Floating {
                        ordinal: floating_ordinal - 1,
                    }
                }
            };
            PlacedSyllable {
                syllable_index: entry.syllable_index,
                column: entry.column,
                anchor,
            }
        })
        .collect()
}

/// `count` columns spread evenly over `start..end`. A lead-in run starts at
/// `start` itself (the bar start); otherwise the columns lie strictly between
/// the two ends.
fn spread(start: u32, end: u32, count: usize, lead_in: bool) -> Vec<u32> {
    let length = u64::from(end.saturating_sub(start));
    let divisor = (count + usize::from(!lead_in)) as u64;
    (0..count)
        .map(|offset| {
            let step = (offset + usize::from(!lead_in)) as u64;
            let rounded = (step * length + divisor / 2) / divisor.max(1);
            start + u32::try_from(rounded).unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
#[path = "chord_lyric_placement_tests.rs"]
mod tests;
