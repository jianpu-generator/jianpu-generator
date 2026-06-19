use jianpu_generator::{
    error::{Diagnostic, IrrecoverableError, Warning},
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
        diagnostic_view_zones: Vec<DiagnosticViewZoneOut>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
        diagnostic_view_zones: Vec<DiagnosticViewZoneOut>,
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

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MeasureSpanOut {
    /// Inclusive start of note content (for cursor/selection mapping).
    pub start: usize,
    /// Exclusive end of measure content in source.
    pub end: usize,
    /// Byte offset of the first source line in this measure group, for view zones.
    pub view_zone_start: usize,
    /// Section label from `label="..."` directive, if present on this measure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_label: Option<String>,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListMeasureSpansResponse {
    Ok { spans: Vec<MeasureSpanOut> },
    Err,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct ScoreLineHintOut {
    pub line_start: usize,
    pub abbreviation: String,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListScoreLineHintsResponse {
    Ok { hints: Vec<ScoreLineHintOut> },
    Err { diagnostics: Vec<DiagnosticOut> },
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

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct DiagnosticMessageOut {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<String>,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct DiagnosticViewZoneOut {
    pub severity: DiagnosticSeverity,
    /// 1-based line number; view zone is inserted after this line.
    pub after_line_number: usize,
    pub messages: Vec<DiagnosticMessageOut>,
}

pub(crate) fn diagnostic_from_error(source: &str, e: IrrecoverableError) -> DiagnosticOut {
    let report = error_reporter::render_with_source(source, &e);
    let span = e.span();
    DiagnosticOut {
        severity: DiagnosticSeverity::Error,
        message: e.message(),
        span: SpanOut {
            start: span.start,
            end: span.end,
        },
        report: Some(report),
    }
}

pub(crate) fn diagnostic_from_warning(source: &str, e: Warning) -> DiagnosticOut {
    let report = error_reporter::render_warning_with_source(source, &e);
    DiagnosticOut {
        severity: DiagnosticSeverity::Warning,
        message: e.message,
        span: SpanOut {
            start: e.span.start,
            end: e.span.end,
        },
        report: Some(report),
    }
}

pub(crate) fn diagnostic_from_diagnostic(source: &str, d: Diagnostic) -> DiagnosticOut {
    match d {
        Diagnostic::Warning(w) => diagnostic_from_warning(source, w),
        Diagnostic::Error(e) => DiagnosticOut {
            severity: DiagnosticSeverity::Error,
            message: e.message().clone(),
            span: SpanOut {
                start: e.span.start,
                end: e.span.end,
            },
            report: None,
        },
    }
}

fn byte_offset_to_line_number(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset.min(source.len())]
        .bytes()
        .filter(|&b| b == b'\n')
        .count()
        + 1
}

struct ViewZoneAccumulator {
    severity: DiagnosticSeverity,
    messages: Vec<DiagnosticMessageOut>,
}

pub(crate) fn group_diagnostics_into_view_zones(
    source: &str,
    diagnostics: &[DiagnosticOut],
) -> Vec<DiagnosticViewZoneOut> {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<(usize, u8), ViewZoneAccumulator> = BTreeMap::new();

    for d in diagnostics {
        let line = byte_offset_to_line_number(source, d.span.end);
        let severity_order = match d.severity {
            DiagnosticSeverity::Error => 0,
            DiagnosticSeverity::Warning => 1,
        };
        let entry = groups
            .entry((line, severity_order))
            .or_insert_with(|| ViewZoneAccumulator {
                severity: d.severity.clone(),
                messages: Vec::new(),
            });
        entry.messages.push(DiagnosticMessageOut {
            message: d.message.clone(),
            report: d.report.clone(),
        });
    }

    groups
        .into_iter()
        .map(|((line, _), accumulator)| DiagnosticViewZoneOut {
            severity: accumulator.severity,
            after_line_number: line,
            messages: accumulator.messages,
        })
        .collect()
}
