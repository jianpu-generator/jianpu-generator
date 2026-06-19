use crate::ast::grouped::Score;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use crate::parser;

/// Find the index of the measure whose `source_span` contains `byte_offset`.
///
/// Returns `None` when `byte_offset` falls outside all measure spans
/// (e.g. in `[metadata]`, `[parts]`, or a directive line).
pub fn find_measure_at_byte_offset(score: &Score, byte_offset: usize) -> Option<usize> {
    score
        .measures
        .iter()
        .position(|m| m.source_span.start <= byte_offset && byte_offset <= m.source_span.end)
}

/// Find the index of the measure that contains the given 0-based line number.
///
/// Converts `line_number` to the byte offset of the first character on that
/// line, then delegates to [`find_measure_at_byte_offset`].  Returns `None`
/// when the line falls outside all measure spans (e.g. `[metadata]`,
/// `[parts]`, directive lines, or blank separator lines).
pub fn find_measure_at_line_number(
    score: &Score,
    source: &str,
    line_number: usize,
) -> Option<usize> {
    let byte_offset: usize = source
        .split('\n')
        .take(line_number)
        .map(|line| line.len() + 1) // +1 for the '\n' byte
        .sum();
    find_measure_at_byte_offset(score, byte_offset)
}

/// Source byte ranges for a measure in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureSourceSpan {
    /// Inclusive start of note content (for cursor/selection mapping).
    pub start: usize,
    /// Exclusive end of measure content in source.
    pub end: usize,
    /// Byte offset of the first source line in this measure group, for view zones.
    pub view_zone_start: usize,
    /// Section label from `label="..."` directive, if present on this measure.
    pub section_label: Option<String>,
}

/// Return the source byte span of every measure in the compiled score.
///
/// Spans are in source order and correspond 1-to-1 with measures.
pub fn list_measure_spans_from_source(
    source: &str,
    filename: &str,
) -> Result<Vec<MeasureSourceSpan>, IrrecoverableError> {
    let sections = parser::load_document_sections(source)?;
    let (score_content, score_offset) = sections.score;
    let view_zone_starts =
        parser::score::measure_group::view_zone_starts(&score_content, score_offset);

    let score = crate::compile(source, filename)?;
    if view_zone_starts.len() != score.measures.len() {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::internal_invariant(
                Span::new(0, 0),
                format!(
                    "view zone starts ({}) and measures ({}) out of sync",
                    view_zone_starts.len(),
                    score.measures.len()
                ),
            ),
        ));
    }

    Ok(score
        .measures
        .iter()
        .zip(view_zone_starts)
        .map(|(measure, view_zone_start)| MeasureSourceSpan {
            start: measure.source_span.start,
            end: measure.source_span.end,
            view_zone_start,
            section_label: measure.label.clone(),
        })
        .collect())
}
