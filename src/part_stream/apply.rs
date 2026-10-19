//! Edited stream → source: rewrite only the measures whose chunk changed.

use itertools::Itertools;

use crate::error::Span;

use super::document::{ScoreDocument, ScoreLine, ScoreMeasure};
use super::split::split_timed_stream;
use super::timing::rest_fill;
use super::{
    editable_part, find_part, EditablePart, PartStreamApplyResult, PartStreamError,
    StreamDiagnostic, StreamDiagnosticSeverity,
};

/// Replace `span` of the source with `replacement`.
struct SourceEdit {
    span: Span,
    replacement: String,
}

impl SourceEdit {
    fn new(start: usize, end: usize, replacement: impl Into<String>) -> Self {
        Self {
            span: Span::new(start, end),
            replacement: replacement.into(),
        }
    }
}

pub(super) fn apply(source: &str, abbreviation: &str, stream: &str) -> PartStreamApplyResult {
    let loaded = ScoreDocument::load(source).and_then(|document| {
        let index = find_part(&document, abbreviation)?;
        let part = editable_part(&document, index)?;
        Ok(LoadedPart { document, part })
    });
    let LoadedPart { document, part } = match loaded {
        Ok(loaded) => loaded,
        Err(error) => return refused(&error),
    };
    let split = split_timed_stream(stream, part.kind, &document.capacities());
    let has_error = split
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == StreamDiagnosticSeverity::Error);
    if has_error {
        return PartStreamApplyResult {
            source: None,
            diagnostics: split.diagnostics,
            measure_starts: split.measure_starts,
            changed_measure_count: 0,
        };
    }
    let key = abbreviation;
    let mut edits: Vec<SourceEdit> = document
        .measures
        .iter()
        .enumerate()
        .filter_map(|(index, measure)| {
            measure_edit(
                measure,
                &part,
                key,
                part.chunks.get(index),
                split.chunks.get(index),
            )
        })
        .collect();
    let appended: Vec<&String> = split.chunks.iter().skip(document.measures.len()).collect();
    let changed_measure_count = edits.len() + appended.len();
    if !appended.is_empty() {
        let separator = if document.measures.is_empty() {
            ""
        } else {
            "\n\n"
        };
        let text = appended
            .iter()
            .map(|chunk| format!("[{key}] {chunk}"))
            .join("\n\n");
        edits.push(SourceEdit::new(
            document.append_offset,
            document.append_offset,
            format!("{separator}{text}"),
        ));
    }
    PartStreamApplyResult {
        source: Some(splice(source, edits)),
        diagnostics: split.diagnostics,
        measure_starts: split.measure_starts,
        changed_measure_count,
    }
}

struct LoadedPart {
    document: ScoreDocument,
    part: EditablePart,
}

fn refused(error: &PartStreamError) -> PartStreamApplyResult {
    PartStreamApplyResult {
        source: None,
        diagnostics: vec![StreamDiagnostic {
            span: Span::new(0, 0),
            message: error.describe(),
            severity: StreamDiagnosticSeverity::Error,
        }],
        measure_starts: Vec::new(),
        changed_measure_count: 0,
    }
}

/// The edit that turns `original` into `edited` in one measure, if any.
fn measure_edit(
    measure: &ScoreMeasure,
    part: &EditablePart,
    key: &str,
    original: Option<&String>,
    edited: Option<&String>,
) -> Option<SourceEdit> {
    let written = measure.written_line(part.index);
    match (edited, written) {
        (Some(chunk), _) if Some(chunk) == original => None,
        (Some(chunk), Some(line)) => Some(SourceEdit::new(
            line.content_start,
            line.content_end(),
            chunk.clone(),
        )),
        (Some(chunk), None) => Some(insert_line(
            measure,
            part.index,
            &format!("[{key}] {chunk}"),
        )),
        (None, None) => None,
        (None, Some(line)) if measure.data_lines.len() == 1 => {
            let rest = rest_fill(measure.capacity);
            (Some(&rest) != original)
                .then(|| SourceEdit::new(line.content_start, line.content_end(), rest))
        }
        (None, Some(line)) => Some(remove_line(measure, line)),
    }
}

/// Removing a line takes its own `\n`, or — for the group's last line — the
/// `\n` before it, so the blank line separating the next group stays intact.
fn remove_line(measure: &ScoreMeasure, line: &ScoreLine) -> SourceEdit {
    if line.line_end < measure.end {
        SourceEdit::new(line.line_start, line.line_end + 1, "")
    } else {
        SourceEdit::new(line.line_start.saturating_sub(1), line.line_end, "")
    }
}

/// A new `[Key] chunk` line, placed before the first line of a part declared
/// after this one, or after the group's last line.
fn insert_line(measure: &ScoreMeasure, part_index: usize, line: &str) -> SourceEdit {
    match measure
        .data_lines
        .iter()
        .find(|existing| existing.part_index > part_index)
    {
        Some(later) => SourceEdit::new(later.line_start, later.line_start, format!("{line}\n")),
        None => SourceEdit::new(measure.end, measure.end, format!("\n{line}")),
    }
}

fn splice(source: &str, edits: Vec<SourceEdit>) -> String {
    let mut result = String::with_capacity(source.len());
    let mut cursor = 0;
    for edit in edits.into_iter().sorted_by_key(|edit| edit.span.start) {
        result.push_str(source.get(cursor..edit.span.start).unwrap_or_default());
        result.push_str(&edit.replacement);
        cursor = cursor.max(edit.span.end);
    }
    result.push_str(source.get(cursor..).unwrap_or_default());
    result
}

#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;
