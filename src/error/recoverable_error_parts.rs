use crate::error::{DocumentSection, RecoverableError, RecoverableErrorKind, Span};

impl RecoverableError {
    pub fn parts_malformed_line(span: Span, line: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsMalformedLine {
                line: line.to_string(),
            },
        }
    }

    pub fn parts_duplicate_abbreviation(span: Span, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsDuplicateAbbreviation {
                abbrev: abbrev.to_string(),
            },
        }
    }

    pub fn parts_empty_section(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptySection,
        }
    }

    pub fn parts_empty_display_name(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyDisplayName,
        }
    }

    pub fn parts_empty_abbreviation(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyAbbreviation,
        }
    }

    pub fn parts_empty_track_name(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyTrackName,
        }
    }

    pub fn parts_invalid_columns(span: Span, rhs: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsInvalidColumns {
                rhs: rhs.to_string(),
            },
        }
    }

    pub fn section_unknown(span: Span, name: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionUnknown {
                name: name.to_string(),
            },
        }
    }

    pub fn section_duplicate(span: Span, section: DocumentSection) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionDuplicate { section },
        }
    }

    pub fn section_missing(span: Span, section: DocumentSection) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionMissing { section },
        }
    }

    pub fn lyrics_line_empty(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsLineEmpty,
        }
    }

    pub fn lyrics_no_notes_track(span: Span, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsNoNotesTrack {
                abbrev: abbrev.to_string(),
            },
        }
    }

    pub fn parts_first_part_cannot_follow(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsFirstPartCannotFollow,
        }
    }

    pub fn parts_unknown_soundfont(span: Span, soundfont: &str, suggestions: Vec<String>) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsUnknownSoundfont {
                soundfont: soundfont.to_string(),
                suggestions,
            },
        }
    }

    pub fn parts_octave_offset_too_large(span: Span, offset: i8) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsOctaveOffsetTooLarge { offset },
        }
    }

    pub fn parts_follow_unknown_target(span: Span, target: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsFollowUnknownTarget {
                target: target.to_string(),
            },
        }
    }

    pub fn part_key_unknown(span: Span, key: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartKeyUnknown {
                key: key.to_string(),
            },
        }
    }

    pub fn score_line_missing_key_prefix(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::ScoreLineMissingKeyPrefix,
        }
    }

    pub fn positional_lyrics_ambiguous_standalone_target(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PositionalLyricsAmbiguousStandaloneTarget,
        }
    }

    pub fn tie_on_rest(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TieOnRest,
        }
    }

    pub fn repeat_no_prior_note(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::RepeatNoPriorNote,
        }
    }

    pub fn dangling_tie(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DanglingTie,
        }
    }

    pub fn tie_pitch_mismatch(span: Span, expected: String, got: String) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TiePitchMismatch { expected, got },
        }
    }
}
