//! Stream → measure chunks, re-barred by each measure's beat capacity.

use crate::ast::parsed::ScoreEvent;
use crate::error::{Diagnostic, Span, Spanned};
use crate::parser::score::timed_parser::{GroupStack, TimedLexToken};

use super::timing::{
    is_inline_directive, normalize_whitespace, rest_fill, timed_layout, TimedCluster, TimedLineKind,
};
use super::{StreamDiagnostic, StreamDiagnosticSeverity};

/// Capacity used past the last measure group when no capacity is known at all.
const DEFAULT_CAPACITY: u32 = 16;

pub(super) struct SplitStream {
    /// Whitespace-normalized chunk per measure.
    pub(super) chunks: Vec<String>,
    pub(super) measure_starts: Vec<usize>,
    pub(super) diagnostics: Vec<StreamDiagnostic>,
}

/// Where a measure starts in the stream: the first cluster's index and the
/// byte offset the measure's chunk is cut at.
struct MeasureCut {
    cluster_index: usize,
    offset: usize,
}

struct Layout {
    cuts: Vec<MeasureCut>,
    /// Quarter-beats of rest needed to complete the last measure.
    trailing_rest: u32,
}

pub(super) fn split_timed_stream(
    stream: &str,
    kind: TimedLineKind,
    capacities: &[u32],
) -> SplitStream {
    let mut diagnostics = forbidden_character_errors(stream);
    let sanitized: String = stream
        .chars()
        .map(|character| {
            if matches!(character, '|' | '@') {
                ' '
            } else {
                character
            }
        })
        .collect();
    let parsed = match kind.parse(&sanitized, &mut GroupStack::default()) {
        Ok(parsed) => parsed,
        Err(error) => {
            let span = error.span().copied().unwrap_or(Span::new(0, stream.len()));
            diagnostics.push(error_at(span, error.message()));
            return SplitStream {
                chunks: Vec::new(),
                measure_starts: Vec::new(),
                diagnostics,
            };
        }
    };
    let tokens = kind.lex(&sanitized).unwrap_or_default();
    diagnostics.extend(
        parsed
            .lex_errors
            .iter()
            .map(|error| error_at(error.span, error.message())),
    );
    diagnostics.extend(parsed.chord_errors.iter().map(stream_diagnostic));
    diagnostics.extend(inline_directive_errors(&parsed.events));

    let timed = timed_layout(&parsed.events);
    let capacity_of = |measure: usize| {
        capacities
            .get(measure)
            .or(capacities.last())
            .copied()
            .unwrap_or(DEFAULT_CAPACITY)
            * timed.multiplier
    };
    let layout = lay_out(&timed.clusters, &tokens, capacity_of, &mut diagnostics);
    let trailing_rest = layout.trailing_rest / timed.multiplier;
    if layout.trailing_rest % timed.multiplier != 0 {
        diagnostics.push(error_at(
            Span::new(stream.len(), stream.len()),
            "the stream ends partway through a tuplet beat",
        ));
    }
    let ranges = measure_ranges(stream, &layout.cuts);
    let last_index = ranges.len().saturating_sub(1);
    let chunks = ranges
        .iter()
        .enumerate()
        .map(|(index, range)| {
            let written =
                normalize_whitespace(stream.get(range.start..range.end).unwrap_or_default());
            match (index == last_index, rest_fill(trailing_rest)) {
                (true, rest) if !rest.is_empty() => format!("{written} {rest}").trim().to_string(),
                _ => written,
            }
        })
        .collect();
    SplitStream {
        chunks,
        measure_starts: ranges
            .iter()
            .map(|range| first_non_whitespace(stream, range.start))
            .collect(),
        diagnostics,
    }
}

/// The stream range each measure's chunk is cut from; the first one starts
/// at the stream's start so nothing before the first cluster is lost.
fn measure_ranges(stream: &str, cuts: &[MeasureCut]) -> Vec<Span> {
    let ends = cuts
        .iter()
        .skip(1)
        .map(|cut| cut.offset)
        .chain([stream.len()]);
    cuts.iter()
        .zip(ends)
        .enumerate()
        .map(|(index, (cut, end))| Span::new(if index == 0 { 0 } else { cut.offset }, end))
        .collect()
}

