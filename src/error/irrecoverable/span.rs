use super::kind::IrrecoverableErrorKind;
use crate::error::Span;

pub(super) fn span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    parse_span(kind).or_else(|| export_span(kind))
}

fn parse_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::DittoNoPrecedent { span, .. }
        | IrrecoverableErrorKind::IncompleteMeasure { span, .. }
        | IrrecoverableErrorKind::MeasureOverflow { span, .. }
        | IrrecoverableErrorKind::PartMeasureCountMismatch { span, .. }
        | IrrecoverableErrorKind::MeasureIndexOutOfRange { span, .. }
        | IrrecoverableErrorKind::InvalidMeasureRange { span, .. }
        | IrrecoverableErrorKind::DirectiveUnclosedParen { span }
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
        | IrrecoverableErrorKind::DirectiveTimeZeroDenominator { span }
        | IrrecoverableErrorKind::LexUnexpectedChar { span, .. }
        | IrrecoverableErrorKind::LexBpmMissingNumber { span }
        | IrrecoverableErrorKind::LexBpmInvalid { span, .. }
        | IrrecoverableErrorKind::LexTimeInvalidNumerator { span, .. }
        | IrrecoverableErrorKind::LexTimeInvalidDenominator { span, .. }
        | IrrecoverableErrorKind::LexTimeZeroDenominator { span }
        | IrrecoverableErrorKind::KeyChangeMissingPrefix { span, .. }
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
