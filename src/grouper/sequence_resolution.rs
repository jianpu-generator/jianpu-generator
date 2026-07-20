use crate::ast::grouped::{Score, SequenceSpan};
use crate::ast::parsed::PartDecl;
use crate::error::{Diagnostic, RecoverableError};
use crate::parser::group_parser::GroupSection;
use crate::parser::sequence_parser::SequenceSection;

/// Resolves a parsed `# sequence` section against `score.measures`' labels
/// and stores the result on `score.sequence`, or leaves it `None` if the
/// section is absent or invalid.
///
/// - Each label must be defined on at most one measure; a duplicate
///   definition is a recoverable error (attached to the second occurrence)
///   and the sequence is dropped, since it would be ambiguous which measure
///   a `# sequence` entry refers to.
/// - A `# sequence` entry referencing an undefined label is a recoverable
///   document-level error; that entry is skipped but the rest of the
///   sequence still resolves.
/// - A label defined but never referenced by `# sequence` is not an error.
/// - An entry's `(-abbrev ...)` suffix referencing an abbreviation that
///   matches no declared part or group is a recoverable document-level
///   error; that abbreviation is dropped from the entry's omissions but the
///   rest of the entry still resolves.
pub(super) fn resolve_sequence(
    score: &mut Score,
    sequence: Option<SequenceSection>,
    parse_errors: Vec<RecoverableError>,
    declarations: &[PartDecl],
    group: Option<&GroupSection>,
) {
    score
        .document_diagnostics
        .extend(parse_errors.into_iter().map(Diagnostic::Error));

    let Some(sequence) = sequence else {
        return;
    };

    let Some(label_starts) = collect_unique_label_starts(score) else {
        return;
    };

    let spans = build_spans(&label_starts, score.measures.len());
    score.sequence = Some(resolve_entries(
        score,
        sequence,
        &spans,
        declarations,
        group,
    ));
}

/// Expands a `# sequence` omission abbreviation into the individual part
/// abbreviations it refers to: a plain part abbreviation expands to itself;
/// a group abbreviation expands (recursively, since a group's members may
/// themselves be earlier groups) to its member parts' abbreviations.
/// Returns `None` if the abbreviation matches neither a declared part nor a
/// declared group.
fn expand_abbreviation(
    abbreviation: &str,
    declarations: &[PartDecl],
    group: Option<&GroupSection>,
) -> Option<Vec<String>> {
    expand_abbreviation_visited(abbreviation, declarations, group, &mut Vec::new())
}

/// Same as [`expand_abbreviation`], but tracks already-visited group
/// abbreviations in `visited` to guard against self-referential group
/// cycles (an already-visited group expands to nothing further).
fn expand_abbreviation_visited<'a>(
    abbreviation: &'a str,
    declarations: &[PartDecl],
    group: Option<&'a GroupSection>,
    visited: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    if declarations
        .iter()
        .any(|decl| decl.abbreviation == abbreviation)
    {
        return Some(vec![abbreviation.to_string()]);
    }
    let group_def = group?
        .groups
        .iter()
        .find(|def| def.abbreviation == abbreviation)?;
    if visited.contains(&abbreviation) {
        return Some(Vec::new());
    }
    visited.push(abbreviation);
    Some(
        group_def
            .members
            .iter()
            .flat_map(|member| {
                expand_abbreviation_visited(member, declarations, group, visited)
                    .unwrap_or_default()
            })
            .collect(),
    )
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
                omit_parts: Vec::new(),
                omit_parts_display: Vec::new(),
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
    declarations: &[PartDecl],
    group: Option<&GroupSection>,
) -> Vec<SequenceSpan> {
    sequence
        .entries
        .into_iter()
        .filter_map(
            |entry| match spans.iter().find(|span| span.label == entry.label) {
                Some(span) => {
                    let resolved = resolve_omit_parts(
                        score,
                        &entry.label,
                        &entry.omit_parts,
                        entry.span,
                        declarations,
                        group,
                    );
                    Some(SequenceSpan {
                        label: span.label.clone(),
                        start: span.start,
                        end: span.end,
                        omit_parts: resolved.expanded,
                        omit_parts_display: resolved.display,
                    })
                }
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

/// The result of resolving an entry's `(-abbrev ...)` suffix: the
/// individual part abbreviations to filter out at MIDI-expansion time
/// (`expanded`, with any group abbreviation spelled out to its members),
/// and the abbreviations as written, for display (`display`, which keeps a
/// group abbreviation as-is rather than expanding it).
struct ResolvedOmitParts {
    expanded: Vec<String>,
    display: Vec<String>,
}

/// Validates each omitted abbreviation against the declared parts/groups,
/// attaching a recoverable error and dropping any abbreviation that matches
/// neither.
fn resolve_omit_parts(
    score: &mut Score,
    label: &str,
    omit_parts: &[String],
    span: crate::error::Span,
    declarations: &[PartDecl],
    group: Option<&GroupSection>,
) -> ResolvedOmitParts {
    let mut expanded = Vec::new();
    let mut display = Vec::new();
    for abbreviation in omit_parts {
        match expand_abbreviation(abbreviation, declarations, group) {
            Some(parts) => {
                expanded.extend(parts);
                display.push(abbreviation.clone());
            }
            None => {
                score
                    .document_diagnostics
                    .push(Diagnostic::Error(RecoverableError::general(
                        span,
                        format!(
                            "sequence entry \"{label}\" omits unknown part/group \"{abbreviation}\""
                        ),
                    )));
            }
        }
    }
    ResolvedOmitParts { expanded, display }
}

#[cfg(test)]
mod tests;
