use crate::ast::parsed::{ParsedTimedTrack, ParsedTrack, PartDecl, PartKind};

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
