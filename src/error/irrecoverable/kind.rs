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
    pub fn span(&self) -> Option<&Span> {
        super::span::span(self)
    }

    pub fn internal_invariant(span: Span, detail: impl Into<String>) -> Self {
        Self::InternalInvariant {
            span,
            detail: detail.into(),
        }
    }
}
