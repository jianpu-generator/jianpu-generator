use jianpu_generator::{
    compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor},
    error::{Diagnostic, IrrecoverableError, Warning},
    error_reporter,
    renderer::new_types::{SvgDocument, SvgElement, SvgKind, Tag},
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

#[derive(Debug, Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct SvgDocumentOut {
    pub width_pt: f32,
    pub height_pt: f32,
    pub elements: Vec<SvgElementOut>,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct SvgElementOut {
    pub x: f32,
    pub y: f32,
    pub variant: String,
    pub kind: SvgKindOut,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum SvgKindOut {
    Text {
        content: String,
        font_size: f32,
        anchor: TextAnchorOut,
        baseline: DominantBaselineOut,
        font: FontFamilyOut,
        weight: FontWeightOut,
        italic: bool,
    },
    Line {
        x2: f32,
        y2: f32,
        stroke_width: f32,
    },
    Circle {
        r: f32,
    },
    Path {
        control_x: f32,
        control_y: f32,
        end_x: f32,
        end_y: f32,
        stroke_width: f32,
    },
    Rect {
        width: f32,
        height: f32,
    },
    ErrorRect {
        width: f32,
        height: f32,
    },
    TransparentRect {
        width: f32,
        height: f32,
    },
    Group {
        children: Vec<SvgElementOut>,
        tag: Option<TagOut>,
    },
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum TagOut {
    Measure { index: usize },
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum TextAnchorOut {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum DominantBaselineOut {
    Middle,
    Hanging,
    Ideographic,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum FontFamilyOut {
    Monospace,
    SansSerif,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum FontWeightOut {
    Normal,
    Bold,
}

pub(crate) fn svg_document_to_out(doc: &SvgDocument) -> SvgDocumentOut {
    SvgDocumentOut {
        width_pt: doc.width_pt,
        height_pt: doc.height_pt,
        elements: doc.elements.iter().map(svg_element_to_out).collect(),
    }
}

fn svg_element_to_out(el: &SvgElement) -> SvgElementOut {
    SvgElementOut {
        x: el.x,
        y: el.y,
        variant: el.variant.to_string(),
        kind: svg_kind_to_out(&el.kind),
    }
}

fn svg_kind_to_out(kind: &SvgKind) -> SvgKindOut {
    match kind {
        SvgKind::Text {
            content,
            font_size,
            anchor,
            baseline,
            font,
            weight,
            italic,
        } => SvgKindOut::Text {
            content: content.clone(),
            font_size: *font_size,
            anchor: match anchor {
                TextAnchor::Start => TextAnchorOut::Start,
                TextAnchor::Middle => TextAnchorOut::Middle,
                TextAnchor::End => TextAnchorOut::End,
            },
            baseline: match baseline {
                DominantBaseline::Middle => DominantBaselineOut::Middle,
                DominantBaseline::Hanging => DominantBaselineOut::Hanging,
                DominantBaseline::Ideographic => DominantBaselineOut::Ideographic,
            },
            font: match font {
                FontFamily::Monospace => FontFamilyOut::Monospace,
                FontFamily::SansSerif => FontFamilyOut::SansSerif,
            },
            weight: match weight {
                FontWeight::Normal => FontWeightOut::Normal,
                FontWeight::Bold => FontWeightOut::Bold,
            },
            italic: *italic,
        },
        SvgKind::Line {
            x2,
            y2,
            stroke_width,
        } => SvgKindOut::Line {
            x2: *x2,
            y2: *y2,
            stroke_width: *stroke_width,
        },
        SvgKind::Circle { r } => SvgKindOut::Circle { r: *r },
        SvgKind::Path {
            control_x,
            control_y,
            end_x,
            end_y,
            stroke_width,
        } => SvgKindOut::Path {
            control_x: *control_x,
            control_y: *control_y,
            end_x: *end_x,
            end_y: *end_y,
            stroke_width: *stroke_width,
        },
        SvgKind::Rect { width, height } => SvgKindOut::Rect {
            width: *width,
            height: *height,
        },
        SvgKind::ErrorRect { width, height } => SvgKindOut::ErrorRect {
            width: *width,
            height: *height,
        },
        SvgKind::TransparentRect { width, height } => SvgKindOut::TransparentRect {
            width: *width,
            height: *height,
        },
        SvgKind::Group { children, tag } => SvgKindOut::Group {
            children: children.iter().map(svg_element_to_out).collect(),
            tag: tag.as_ref().map(|t| match t {
                Tag::Measure { index } => TagOut::Measure { index: *index },
            }),
        },
    }
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum RenderResponse {
    Ok {
        documents: Vec<SvgDocumentOut>,
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
    /// 1-indexed first line of this measure (inclusive).
    pub start_line: usize,
    /// 1-indexed last line of this measure (inclusive).
    pub end_line: usize,
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

pub(crate) fn diagnostic_from_error(source: &str, e: &IrrecoverableError) -> DiagnosticOut {
    let report = error_reporter::render_with_source(source, e);
    let span = e
        .span()
        .map(|s| SpanOut {
            start: s.start,
            end: s.end,
        })
        .unwrap_or(SpanOut { start: 0, end: 0 });
    DiagnosticOut {
        severity: DiagnosticSeverity::Error,
        message: e.message(),
        span,
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
            message: e.message(),
            span: SpanOut {
                start: e.span.start,
                end: e.span.end,
            },
            report: None,
        },
    }
}

fn byte_offset_to_line_number(source: &str, byte_offset: usize) -> usize {
    source
        .as_bytes()
        .iter()
        .take(byte_offset.min(source.len()))
        .filter(|&&b| b == b'\n')
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
