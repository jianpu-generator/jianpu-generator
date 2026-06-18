use std::path::PathBuf;

use super::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSection {
    Metadata,
    Parts,
    Score,
}

impl DocumentSection {
    pub fn header(self) -> &'static str {
        match self {
            Self::Metadata => "[metadata]",
            Self::Parts => "[parts]",
            Self::Score => "[score]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredMetadataField {
    Title,
    Author,
}

impl RequiredMetadataField {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Author => "author",
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrrecoverableErrorKind {
    UnknownSection {
        span: Span,
        name: String,
    },
    WrongSectionCount {
        span: Span,
        got: usize,
    },
    SectionsOutOfOrder {
        span: Span,
    },
    DuplicateSection {
        span: Span,
        section: DocumentSection,
    },
    MissingSection {
        span: Span,
        section: DocumentSection,
    },
    MetadataInvalidInteger {
        span: Span,
        field: String,
        value: String,
    },
    MetadataMustBePositive {
        span: Span,
        field: String,
    },
    MetadataMalformedLine {
        span: Span,
        line: String,
    },
    MetadataUnknownField {
        span: Span,
        field: String,
    },
    MetadataMissingField {
        span: Span,
        field: RequiredMetadataField,
    },
    PartsMalformedLine {
        span: Span,
        line: String,
    },
    PartsDuplicateAbbreviation {
        span: Span,
        abbrev: String,
    },
    PartsEmptySection {
        span: Span,
    },
    PartsEmptyDisplayName {
        span: Span,
    },
    PartsEmptyAbbreviation {
        span: Span,
    },
    PartsEmptyTrackName {
        span: Span,
    },
    PartsInvalidColumns {
        span: Span,
        rhs: String,
    },
    PartsNoNotesTrack {
        span: Span,
    },
    MeasureNoDataLines {
        span: Span,
    },
    MeasureMissingRoleLine {
        span: Span,
        role: String,
        abbrev: String,
    },
    DittoNoPrecedent {
        span: Span,
        role: String,
    },
    DirectiveUnclosedParen {
        span: Span,
    },
    DirectiveUnclosedQuote {
        span: Span,
    },
    DirectiveInvalidBpm {
        span: Span,
        value: String,
    },
    DirectiveLabelNotQuoted {
        span: Span,
        value: String,
    },
    DirectiveLabelEmpty {
        span: Span,
    },
    DirectiveUnknown {
        span: Span,
        token: String,
    },
    DirectiveKeyMissingNoteName {
        span: Span,
    },
    DirectiveKeyInvalidNoteName {
        span: Span,
        name: char,
    },
    DirectiveKeyInvalidOctave {
        span: Span,
        value: String,
    },
    DirectiveTimeInvalid {
        span: Span,
        value: String,
    },
    DirectiveTimeInvalidNumerator {
        span: Span,
        num: String,
    },
    DirectiveTimeInvalidDenominator {
        span: Span,
        den: String,
    },
    DirectiveTimeZeroDenominator {
        span: Span,
    },
    LexUnexpectedChar {
        span: Span,
        ch: char,
    },
    LexBpmMissingNumber {
        span: Span,
    },
    LexBpmInvalid {
        span: Span,
        value: String,
    },
    LexTimeInvalidNumerator {
        span: Span,
        num: String,
    },
    LexTimeInvalidDenominator {
        span: Span,
        den: String,
    },
    LexTimeZeroDenominator {
        span: Span,
    },
    KeyChangeMissingPrefix {
        span: Span,
        text: String,
    },
    KeyChangeMissingNoteName {
        span: Span,
        text: String,
    },
    KeyChangeInvalidNoteName {
        span: Span,
        name: char,
    },
    KeyChangeInvalidOctave {
        span: Span,
        text: String,
    },
    NoteExpectedPitchDigit {
        span: Span,
        ch: char,
    },
    ChordExpectedDegreeDigit {
        span: Span,
        ch: char,
    },
    ChordInvalidToken {
        span: Span,
        token: String,
    },
    ChordUnknownSuffix {
        span: Span,
        suffix: String,
        token: String,
    },
    ChordInvalidBass {
        span: Span,
        bass: String,
    },
    ChordBassUnexpectedChar {
        span: Span,
        ch: char,
        bass: String,
    },
    ChordBassTrailingChars {
        span: Span,
        bass: String,
    },
    DashAfterRest {
        span: Span,
    },
    DurationUnexpectedChar {
        span: Span,
        ch: char,
    },
    DurationMixedOctaveMarkers {
        span: Span,
    },
    DurationCannotDotQuarterBeat {
        span: Span,
    },
    GroupTooFewNotes {
        span: Span,
    },
    GroupUnexpectedCloseParen {
        span: Span,
    },
    UnclosedGroupAtEnd {
        span: Span,
        part: String,
    },
    IncompleteMeasure {
        span: Span,
        expected: u32,
        got: u32,
    },
    LyricsLineEmpty {
        span: Span,
    },
    UnderscoreOnlyOnLyrics {
        span: Span,
    },
    LyricsNoNotesTrack {
        span: Span,
        abbrev: String,
    },
    MeasureOverflow {
        span: Span,
        part: Option<String>,
        event_label: String,
        duration: u32,
        capacity: u32,
        used: u32,
    },
    ExtensionNoPrecedingEvent {
        span: Span,
        part: Option<String>,
        chord_track: bool,
    },
    TieNoPrecedingNote {
        span: Span,
        part: Option<String>,
    },
    PartMeasureCountMismatch {
        span: Span,
        part: String,
        got: usize,
        expected: usize,
    },
    MeasureIndexOutOfRange {
        span: Span,
        index: usize,
        total: usize,
    },
    InvalidMeasureRange {
        span: Span,
        start: usize,
        end: usize,
        total: usize,
    },
    MidiWriteFailed {
        span: Span,
    },
    WavInvalidMidiBytes {
        span: Span,
    },
    WavSynthInitFailed {
        span: Span,
    },
    WavSoundfontLoadFailed {
        span: Span,
    },
    WavWriterCreateFailed {
        span: Span,
        source: String,
    },
    WavWriteSampleFailed {
        span: Span,
        source: String,
    },
    WavFinalizeFailed {
        span: Span,
        source: String,
    },
    PdfSvgParseFailed {
        span: Span,
        detail: String,
    },
    PdfSvgConversionFailed {
        span: Span,
        detail: String,
    },
    ZipStartFileFailed {
        span: Span,
        source: String,
    },
    ZipWriteFailed {
        span: Span,
        source: String,
    },
    ZipFinishFailed {
        span: Span,
        source: String,
    },
    IoReadFailed {
        span: Span,
        path: PathBuf,
        source: String,
    },
    IoWriteFailed {
        span: Span,
        path: PathBuf,
        source: String,
    },
    InternalInvariant {
        span: Span,
        detail: String,
    },
}

impl IrrecoverableErrorKind {
    #[allow(clippy::too_many_lines)]
    pub fn span(&self) -> &Span {
        match self {
            Self::UnknownSection { span, .. }
            | Self::WrongSectionCount { span, .. }
            | Self::SectionsOutOfOrder { span }
            | Self::DuplicateSection { span, .. }
            | Self::MissingSection { span, .. }
            | Self::MetadataInvalidInteger { span, .. }
            | Self::MetadataMustBePositive { span, .. }
            | Self::MetadataMalformedLine { span, .. }
            | Self::MetadataUnknownField { span, .. }
            | Self::MetadataMissingField { span, .. }
            | Self::PartsMalformedLine { span, .. }
            | Self::PartsDuplicateAbbreviation { span, .. }
            | Self::PartsEmptySection { span }
            | Self::PartsEmptyDisplayName { span }
            | Self::PartsEmptyAbbreviation { span }
            | Self::PartsEmptyTrackName { span }
            | Self::PartsInvalidColumns { span, .. }
            | Self::PartsNoNotesTrack { span }
            | Self::MeasureNoDataLines { span }
            | Self::MeasureMissingRoleLine { span, .. }
            | Self::DittoNoPrecedent { span, .. }
            | Self::DirectiveUnclosedParen { span }
            | Self::DirectiveUnclosedQuote { span }
            | Self::DirectiveInvalidBpm { span, .. }
            | Self::DirectiveLabelNotQuoted { span, .. }
            | Self::DirectiveLabelEmpty { span }
            | Self::DirectiveUnknown { span, .. }
            | Self::DirectiveKeyMissingNoteName { span }
            | Self::DirectiveKeyInvalidNoteName { span, .. }
            | Self::DirectiveKeyInvalidOctave { span, .. }
            | Self::DirectiveTimeInvalid { span, .. }
            | Self::DirectiveTimeInvalidNumerator { span, .. }
            | Self::DirectiveTimeInvalidDenominator { span, .. }
            | Self::DirectiveTimeZeroDenominator { span }
            | Self::LexUnexpectedChar { span, .. }
            | Self::LexBpmMissingNumber { span }
            | Self::LexBpmInvalid { span, .. }
            | Self::LexTimeInvalidNumerator { span, .. }
            | Self::LexTimeInvalidDenominator { span, .. }
            | Self::LexTimeZeroDenominator { span }
            | Self::KeyChangeMissingPrefix { span, .. }
            | Self::KeyChangeMissingNoteName { span, .. }
            | Self::KeyChangeInvalidNoteName { span, .. }
            | Self::KeyChangeInvalidOctave { span, .. }
            | Self::NoteExpectedPitchDigit { span, .. }
            | Self::ChordExpectedDegreeDigit { span, .. }
            | Self::ChordInvalidToken { span, .. }
            | Self::ChordUnknownSuffix { span, .. }
            | Self::ChordInvalidBass { span, .. }
            | Self::ChordBassUnexpectedChar { span, .. }
            | Self::ChordBassTrailingChars { span, .. }
            | Self::DashAfterRest { span }
            | Self::DurationUnexpectedChar { span, .. }
            | Self::DurationMixedOctaveMarkers { span }
            | Self::DurationCannotDotQuarterBeat { span }
            | Self::GroupTooFewNotes { span }
            | Self::GroupUnexpectedCloseParen { span }
            | Self::UnclosedGroupAtEnd { span, .. }
            | Self::IncompleteMeasure { span, .. }
            | Self::LyricsLineEmpty { span }
            | Self::UnderscoreOnlyOnLyrics { span }
            | Self::LyricsNoNotesTrack { span, .. }
            | Self::MeasureOverflow { span, .. }
            | Self::ExtensionNoPrecedingEvent { span, .. }
            | Self::TieNoPrecedingNote { span, .. }
            | Self::PartMeasureCountMismatch { span, .. }
            | Self::MeasureIndexOutOfRange { span, .. }
            | Self::InvalidMeasureRange { span, .. }
            | Self::MidiWriteFailed { span }
            | Self::WavInvalidMidiBytes { span }
            | Self::WavSynthInitFailed { span }
            | Self::WavSoundfontLoadFailed { span }
            | Self::WavWriterCreateFailed { span, .. }
            | Self::WavWriteSampleFailed { span, .. }
            | Self::WavFinalizeFailed { span, .. }
            | Self::PdfSvgParseFailed { span, .. }
            | Self::PdfSvgConversionFailed { span, .. }
            | Self::ZipStartFileFailed { span, .. }
            | Self::ZipWriteFailed { span, .. }
            | Self::ZipFinishFailed { span, .. }
            | Self::IoReadFailed { span, .. }
            | Self::IoWriteFailed { span, .. }
            | Self::InternalInvariant { span, .. } => span,
        }
    }

    pub fn internal_invariant(span: Span, detail: impl Into<String>) -> Self {
        Self::InternalInvariant {
            span,
            detail: detail.into(),
        }
    }
}
