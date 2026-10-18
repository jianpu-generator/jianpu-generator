//! Where the syllables of a lyric line under a chord part sit in the measure;
//! see "Lyrics under a chord part" in `syntax.md`.
//!
//! The Nth bracketed entry sits at the column of the Nth chord attack (so the
//! chord symbol is drawn above it); unbracketed syllables spread over the span
//! between neighbouring brackets. `compile_lyrics_slice`, the lyric source
//! spans and the grouper's tally all read the attacks from
//! `lyric_slots::chord_attacks`, so they cannot disagree.
//!
//! Spreading needs enough distinct integer columns between two attacks; the
//! grouper guarantees that by raising the measure's resolution multiplier
//! (see `unbracketed_resolution_multiplier`).

use crate::ast::grouped::NoteEvent;
use crate::ast::parsed::Syllable;
use crate::error::Span;
use crate::lyric_slots::{chord_attacks, ChordAttack};
use crate::parser::score::timed_parser::{parse_duration_suffixes, NoteHead};

/// What a placed syllable is attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyllableAnchor {
    /// A bracketed entry, sung on the chord at this event index.
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
    /// Indices into `syllables` of the bracketed items, in order.
    bracket_indices: Vec<usize>,
    measure_end: u32,
    multiplier: u32,
}

impl MeasureLine<'_> {
    /// A mismatched tally is already an error; extra brackets just reuse the
    /// last attack rather than panic.
    fn attack_of(&self, bracket_ordinal: usize) -> Option<&ChordAttack> {
        self.attacks
            .get(bracket_ordinal)
            .or_else(|| self.attacks.last())
    }

    fn bracket_column(&self, bracket_ordinal: usize) -> u32 {
        self.attack_of(bracket_ordinal)
            .map_or(0, |attack| attack.column)
    }

    /// Unbracketed syllables before the first bracket, spread from the bar
    /// start up to the first bracket's attack.
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
        let pushed = column
            + self.syllables.get(bracket_index).map_or(0, |syllable| {
                rest_duration(&syllable.text) * self.multiplier
            });
        // A rest that would swallow the whole gap pushes nothing.
        let run_start = if pushed < run_end_column {
            pushed
        } else {
            column
        };
        let run_len = next_bracket.unwrap_or(self.syllables.len()) - bracket_index - 1;
        std::iter::once(Located {
            syllable_index: bracket_index,
            column,
            chord_event: self.attack_of(ordinal).map(|attack| attack.event_index),
        })
        .chain(
            spread(run_start, run_end_column, run_len, false)
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
/// `events` are the chord part's measure events; `multiplier` is the measure's
/// resolution multiplier (needed to scale a rest bracket's named duration).
pub(crate) fn place_chord_syllables(
    events: &[NoteEvent],
    syllables: &[Syllable],
    multiplier: u32,
) -> Vec<PlacedSyllable> {
    let line = MeasureLine {
        syllables,
        attacks: chord_attacks(events),
        bracket_indices: syllables
            .iter()
            .enumerate()
            .filter(|(_, syllable)| syllable.bracketed)
            .map(|(index, _)| index)
            .collect(),
        measure_end: events.iter().map(NoteEvent::duration).sum(),
        multiplier,
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

/// What a rest bracket (`0`, `0_`, `0=`, `0-`, `0.` ...) names, read by the
/// notes parser's own duration rules. `duration` is in unscaled columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RestBracket {
    pub(crate) duration: u32,
    pub(crate) dotted: bool,
    pub(crate) double_dotted: bool,
}

/// Parses a bracketed item as a rest (`0` followed by the notes line's
/// duration suffixes); `None` for an ordinary syllable.
pub(crate) fn parse_rest_bracket(text: &str) -> Option<RestBracket> {
    let chars: Vec<char> = text.chars().collect();
    if chars.first() != Some(&'0') {
        return None;
    }
    parse_duration_suffixes::<NoteHead>(&chars, 0, 1, true, &Span::new(0, text.len()))
        .ok()
        .map(|parsed| RestBracket {
            duration: parsed.duration,
            dotted: parsed.dotted,
            double_dotted: parsed.double_dotted,
        })
}

/// The duration a rest bracket names, in unscaled columns; `0` for a bracket
/// that isn't a rest.
fn rest_duration(text: &str) -> u32 {
    parse_rest_bracket(text).map_or(0, |rest| rest.duration)
}

#[cfg(test)]
#[path = "chord_lyric_placement_tests.rs"]
mod tests;