fn lay_out(
    clusters: &[TimedCluster],
    tokens: &[Spanned<TimedLexToken>],
    capacity_of: impl Fn(usize) -> u32,
    diagnostics: &mut Vec<StreamDiagnostic>,
) -> Layout {
    let mut cuts: Vec<MeasureCut> = Vec::new();
    let mut filled = 0;
    for (index, cluster) in clusters.iter().enumerate() {
        let starts_measure = cuts.is_empty() || filled >= capacity_of(cuts.len() - 1);
        if starts_measure {
            if let Some(error) = tuplet_crossing_error(clusters, index, cuts.len()) {
                diagnostics.push(error);
            }
            cuts.push(MeasureCut {
                cluster_index: index,
                offset: cut_offset(cluster.start, tokens),
            });
            filled = 0;
        }
        let capacity = capacity_of(cuts.len() - 1);
        if filled + cluster.duration > capacity {
            diagnostics.push(error_at(
                Span::new(cluster.start, cluster.end),
                format!(
                    "this note crosses the barline of measure {}; split it with a tie (`~`)",
                    cuts.len()
                ),
            ));
            filled = capacity;
        } else {
            filled += cluster.duration;
        }
    }
    diagnostics.extend(extension_in_tuplet_measure_errors(clusters, &cuts));
    let trailing_rest = cuts
        .len()
        .checked_sub(1)
        .map_or(0, |last| capacity_of(last).saturating_sub(filled));
    Layout {
        cuts,
        trailing_rest,
    }
}

/// A tuplet whose members land on both sides of a barline.
fn tuplet_crossing_error(
    clusters: &[TimedCluster],
    index: usize,
    measure: usize,
) -> Option<StreamDiagnostic> {
    let previous = clusters.get(index.checked_sub(1)?)?.tuplet?;
    let current = clusters.get(index)?.tuplet?;
    if previous.id != current.id {
        return None;
    }
    let members: Vec<&TimedCluster> = clusters
        .iter()
        .filter(|cluster| cluster.tuplet.is_some_and(|tuplet| tuplet.id == current.id))
        .collect();
    let start = members.first()?.start;
    let end = members.last()?.end;
    Some(error_at(
        Span::new(start, end),
        format!("this tuplet crosses the barline of measure {measure}"),
    ))
}

/// The real parser counts `-` on a tuplet measure's rescaled grid, so a
/// measure mixing the two could be re-barred differently than it renders.
fn extension_in_tuplet_measure_errors(
    clusters: &[TimedCluster],
    cuts: &[MeasureCut],
) -> Vec<StreamDiagnostic> {
    let ends = cuts
        .iter()
        .skip(1)
        .map(|cut| cut.cluster_index)
        .chain([clusters.len()]);
    cuts.iter()
        .zip(ends)
        .filter_map(|(cut, end)| {
            let measure = clusters.get(cut.cluster_index..end)?;
            let has_tuplet = measure.iter().any(|cluster| cluster.tuplet.is_some());
            let extended = measure.iter().find(|cluster| cluster.has_extension)?;
            has_tuplet.then(|| {
                error_at(
                    Span::new(extended.start, extended.end),
                    "a `-` extension can't share a measure with a tuplet in a part stream",
                )
            })
        })
        .collect()
}

/// Moves a cut back over the `(` / `N:{` that open right before a cluster,
/// so they travel with the measure they open.
fn cut_offset(cluster_start: usize, tokens: &[Spanned<TimedLexToken>]) -> usize {
    tokens
        .iter()
        .rev()
        .skip_while(|token| token.span.start >= cluster_start)
        .take_while(|token| {
            matches!(
                token.value,
                TimedLexToken::LParen | TimedLexToken::LBrace { .. }
            )
        })
        .last()
        .map_or(cluster_start, |token| token.span.start)
}

fn forbidden_character_errors(stream: &str) -> Vec<StreamDiagnostic> {
    stream
        .char_indices()
        .filter_map(|(offset, character)| {
            let message = match character {
                '|' => "barlines are not allowed in a part stream; measures are inferred from beat counts",
                '@' => "`@` is only allowed in follow parts",
                _ => return None,
            };
            Some(error_at(Span::new(offset, offset + 1), message))
        })
        .collect()
}

fn inline_directive_errors(events: &[Spanned<ScoreEvent>]) -> Vec<StreamDiagnostic> {
    events
        .iter()
        .filter(|event| is_inline_directive(&event.value))
        .map(|event| {
            error_at(
                event.span,
                "directives are not allowed in a part stream; put them on the measure's directive line",
            )
        })
        .collect()
}

fn stream_diagnostic(diagnostic: &Diagnostic) -> StreamDiagnostic {
    StreamDiagnostic {
        span: diagnostic.span(),
        message: diagnostic.message(),
        severity: match diagnostic {
            Diagnostic::Warning(_) => StreamDiagnosticSeverity::Warning,
            Diagnostic::Error(_) => StreamDiagnosticSeverity::Error,
        },
    }
}

fn error_at(span: Span, message: impl Into<String>) -> StreamDiagnostic {
    StreamDiagnostic {
        span,
        message: message.into(),
        severity: StreamDiagnosticSeverity::Error,
    }
}

fn first_non_whitespace(stream: &str, from: usize) -> usize {
    stream
        .get(from..)
        .and_then(|rest| rest.find(|character: char| !character.is_whitespace()))
        .map_or(from, |offset| from + offset)
}

#[cfg(test)]
#[path = "split_tests.rs"]
mod tests;
