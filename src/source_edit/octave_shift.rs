//! Bulk-rewrites the `'`/`,` octave markers on every note belonging to one
//! part, so a part transcribed an octave too high/low can be corrected
//! without editing each note by hand.
//!
//! Infallible / best-effort, mirroring [`super::update_part_declaration`]'s
//! "not found -> unchanged" convention: an unknown abbreviation, a
//! `follow[X]` part (which has no notes of its own), or any parse failure
//! returns `source` unchanged.

use crate::ast::parsed::{ParsedMeasureSlot, ParsedTrack, ScoreEvent};
use crate::error::Span;
use crate::parser;

/// Shifts every note in the part named `abbreviation` by `delta` octaves,
/// rewriting each note's `'`/`,` marker run in place.
pub fn shift_part_octave(source: &str, abbreviation: &str, delta: i8) -> String {
    if delta == 0 {
        return source.to_string();
    }

    let Ok(document) = parser::parse(source, "input.jianpu", &[]) else {
        return source.to_string();
    };

    let is_follow = document
        .declarations
        .iter()
        .find(|decl| decl.abbreviation == abbreviation)
        .is_none_or(|decl| decl.follow_target.is_some());
    if is_follow {
        return source.to_string();
    }

    let Some(ParsedTrack::Timed(track)) = document.tracks.iter().find(|track| {
        let ParsedTrack::Timed(track) = track;
        track.abbreviation == abbreviation
    }) else {
        return source.to_string();
    };

    let mut edits: Vec<(Span, String)> = track
        .measure_slots
        .iter()
        .filter_map(|slot| match slot {
            ParsedMeasureSlot::Real { events } => Some(events),
            ParsedMeasureSlot::EmptyNote { .. } => None,
        })
        .flatten()
        .filter_map(|spanned| {
            let ScoreEvent::Note(note) = &spanned.value else {
                return None;
            };
            let new_octave = note.octave.saturating_add(delta);
            if new_octave == note.octave {
                return None;
            }
            let text = source.get(spanned.span.start..spanned.span.end)?;
            Some((spanned.span, rewrite_octave_marker(text, new_octave)))
        })
        .collect();

    if edits.is_empty() {
        return source.to_string();
    }

    edits.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));

    let mut result = source.to_string();
    for (span, replacement) in edits {
        result.replace_range(span.start..span.end, &replacement);
    }
    result
}

/// Rewrites one note token's `'`/`,` octave marker to reflect `new_octave`,
/// preserving every other suffix character (duration/tie/dot) and their
/// relative order. The marker is re-inserted immediately after the
/// pitch+accidental head, matching the convention used throughout this
/// codebase's `.jianpu` sources (octave marker before duration/tie suffixes).
fn rewrite_octave_marker(text: &str, new_octave: i8) -> String {
    let split_at = text
        .char_indices()
        .find(|(_, c)| matches!(c, '_' | '=' | '-' | '.' | '~' | '\'' | ','))
        .map_or(text.len(), |(index, _)| index);
    let (head, remainder) = text.split_at(split_at);

    let remainder_without_octave: String = remainder
        .chars()
        .filter(|c| !matches!(c, '\'' | ','))
        .collect();

    let marker = match new_octave.cmp(&0) {
        std::cmp::Ordering::Greater => "'".repeat(new_octave as usize),
        std::cmp::Ordering::Less => ",".repeat((-new_octave) as usize),
        std::cmp::Ordering::Equal => String::new(),
    };

    format!("{head}{marker}{remainder_without_octave}")
}

#[cfg(test)]
#[path = "octave_shift_tests.rs"]
mod octave_shift_tests;
