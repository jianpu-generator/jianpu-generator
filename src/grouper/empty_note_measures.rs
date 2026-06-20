use crate::ast::grouped::{GroupedMeasure, Notes};
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span, Warning,
};

fn apply_per_measure_errors(
    measure: &mut GroupedMeasure,
    idx: usize,
    errors: &PerMeasureErrors<'_>,
) {
    if let Some(e) = errors.beat_errors.get(idx).and_then(|e| e.as_ref()) {
        measure.beat_overflow_error = Some(e.clone());
    }
    let dotted = errors
        .dotted_eighth_errors
        .get(idx)
        .map_or(&[][..], Vec::as_slice);
    if !dotted.is_empty() {
        measure.dotted_eighth_errors = dotted.to_vec();
    }
    if let Some(e) = errors
        .dash_after_rest_errors
        .get(idx)
        .and_then(|e| e.as_ref())
    {
        if measure.dash_after_rest_error.is_none() {
            measure.dash_after_rest_error = Some(e.clone());
        }
    }
    let chords = errors.chord_errors.get(idx).map_or(&[][..], Vec::as_slice);
    if !chords.is_empty() {
        measure.chord_errors = chords.to_vec();
    }
    if let Some(e) = errors.lex_errors.get(idx).and_then(|e| e.as_ref()) {
        measure.lex_error = Some(e.clone());
    }
    if let Some(e) = errors.lyrics_errors.get(idx).and_then(|e| e.as_ref()) {
        measure.lyrics_parse_error = Some(e.clone());
    }
}

pub(super) struct PerMeasureErrors<'a> {
    pub(super) beat_errors: &'a [Option<Warning>],
    pub(super) dotted_eighth_errors: &'a [Vec<Diagnostic>],
    pub(super) dash_after_rest_errors: &'a [Option<RecoverableError>],
    pub(super) chord_errors: &'a [Vec<Diagnostic>],
    pub(super) lex_errors: &'a [Option<RecoverableError>],
    pub(super) lyrics_errors: &'a [Option<RecoverableError>],
}

pub(super) fn align_empty_note_measures(
    measures: &mut Vec<GroupedMeasure>,
    empty_note_measure_spans: &[Option<Span>],
    errors: &PerMeasureErrors<'_>,
) -> Result<(), IrrecoverableError> {
    if empty_note_measure_spans.is_empty() {
        for (idx, measure) in measures.iter_mut().enumerate() {
            apply_per_measure_errors(measure, idx, errors);
        }
        return Ok(());
    }

    let mut filled = std::mem::take(measures).into_iter();
    let aligned = empty_note_measure_spans
        .iter()
        .enumerate()
        .map(|(idx, empty_span)| {
            if let Some(span) = empty_span {
                Ok(GroupedMeasure {
                    notes: Notes { events: Vec::new() },
                    source_span: *span,
                    paired_lyrics: None,
                    lyrics_error: None,
                    beat_overflow_error: None,
                    dash_after_rest_error: errors
                        .dash_after_rest_errors
                        .get(idx)
                        .and_then(|e| e.clone()),
                    dotted_eighth_errors: Vec::new(),
                    chord_errors: errors.chord_errors.get(idx).cloned().unwrap_or_default(),
                    lex_error: errors.lex_errors.get(idx).and_then(|e| e.clone()),
                    lyrics_parse_error: errors.lyrics_errors.get(idx).and_then(|e| e.clone()),
                    extension_no_preceding_event_error: None,
                })
            } else {
                let mut measure = filled.next().ok_or_else(|| {
                    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                        Span::new(0, 0),
                        "empty_note_measure_spans and grouped measures out of sync",
                    ))
                })?;
                apply_per_measure_errors(&mut measure, idx, errors);
                Ok(measure)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    *measures = aligned;
    Ok(())
}
