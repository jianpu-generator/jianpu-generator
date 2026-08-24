use jianpu_generator::error::{Diagnostic, IrrecoverableError, Warning};
use serde::Serialize;
use tsify::Tsify;

use crate::svg_types::SvgDocumentOut;

pub(crate) use crate::lyric_selection_types::{
    GroupLyricSelectionResponse, LyricCellIn, LyricSelectionRunOut,
};
pub(crate) use crate::note_selection_types::{
    GroupNoteSelectionResponse, NoteCellIn, NoteSelectionRunOut,
};

#[cfg(feature = "midi")]
pub(crate) use crate::types_export::{GenerateMidiResponse, GenerateSplitMidisResponse};
#[cfg(feature = "pdf")]
pub(crate) use crate::types_export::{GeneratePdfResponse, GenerateSplitPdfsResponse};
#[cfg(feature = "wav")]
pub(crate) use crate::types_export::{
    GenerateSplitWavsResponse, GenerateWavResponse, NoteTimingOut, NoteTimingsResponse,
};

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct SpanOut {
    /// UTF-8 byte offset (inclusive).
    pub start: usize,
    /// UTF-8 byte offset (exclusive).
    pub end: usize,
}

/// One inclusive measure-index range from JS, deserialized via
/// `serde_wasm_bindgen::from_value` rather than a wasm-bindgen `Vec<T>` param
/// (which only works for `JsCast` types) — see [`crate::note_selection_types::NoteCellIn`]
/// for the same convention. Maps 1:1 onto [`jianpu_generator::grid_layout::MeasureRange`].
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MeasureRangeIn {
    pub start: usize,
    pub end: usize,
}

impl From<MeasureRangeIn> for jianpu_generator::grid_layout::MeasureRange {
    fn from(r: MeasureRangeIn) -> Self {
        jianpu_generator::grid_layout::MeasureRange {
            start: r.start,
            end: r.end,
        }
    }
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
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct PartOut {
    pub abbreviation: String,
    pub display_name: String,
    pub has_lyrics: bool,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum PartDeclarationModeOut {
    Chords,
    Notes,
    #[serde(rename = "notes+lyrics")]
    NotesLyrics,
    Percussion,
    Follow,
    Lyrics,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub struct PartDeclarationOut {
    pub abbreviation: String,
    pub display_name: String,
    pub line_number: u32,
    pub mode: PartDeclarationModeOut,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soundfont: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub octave_offset: Option<i8>,
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
    Ok {
        parts: Vec<PartOut>,
        declarations: Vec<PartDeclarationOut>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListPartDeclarationsResponse {
    Ok {
        declarations: Vec<PartDeclarationOut>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[derive(Debug, Clone, Copy, Tsify, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum SymbolKindOut {
    Abbreviation,
    SectionLabel,
}

#[derive(Debug, Clone, Copy, Tsify, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum OccurrenceRoleOut {
    Declaration,
    Reference,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct SymbolOccurrenceOut {
    pub span: SpanOut,
    /// The region a caret may rest in to trigger a rename of this occurrence;
    /// usually equal to `span` but wider for occurrences whose renamable text
    /// sits inside a larger token (e.g. a section label declaration's `span`
    /// covers just the quoted text in `label="C"`, while `hit_span` covers
    /// the whole token).
    pub hit_span: SpanOut,
    pub role: OccurrenceRoleOut,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct SymbolOut {
    pub name: String,
    pub kind: SymbolKindOut,
    pub occurrences: Vec<SymbolOccurrenceOut>,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListSymbolsResponse {
    Ok { symbols: Vec<SymbolOut> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct TextEditOut {
    pub span: SpanOut,
    pub replacement: String,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum RenameSymbolResponse {
    Ok { edits: Vec<TextEditOut> },
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

#[derive(Debug, Clone, Tsify, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub struct NoteSpanOut {
    /// Index into the compiled score's parts, matching the SVG's
    /// `data-part-index` attribute on a `Tag::Note` group.
    pub source_part_index: usize,
    /// Matches the SVG's `data-note-id` attribute on a `Tag::Note` group.
    pub note_id: usize,
    /// Index into the score's measures, in source order.
    pub measure_index: usize,
    /// Inclusive start byte of this event's token in the original source.
    /// Absent for a rest, which has no single source token to map to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<usize>,
    /// Exclusive end byte of this event's token in the original source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<usize>,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListNoteSpansResponse {
    Ok { spans: Vec<NoteSpanOut> },
    Err,
}

#[derive(Debug, Clone, Tsify, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub struct LyricSpanOut {
    /// Index into the compiled score's parts, matching the SVG's
    /// `data-part-index` attribute on a `Tag::Lyric` group.
    pub source_part_index: usize,
    /// Matches the SVG's `data-note-id` attribute on a `Tag::Lyric` group.
    pub note_id: usize,
    /// Matches the SVG's `data-verse` attribute on a `Tag::Lyric` group.
    pub verse: usize,
    /// Index into the score's measures, in source order.
    pub measure_index: usize,
    /// Inclusive start byte of this syllable's own token in the original source.
    pub start: usize,
    /// Exclusive end byte of this syllable's own token in the original source.
    pub end: usize,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListLyricSpansResponse {
    Ok { spans: Vec<LyricSpanOut> },
    Err,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct SectionRangeOut {
    pub first_line: usize,
    pub last_line: usize,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct SequenceEntryOut {
    pub label: String,
    pub start_measure_index: usize,
    pub end_measure_index: usize,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListMeasureSpansResponse {
    Ok {
        spans: Vec<MeasureSpanOut>,
        section_ranges: Vec<SectionRangeOut>,
        sequence_entries: Vec<SequenceEntryOut>,
    },
    Err,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct DiagnosticMessageOut {
    pub message: String,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct DiagnosticViewZoneOut {
    pub severity: DiagnosticSeverity,
    /// 1-based line number; view zone is inserted after this line.
    pub after_line_number: usize,
    pub messages: Vec<DiagnosticMessageOut>,
}

pub(crate) fn diagnostic_from_error(e: &IrrecoverableError) -> DiagnosticOut {
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
    }
}

pub(crate) fn diagnostic_from_warning(e: Warning) -> DiagnosticOut {
    DiagnosticOut {
        severity: DiagnosticSeverity::Warning,
        message: e.message,
        span: SpanOut {
            start: e.span.start,
            end: e.span.end,
        },
    }
}

pub(crate) fn diagnostic_from_diagnostic(d: Diagnostic) -> DiagnosticOut {
    match d {
        Diagnostic::Warning(w) => diagnostic_from_warning(w),
        Diagnostic::Error(e) => DiagnosticOut {
            severity: DiagnosticSeverity::Error,
            message: e.message(),
            span: SpanOut {
                start: e.span.start,
                end: e.span.end,
            },
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
