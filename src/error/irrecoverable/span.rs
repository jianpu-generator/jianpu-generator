use super::kind::IrrecoverableErrorKind;
use crate::error::Span;

pub(super) fn span(kind: &IrrecoverableErrorKind) -> &Span {
    document_span(kind)
        .or_else(|| metadata_span(kind))
        .or_else(|| parts_span(kind))
        .or_else(|| measure_span(kind))
        .or_else(|| directive_span(kind))
        .or_else(|| lex_span(kind))
        .or_else(|| note_span(kind))
        .or_else(|| export_span(kind))
        .unwrap_or_else(|| unreachable!("all IrrecoverableErrorKind variants carry a span field"))
}

fn document_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::UnknownSection { span, .. }
        | IrrecoverableErrorKind::WrongSectionCount { span, .. }
        | IrrecoverableErrorKind::SectionsOutOfOrder { span }
        | IrrecoverableErrorKind::DuplicateSection { span, .. }
        | IrrecoverableErrorKind::MissingSection { span, .. } => Some(span),
        _ => None,
    }
}

fn metadata_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::MetadataInvalidInteger { span, .. }
        | IrrecoverableErrorKind::MetadataMustBePositive { span, .. }
        | IrrecoverableErrorKind::MetadataMalformedLine { span, .. }
        | IrrecoverableErrorKind::MetadataUnknownField { span, .. }
        | IrrecoverableErrorKind::MetadataMissingField { span, .. } => Some(span),
        _ => None,
    }
}

fn parts_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::PartsMalformedLine { span, .. }
        | IrrecoverableErrorKind::PartsDuplicateAbbreviation { span, .. }
        | IrrecoverableErrorKind::PartsEmptySection { span }
        | IrrecoverableErrorKind::PartsEmptyDisplayName { span }
        | IrrecoverableErrorKind::PartsEmptyAbbreviation { span }
        | IrrecoverableErrorKind::PartsEmptyTrackName { span }
        | IrrecoverableErrorKind::PartsInvalidColumns { span, .. }
        | IrrecoverableErrorKind::PartsNoNotesTrack { span } => Some(span),
        _ => None,
    }
}

fn measure_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::MeasureNoDataLines { span }
        | IrrecoverableErrorKind::MeasureMissingRoleLine { span, .. }
        | IrrecoverableErrorKind::DittoNoPrecedent { span, .. }
        | IrrecoverableErrorKind::IncompleteMeasure { span, .. }
        | IrrecoverableErrorKind::MeasureOverflow { span, .. }
        | IrrecoverableErrorKind::PartMeasureCountMismatch { span, .. }
        | IrrecoverableErrorKind::MeasureIndexOutOfRange { span, .. }
        | IrrecoverableErrorKind::InvalidMeasureRange { span, .. } => Some(span),
        _ => None,
    }
}

fn directive_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::DirectiveUnclosedParen { span }
        | IrrecoverableErrorKind::DirectiveUnclosedQuote { span }
        | IrrecoverableErrorKind::DirectiveInvalidBpm { span, .. }
        | IrrecoverableErrorKind::DirectiveLabelNotQuoted { span, .. }
        | IrrecoverableErrorKind::DirectiveLabelEmpty { span }
        | IrrecoverableErrorKind::DirectiveUnknown { span, .. }
        | IrrecoverableErrorKind::DirectiveKeyMissingNoteName { span }
        | IrrecoverableErrorKind::DirectiveKeyInvalidNoteName { span, .. }
        | IrrecoverableErrorKind::DirectiveKeyInvalidOctave { span, .. }
        | IrrecoverableErrorKind::DirectiveTimeInvalid { span, .. }
        | IrrecoverableErrorKind::DirectiveTimeInvalidNumerator { span, .. }
        | IrrecoverableErrorKind::DirectiveTimeInvalidDenominator { span, .. }
        | IrrecoverableErrorKind::DirectiveTimeZeroDenominator { span } => Some(span),
        _ => None,
    }
}

fn lex_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::LexUnexpectedChar { span, .. }
        | IrrecoverableErrorKind::LexBpmMissingNumber { span }
        | IrrecoverableErrorKind::LexBpmInvalid { span, .. }
        | IrrecoverableErrorKind::LexTimeInvalidNumerator { span, .. }
        | IrrecoverableErrorKind::LexTimeInvalidDenominator { span, .. }
        | IrrecoverableErrorKind::LexTimeZeroDenominator { span } => Some(span),
        _ => None,
    }
}

fn note_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::KeyChangeMissingPrefix { span, .. }
        | IrrecoverableErrorKind::KeyChangeMissingNoteName { span, .. }
        | IrrecoverableErrorKind::KeyChangeInvalidNoteName { span, .. }
        | IrrecoverableErrorKind::KeyChangeInvalidOctave { span, .. }
        | IrrecoverableErrorKind::NoteExpectedPitchDigit { span, .. }
        | IrrecoverableErrorKind::ChordExpectedDegreeDigit { span, .. }
        | IrrecoverableErrorKind::ChordInvalidToken { span, .. }
        | IrrecoverableErrorKind::ChordUnknownSuffix { span, .. }
        | IrrecoverableErrorKind::ChordInvalidBass { span, .. }
        | IrrecoverableErrorKind::ChordBassUnexpectedChar { span, .. }
        | IrrecoverableErrorKind::ChordBassTrailingChars { span, .. }
        | IrrecoverableErrorKind::DashAfterRest { span }
        | IrrecoverableErrorKind::DurationUnexpectedChar { span, .. }
        | IrrecoverableErrorKind::DurationMixedOctaveMarkers { span }
        | IrrecoverableErrorKind::DurationCannotDotQuarterBeat { span }
        | IrrecoverableErrorKind::GroupUnexpectedCloseParen { span }
        | IrrecoverableErrorKind::UnclosedGroupAtEnd { span, .. }
        | IrrecoverableErrorKind::LyricsLineEmpty { span }
        | IrrecoverableErrorKind::UnderscoreOnlyOnLyrics { span }
        | IrrecoverableErrorKind::LyricsNoNotesTrack { span, .. }
        | IrrecoverableErrorKind::ExtensionNoPrecedingEvent { span, .. }
        | IrrecoverableErrorKind::TieNoPrecedingNote { span, .. } => Some(span),
        _ => None,
    }
}

fn export_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::MidiWriteFailed { span }
        | IrrecoverableErrorKind::WavInvalidMidiBytes { span }
        | IrrecoverableErrorKind::WavSynthInitFailed { span }
        | IrrecoverableErrorKind::WavSoundfontLoadFailed { span }
        | IrrecoverableErrorKind::WavWriterCreateFailed { span, .. }
        | IrrecoverableErrorKind::WavWriteSampleFailed { span, .. }
        | IrrecoverableErrorKind::WavFinalizeFailed { span, .. }
        | IrrecoverableErrorKind::PdfSvgParseFailed { span, .. }
        | IrrecoverableErrorKind::PdfSvgConversionFailed { span, .. }
        | IrrecoverableErrorKind::ZipStartFileFailed { span, .. }
        | IrrecoverableErrorKind::ZipWriteFailed { span, .. }
        | IrrecoverableErrorKind::ZipFinishFailed { span, .. }
        | IrrecoverableErrorKind::IoReadFailed { span, .. }
        | IrrecoverableErrorKind::IoWriteFailed { span, .. }
        | IrrecoverableErrorKind::InternalInvariant { span, .. } => Some(span),
        _ => None,
    }
}
