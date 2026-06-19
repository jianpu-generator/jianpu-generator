use crate::ast::grouped::{GroupedMeasure, Notes};
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span, Warning,
};

fn apply_per_measure_errors(
    measure: &mut GroupedMeasure,
    beat_error: Option<&Warning>,
    dotted_eighth_errors: &[Diagnostic],
    dash_after_rest_error: Option<&RecoverableError>,
    chord_errors: &[Diagnostic],
    lex_error: Option<&RecoverableError>,
) {
    if let Some(beat_error) = beat_error {
        measure.beat_overflow_error = Some(beat_error.clone());
    }
    if !dotted_eighth_errors.is_empty() {
        measure.dotted_eighth_errors = dotted_eighth_errors.to_vec();
    }
    if let Some(dash_after_rest_error) = dash_after_rest_error {
        if measure.dash_after_rest_error.is_none() {
            measure.dash_after_rest_error = Some(dash_after_rest_error.clone());
        }
    }
    if !chord_errors.is_empty() {
        measure.chord_errors = chord_errors.to_vec();
    }
    if let Some(lex_error) = lex_error {
        measure.lex_error = Some(lex_error.clone());
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn align_empty_note_measures(
    measures: &mut Vec<GroupedMeasure>,
    empty_note_measure_spans: &[Option<Span>],
    per_measure_beat_errors: &[Option<Warning>],
    per_measure_dotted_eighth_errors: &[Vec<Diagnostic>],
    per_measure_dash_after_rest_errors: &[Option<RecoverableError>],
    per_measure_chord_errors: &[Vec<Diagnostic>],
    per_measure_lex_errors: &[Option<RecoverableError>],
) -> Result<(), IrrecoverableError> {
    if empty_note_measure_spans.is_empty() {
        for (idx, measure) in measures.iter_mut().enumerate() {
            apply_per_measure_errors(
                measure,
                per_measure_beat_errors.get(idx).and_then(|e| e.as_ref()),
                per_measure_dotted_eighth_errors
                    .get(idx)
                    .map_or(&[][..], Vec::as_slice),
                per_measure_dash_after_rest_errors
                    .get(idx)
                    .and_then(|e| e.as_ref()),
                per_measure_chord_errors
                    .get(idx)
                    .map_or(&[][..], Vec::as_slice),
                per_measure_lex_errors.get(idx).and_then(|e| e.as_ref()),
            );
        }
        return Ok(());
    }

    let mut filled = std::mem::take(measures).into_iter();
    let aligned = empty_note_measure_spans
        .iter()
        .enumerate()
        .map(|(idx, empty_span)| {
            let beat_error = per_measure_beat_errors.get(idx).and_then(|e| e.clone());
            let dotted_eighth_errors = per_measure_dotted_eighth_errors
                .get(idx)
                .cloned()
                .unwrap_or_default();
            let dash_after_rest_error = per_measure_dash_after_rest_errors
                .get(idx)
                .and_then(|e| e.clone());
            let chord_errors = per_measure_chord_errors
                .get(idx)
                .cloned()
                .unwrap_or_default();
            let lex_error = per_measure_lex_errors.get(idx).and_then(|e| e.clone());

            if let Some(span) = empty_span {
                Ok(GroupedMeasure {
                    notes: Notes { events: Vec::new() },
                    source_span: *span,
                    paired_lyrics: None,
                    lyrics_error: None,
                    beat_overflow_error: None,
                    dash_after_rest_error,
                    dotted_eighth_errors: Vec::new(),
                    chord_errors,
                    lex_error,
                })
            } else {
                let mut measure = filled.next().ok_or_else(|| {
                    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                        Span::new(0, 0),
                        "empty_note_measure_spans and grouped measures out of sync",
                    ))
                })?;
                apply_per_measure_errors(
                    &mut measure,
                    beat_error.as_ref(),
                    &dotted_eighth_errors,
                    dash_after_rest_error.as_ref(),
                    &chord_errors,
                    lex_error.as_ref(),
                );
                Ok(measure)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    *measures = aligned;
    Ok(())
}
