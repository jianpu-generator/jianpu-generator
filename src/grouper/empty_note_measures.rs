use crate::ast::grouped::{GroupedMeasure, Notes};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span};

pub fn align_empty_note_measures(
    measures: &mut Vec<GroupedMeasure>,
    empty_note_measure_spans: &[Option<Span>],
    per_measure_beat_errors: &[Option<RecoverableError>],
) -> Result<(), IrrecoverableError> {
    if empty_note_measure_spans.is_empty() {
        for (measure, error) in measures.iter_mut().zip(per_measure_beat_errors.iter()) {
            if error.is_some() {
                measure.beat_overflow_error = error.clone();
            }
        }
        return Ok(());
    }

    let mut filled = std::mem::take(measures).into_iter();
    let aligned = empty_note_measure_spans
        .iter()
        .zip(per_measure_beat_errors.iter())
        .map(|(empty_span, beat_error)| {
            if let Some(span) = empty_span {
                Ok(GroupedMeasure {
                    notes: Notes { events: Vec::new() },
                    source_span: *span,
                    paired_lyrics: None,
                    lyrics_error: None,
                    beat_overflow_error: None,
                    dash_after_rest_error: None,
                })
            } else {
                let mut measure = filled.next().ok_or_else(|| {
                    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                        Span::new(0, 0),
                        "empty_note_measure_spans and grouped measures out of sync",
                    ))
                })?;
                if beat_error.is_some() {
                    measure.beat_overflow_error = beat_error.clone();
                }
                Ok(measure)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    *measures = aligned;
    Ok(())
}
