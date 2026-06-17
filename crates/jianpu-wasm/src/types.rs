use jianpu_generator::{
    error::{IrrecoverableError, RecoverableError},
    error_reporter,
};
use serde::Serialize;
use tsify::Tsify;

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct SpanOut {
    /// UTF-8 byte offset (inclusive).
    pub start: usize,
    /// UTF-8 byte offset (exclusive).
    pub end: usize,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[tsify(into_wasm_abi)]
#[allow(dead_code)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct DiagnosticOut {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: SpanOut,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<String>,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct PartOut {
    pub abbreviation: String,
    pub display_name: String,
    pub has_lyrics: bool,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum RenderResponse {
    Ok {
        svgs: Vec<String>,
        diagnostics: Vec<DiagnosticOut>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListPartsResponse {
    Ok { parts: Vec<PartOut> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum MeasureAtOffsetResponse {
    Ok { measure_index: usize },
    NotInMeasure,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListMeasureSpansResponse {
    Ok { spans: Vec<SpanOut> },
    Err,
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GenerateWavResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        wav: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "pdf")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GeneratePdfResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        pdf: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "pdf")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GenerateSplitPdfsResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        zip: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

pub(crate) fn diagnostic_from_error(source: &str, e: IrrecoverableError) -> DiagnosticOut {
    let report = error_reporter::render_with_source(source, &e);
    DiagnosticOut {
        severity: DiagnosticSeverity::Error,
        message: e.message,
        span: SpanOut {
            start: e.span.start,
            end: e.span.end,
        },
        report: Some(report),
    }
}

pub(crate) fn diagnostic_from_recoverable_error(
    source: &str,
    e: RecoverableError,
) -> DiagnosticOut {
    let report = error_reporter::render_recoverable_with_source(source, &e);
    DiagnosticOut {
        severity: DiagnosticSeverity::Error,
        message: e.message,
        span: SpanOut {
            start: e.span.start,
            end: e.span.end,
        },
        report: Some(report),
    }
}
