use crate::ast::grouped::{MultiPartMeasure, Score};
use crate::error::{Diagnostic, RecoverableError};

fn marker_indices(score: &Score, marker: impl Fn(&MultiPartMeasure) -> bool) -> Vec<usize> {
    score
        .measures
        .iter()
        .enumerate()
        .filter(|(_, m)| marker(m))
        .map(|(i, _)| i)
        .collect()
}

fn push_error_at(score: &mut Score, measure_idx: usize, message: String) {
    if let Some(measure) = score.measures.get_mut(measure_idx) {
        let span = measure.source_span;
        measure
            .diagnostics
            .push(Diagnostic::Error(RecoverableError::general(span, message)));
    }
}

/// Validates the D.C./D.S. al Coda navigation markers (`dcalcoda`, `tocoda`,
/// `coda`, `segno`, `dsalcoda`) and reports a recoverable diagnostic on the
/// offending measure for any invalid combination:
/// - `dcalcoda`/`tocoda`/`coda`, and `segno`/`dsalcoda`/`tocoda`/`coda` must
///   each appear all together or not at all, at most once each.
/// - `segno` must occur at or before `dsalcoda`.
/// - `tocoda` must occur before `coda`.
/// - `dcalcoda` cannot be combined with `segno`/`dsalcoda`.
///
/// This mirrors the validation in `midi::navigation`, which only runs when
/// generating MIDI/WAV audio; this pass ensures the same errors surface for
/// SVG/PDF rendering as well.
pub(super) fn validate_navigation_markers(score: &mut Score) {
    let dc_indices = marker_indices(score, |m| m.dc_al_coda);
    let segno_indices = marker_indices(score, |m| m.segno);
    let ds_indices = marker_indices(score, |m| m.ds_al_coda);
    let to_coda_indices = marker_indices(score, |m| m.to_coda);
    let coda_indices = marker_indices(score, |m| m.coda);

    let uses_dc_scheme = !dc_indices.is_empty();
    let uses_ds_scheme = !segno_indices.is_empty() || !ds_indices.is_empty();

    if !uses_dc_scheme && !uses_ds_scheme && to_coda_indices.is_empty() && coda_indices.is_empty()
    {
        return;
    }

    if uses_dc_scheme && uses_ds_scheme {
        let measure_idx = dc_indices.first().copied().unwrap_or(0);
        push_error_at(
            score,
            measure_idx,
            "dcalcoda cannot appear together with segno or dsalcoda".to_string(),
        );
        return;
    }

    let (repeat_after_indices, repeat_from_indices) = if uses_ds_scheme {
        (ds_indices, segno_indices)
    } else {
        (dc_indices, vec![0])
    };

    let index_groups = [
        &repeat_after_indices,
        &repeat_from_indices,
        &to_coda_indices,
        &coda_indices,
    ];
    if index_groups.iter().any(|indices| indices.len() != 1) {
        let measure_idx = index_groups
            .iter()
            .find_map(|indices| indices.first())
            .copied()
            .unwrap_or(0);
        let marker_names = if uses_ds_scheme {
            "segno, dsalcoda, tocoda, and coda"
        } else {
            "dcalcoda, tocoda, and coda"
        };
        push_error_at(
            score,
            measure_idx,
            format!(
                "{marker_names} must each appear exactly once, and they must all appear together"
            ),
        );
        return;
    }

    let repeat_after = repeat_after_indices.first().copied().unwrap_or_default();
    let repeat_from = repeat_from_indices.first().copied().unwrap_or_default();
    let to_coda = to_coda_indices.first().copied().unwrap_or_default();
    let coda = coda_indices.first().copied().unwrap_or_default();

    if uses_ds_scheme && repeat_from > repeat_after {
        push_error_at(
            score,
            repeat_from,
            "segno must occur at or before dsalcoda".to_string(),
        );
        return;
    }

    if to_coda >= coda {
        push_error_at(score, to_coda, "tocoda must occur before coda".to_string());
    }
}
