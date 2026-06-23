use crate::ast::parsed::{ParsedMeasureSlot, ParsedTimedTrack, ParsedTrack, PartDecl, PartKind};
use crate::error::Spanned;
use crate::error::{IrrecoverableError, RecoverableError};

/// Convenience wrapper that calls `parse` and returns only the tracks,
/// discarding the directive-events accumulator. Used in unit tests.
pub(super) fn parse(
    content: &str,
    base_offset: usize,
    declarations: &[PartDecl],
) -> Result<Vec<ParsedTrack>, IrrecoverableError> {
    super::parse(content, base_offset, declarations).map(|(tracks, _, _)| tracks)
}

/// Convenience wrapper that calls `parse` and returns the recoverable errors,
/// discarding the tracks and directive-events accumulator.
pub(super) fn parse_recoverable_errors(
    content: &str,
    base_offset: usize,
    declarations: &[PartDecl],
) -> Result<Vec<Option<RecoverableError>>, IrrecoverableError> {
    super::parse(content, base_offset, declarations).map(|(_, _, errors)| errors)
}

pub(super) fn decl(name: &str, kind: PartKind) -> PartDecl {
    PartDecl {
        abbreviation: name.into(),
        display_name: name.into(),
        kind,
    }
}

pub(super) fn timed_track<'a>(tracks: &'a [ParsedTrack], abbrev: &str) -> &'a ParsedTimedTrack {
    tracks
        .iter()
        .find_map(|t| match t {
            ParsedTrack::Timed(n) if n.abbreviation == abbrev => Some(n),
            ParsedTrack::Timed(_) => None,
        })
        .unwrap_or_else(|| panic!("timed track '{abbrev}' not found"))
}

pub(super) fn notes_track<'a>(tracks: &'a [ParsedTrack], abbrev: &str) -> &'a ParsedTimedTrack {
    timed_track(tracks, abbrev)
}

pub(super) fn chord_track<'a>(tracks: &'a [ParsedTrack], abbrev: &str) -> &'a ParsedTimedTrack {
    timed_track(tracks, abbrev)
}

pub(super) fn all_events(
    track: &ParsedTimedTrack,
) -> Vec<&Spanned<crate::ast::parsed::ScoreEvent>> {
    track
        .measure_slots
        .iter()
        .flat_map(|slot| match slot {
            ParsedMeasureSlot::Real { events } => events.as_slice(),
            ParsedMeasureSlot::EmptyNote { .. } => &[],
        })
        .collect()
}

pub(super) fn total_lyrics_syllables(track: &ParsedTimedTrack) -> usize {
    track
        .lyrics
        .as_ref()
        .map(|lyrics| {
            lyrics
                .measure_syllables
                .iter()
                .map(|measure| measure.len())
                .sum()
        })
        .unwrap_or(0)
}
