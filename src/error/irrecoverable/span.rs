use super::kind::IrrecoverableErrorKind;
use crate::error::Span;

pub(super) fn span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    parse_span(kind).or_else(|| export_span(kind))
}

fn parse_span(kind: &IrrecoverableErrorKind) -> Option<&Span> {
    match kind {
        IrrecoverableErrorKind::DittoNoPrecedent { span, .. }
        | IrrecoverableErrorKind::LexUnexpectedChar { span, .. }
        | IrrecoverableErrorKind::NoteExpectedPitchDigit { span, .. }
        | IrrecoverableErrorKind::ChordExpectedDegreeDigit { span, .. }
        | IrrecoverableErrorKind::ChordInvalidToken { span, .. }
        | IrrecoverableErrorKind::ChordUnknownSuffix { span, .. }
        | IrrecoverableErrorKind::ChordInvalidBass { span, .. }
        | IrrecoverableErrorKind::ChordBassUnexpectedChar { span, .. }
        | IrrecoverableErrorKind::ChordBassTrailingChars { span, .. }
        | IrrecoverableErrorKind::DashAfterRest { span }
        | IrrecoverableErrorKind::DurationMixedOctaveMarkers { span }
        | IrrecoverableErrorKind::DurationCannotDotQuarterBeat { span }
        | IrrecoverableErrorKind::GroupUnexpectedCloseParen { span }
        | IrrecoverableErrorKind::UnclosedGroupAtEnd { span, .. } => Some(span),
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
