use crate::ast::grouped::{Score, SequenceSpan};
use crate::error::{Diagnostic, RecoverableError};
use crate::navigation_markers::{gather_marker_indices, no_markers_present};
use crate::parser::sequence_parser::SequenceSection;

/// Resolves a parsed `# sequence` section against `score.measures`' labels
/// and stores the result on `score.sequence`, or leaves it `None` if the
/// section is absent or invalid.
///
/// - Mutually exclusive with inline D.C./D.S. al Coda/Fine navigation
///   markers: if both are present, a recoverable error is attached to the
///   first marker measure and the sequence is ignored (the score falls back
///   to marker-based navigation).
/// - Each label must be defined on at most one measure; a duplicate
///   definition is a recoverable error (attached to the second occurrence)
///   and the sequence is dropped, since it would be ambiguous which measure
///   a `# sequence` entry refers to.
/// - A `# sequence` entry referencing an undefined label is a recoverable
///   document-level error; that entry is skipped but the rest of the
///   sequence still resolves.
/// - A label defined but never referenced by `# sequence` is not an error.
pub(super) fn resolve_sequence(
    score: &mut Score,
    sequence: Option<SequenceSection>,
    parse_errors: Vec<RecoverableError>,
) {
    score
        .document_diagnostics
        .extend(parse_errors.into_iter().map(Diagnostic::Error));

    let Some(sequence) = sequence else {
        return;
    };

    if flag_conflict_with_inline_markers(score) {
        return;
    }

    let Some(label_starts) = collect_unique_label_starts(score) else {
        return;
    };

    let spans = build_spans(&label_starts, score.measures.len());
    score.sequence = Some(resolve_entries(score, sequence, &spans));
}

/// Attaches a recoverable error to the earliest inline navigation marker
/// measure and returns `true` if any such marker is present alongside a
/// `# sequence` section (the two schemes are mutually exclusive).
fn flag_conflict_with_inline_markers(score: &mut Score) -> bool {
    let markers = gather_marker_indices(score);
    if no_markers_present(&markers) {
        return false;
    }

    let measure_idx = [
        &markers.dc,
        &markers.segno,
        &markers.ds,
        &markers.to_coda,
        &markers.coda,
        &markers.dc_fine,
        &markers.ds_fine,
        &markers.fine,
    ]
    .iter()
    .filter_map(|indices| indices.first())
    .min()
    .copied();
    if let Some(measure_idx) = measure_idx {
        if let Some(measure) = score.measures.get_mut(measure_idx) {
            let span = measure.source_span;
            measure.diagnostics.push(Diagnostic::Error(RecoverableError::general(
                span,
                "inline navigation markers (dcalcoda/segno/tocoda/coda/dcalfine/fine/dsalcoda/dsalfine) cannot be combined with a `# sequence` section",
            )));
        }
    }
    true
}

/// Collects each label's first-occurrence measure index, or attaches a
/// recoverable error and returns `None` if a label is defined more than once.
fn collect_unique_label_starts(score: &mut Score) -> Option<Vec<(String, usize)>> {
    let mut label_starts: Vec<(String, usize)> = Vec::new();
    for (index, measure) in score.measures.iter().enumerate() {
        let Some(label) = &measure.label else {
            continue;
        };
        if label_starts.iter().any(|(l, _)| l == label) {
            score
                .document_diagnostics
                .push(Diagnostic::Error(RecoverableError::general(
                    measure.source_span,
                    format!("label \"{label}\" is defined more than once"),
                )));
            return None;
        }
        label_starts.push((label.clone(), index));
    }
    Some(label_starts)
}

/// Turns each label's start index into an inclusive span running to the
/// measure before the next label (or to the last measure, for the final one).
fn build_spans(label_starts: &[(String, usize)], measure_count: usize) -> Vec<SequenceSpan> {
    let last_measure_index = measure_count.saturating_sub(1);
    label_starts
        .iter()
        .enumerate()
        .map(|(position, (label, start))| {
            let end = label_starts
                .get(position + 1)
                .map(|(_, next_start)| next_start - 1)
                .unwrap_or(last_measure_index);
            SequenceSpan {
                label: label.clone(),
                start: *start,
                end,
            }
        })
        .collect()
}

/// Resolves each `# sequence` entry against the known spans, attaching a
/// recoverable error and skipping any entry that references an undefined
/// label.
fn resolve_entries(
    score: &mut Score,
    sequence: SequenceSection,
    spans: &[SequenceSpan],
) -> Vec<SequenceSpan> {
    sequence
        .entries
        .into_iter()
        .filter_map(
            |entry| match spans.iter().find(|span| span.label == entry.label) {
                Some(span) => Some(SequenceSpan {
                    label: span.label.clone(),
                    start: span.start,
                    end: span.end,
                }),
                None => {
                    score
                        .document_diagnostics
                        .push(Diagnostic::Error(RecoverableError::general(
                            entry.span,
                            format!("sequence references undefined label \"{}\"", entry.label),
                        )));
                    None
                }
            },
        )
        .collect()
}

#[cfg(test)]
mod tests;
