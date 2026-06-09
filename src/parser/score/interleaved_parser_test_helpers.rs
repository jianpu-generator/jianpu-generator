use crate::ast::parsed::{ParsedChordTrack, ParsedNotesTrack, ParsedTrack, PartDecl, PartKind};

pub(super) fn decl(name: &str, kind: PartKind) -> PartDecl {
    PartDecl {
        abbreviation: name.into(),
        display_name: name.into(),
        kind,
    }
}

pub(super) fn notes_track<'a>(tracks: &'a [ParsedTrack], abbrev: &str) -> &'a ParsedNotesTrack {
    tracks
        .iter()
        .find_map(|t| match t {
            ParsedTrack::Notes(n) if n.abbreviation == abbrev => Some(n),
            _ => None,
        })
        .unwrap_or_else(|| panic!("notes track '{abbrev}' not found"))
}

pub(super) fn chord_track<'a>(tracks: &'a [ParsedTrack], abbrev: &str) -> &'a ParsedChordTrack {
    tracks
        .iter()
        .find_map(|t| match t {
            ParsedTrack::Chord(c) if c.abbreviation == abbrev => Some(c),
            _ => None,
        })
        .unwrap_or_else(|| panic!("chord track '{abbrev}' not found"))
}
