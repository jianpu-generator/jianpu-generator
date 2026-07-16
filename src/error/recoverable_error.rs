use crate::error::{DocumentSection, RecoverableErrorKind, Span};

/// A recoverable error: render continues but the affected measure is highlighted red.
/// Displayed as a red view zone in the editor.
#[derive(Debug, Clone)]
pub struct RecoverableError {
    pub span: Span,
    pub kind: RecoverableErrorKind,
}

impl RecoverableError {
    pub fn message(&self) -> String {
        self.kind.message()
    }

    pub fn general(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::General {
                message: message.into(),
            },
        }
    }

    pub fn lex_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LexUnexpectedChar { ch },
        }
    }

    pub fn measure_no_data_lines(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureNoDataLines,
        }
    }

    pub fn measure_wrong_line_count(span: Span, got: usize, expected: usize) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureWrongLineCount { got, expected },
        }
    }

    pub fn measure_too_many_lines(span: Span, got: usize, expected: usize, parts: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureTooManyLines {
                got,
                expected,
                parts: parts.to_string(),
            },
        }
    }

    pub fn measure_missing_role_line(span: Span, role: &str, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureMissingRoleLine {
                role: role.to_string(),
                abbrev: abbrev.to_string(),
            },
        }
    }

    pub fn dotted_eighth_needs_sixteenth(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DottedEighthNeedsSixteenth,
        }
    }

    pub fn dash_after_rest(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DashAfterRest,
        }
    }

    pub fn extension_no_preceding_event(span: Span, chord_track: bool) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::ExtensionNoPrecedingEvent { chord_track },
        }
    }

    pub fn duration_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationUnexpectedChar { ch },
        }
    }

    pub fn duration_mixed_octave_markers(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationMixedOctaveMarkers,
        }
    }

    pub fn duration_cannot_dot_quarter_beat(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationCannotDotQuarterBeat,
        }
    }

    pub fn measure_directives_missing(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureDirectivesMissing,
        }
    }

    pub fn source_span_missing(span: Span, index: usize) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SourceSpanMissing { index },
        }
    }

    pub fn timed_part_measure_missing(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TimedPartMeasureMissing,
        }
    }

    pub fn part_measure_count_mismatch(
        span: Span,
        part: impl Into<String>,
        got: usize,
        expected: usize,
    ) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartMeasureCountMismatch {
                part: part.into(),
                got,
                expected,
            },
        }
    }

    pub fn metadata_malformed_line(span: Span, line: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataMalformedLine {
                line: line.to_string(),
            },
        }
    }

    pub fn metadata_unknown_field(span: Span, field: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataUnknownField {
                field: field.to_string(),
            },
        }
    }

    pub fn metadata_invalid_integer(span: Span, field: &str, value: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataInvalidInteger {
                field: field.to_string(),
                value: value.to_string(),
            },
        }
    }

    pub fn metadata_must_be_positive(span: Span, field: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataMustBePositive {
                field: field.to_string(),
            },
        }
    }

    pub fn metadata_invalid_boolean(span: Span, field: &str, value: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataInvalidBoolean {
                field: field.to_string(),
                value: value.to_string(),
            },
        }
    }

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

    pub fn parts_follow_target_after_follower(span: Span, target: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsFollowTargetAfterFollower {
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

    pub fn groups_abbreviation_collides_with_part(span: Span, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::GroupsAbbreviationCollidesWithPart {
                abbrev: abbrev.to_string(),
            },
        }
    }

    pub fn groups_unknown_member(span: Span, group: &str, member: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::GroupsUnknownMember {
                group: group.to_string(),
                member: member.to_string(),
            },
        }
    }

    pub fn groups_member_kind_mismatch(span: Span, group: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::GroupsMemberKindMismatch {
                group: group.to_string(),
            },
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
