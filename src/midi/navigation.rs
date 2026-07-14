use crate::ast::grouped::Score;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};

fn marker_indices(
    score: &Score,
    marker: impl Fn(&crate::ast::grouped::MultiPartMeasure) -> bool,
) -> Vec<usize> {
    score
        .measures
        .iter()
        .enumerate()
        .filter(|(_, m)| marker(m))
        .map(|(i, _)| i)
        .collect()
}

fn invalid_at(score: &Score, measure_idx: usize, detail: String) -> IrrecoverableError {
    let span = score
        .measures
        .get(measure_idx)
        .map(|m| m.source_span)
        .unwrap_or(Span::new(0, 0));
    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(span, detail))
}

/// Validates that `dcalcoda`/`tocoda`/`coda` each appear exactly once (or not
/// at all) and that `tocoda` precedes `coda`. Returns their measure indices.
fn resolve_marker_indices(
    score: &Score,
) -> Result<Option<(usize, usize, usize)>, IrrecoverableError> {
    let dc_indices = marker_indices(score, |m| m.dc_al_coda);
    let to_coda_indices = marker_indices(score, |m| m.to_coda);
    let coda_indices = marker_indices(score, |m| m.coda);

    if dc_indices.is_empty() && to_coda_indices.is_empty() && coda_indices.is_empty() {
        return Ok(None);
    }

    if dc_indices.len() != 1 || to_coda_indices.len() != 1 || coda_indices.len() != 1 {
        let measure_idx = dc_indices
            .first()
            .or_else(|| to_coda_indices.first())
            .or_else(|| coda_indices.first())
            .copied()
            .unwrap_or(0);
        return Err(invalid_at(
            score,
            measure_idx,
            "dcalcoda, tocoda, and coda must each appear exactly once, and all three must appear together".to_string(),
        ));
    }

    let (Some(&dc), Some(&to), Some(&coda)) = (
        dc_indices.first(),
        to_coda_indices.first(),
        coda_indices.first(),
    ) else {
        return Err(invalid_at(
            score,
            0,
            "internal invariant violated: marker index missing after length check".to_string(),
        ));
    };

    if to >= coda {
        return Err(invalid_at(
            score,
            to,
            "tocoda must occur before coda".to_string(),
        ));
    }

    Ok(Some((dc, to, coda)))
}

/// Rebuilds `score.measures` to reflect D.C. al Coda navigation
/// (`dcalcoda`/`tocoda`/`coda` markers), so downstream MIDI/WAV generation
/// replays measures in actual playback order instead of written order.
///
/// - No markers present: returns the score unchanged.
/// - All three markers present exactly once, with `tocoda` before `coda`:
///   returns a score whose measures are the expanded playback sequence
///   (pass 1 through D.C. al Coda, then pass 2 from the start through To
///   Coda, then Coda through the literal end).
/// - Any other combination (partial set, duplicates, or `tocoda` at/after
///   `coda`) is an error.
pub fn expand_navigation(score: &Score) -> Result<Score, IrrecoverableError> {
    if score.measures.is_empty() {
        return Ok(score.clone());
    }

    let Some((dc, to, coda)) = resolve_marker_indices(score)? else {
        return Ok(score.clone());
    };

    let last = score.measures.len() - 1;
    let mut idx: Vec<usize> = Vec::new();
    idx.extend(0..=dc);
    idx.extend(0..=to);
    idx.extend(coda..=last);

    let measures = idx
        .into_iter()
        .filter_map(|i| score.measures.get(i).cloned())
        .collect();

    Ok(Score {
        metadata: score.metadata.clone(),
        measures,
        document_diagnostics: score.document_diagnostics.clone(),
    })
}

#[cfg(test)]
mod tests;
