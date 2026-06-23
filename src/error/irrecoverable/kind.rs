use std::path::PathBuf;

use super::Span;

#[derive(Debug, Clone)]
pub enum IrrecoverableErrorKind {
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
