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
    expand_navigation_with_origins(score).map(|(score, _)| score)
}

/// Same as [`expand_navigation`], but also returns a same-length `Vec<usize>`
/// mapping each measure in the expanded score back to its index in the
/// original written `score.measures`.
pub fn expand_navigation_with_origins(
    score: &Score,
) -> Result<(Score, Vec<usize>), IrrecoverableError> {
    if score.measures.is_empty() {
        return Ok((score.clone(), Vec::new()));
    }

    let Some((dc, to, coda)) = resolve_marker_indices(score)? else {
        let origins = (0..score.measures.len()).collect();
        return Ok((score.clone(), origins));
    };

    let last = score.measures.len() - 1;
    let mut idx: Vec<usize> = Vec::new();
    idx.extend(0..=dc);
    idx.extend(0..=to);
    idx.extend(coda..=last);

    let measures = idx
        .iter()
        .filter_map(|&i| score.measures.get(i).cloned())
        .collect();

    Ok((
        Score {
            metadata: score.metadata.clone(),
            measures,
            document_diagnostics: score.document_diagnostics.clone(),
        },
        idx,
    ))
}

/// Smallest position `>= min_pos` in `origins` whose value equals
/// `written_index`, i.e. the earliest playback position (at or after
/// `min_pos`) at which the given written measure is played.
pub fn earliest_playback_position(
    origins: &[usize],
    written_index: usize,
    min_pos: usize,
) -> Option<usize> {
    origins
        .iter()
        .enumerate()
        .skip(min_pos)
        .find(|(_, &origin)| origin == written_index)
        .map(|(pos, _)| pos)
}

/// Translates a written measure index into its position in actual playback
/// order (see [`expand_navigation`]). Falls back to the written index
/// against the original score if the measure has no reachable position
/// (e.g. it lies between `dcalcoda` and `coda`, or navigation markers are
/// absent — in which case the mapping is already the identity).
pub fn expand_for_measure(
    score: &Score,
    measure_index: usize,
) -> Result<(Score, usize), IrrecoverableError> {
    let (expanded, origins) = expand_navigation_with_origins(score)?;
    match earliest_playback_position(&origins, measure_index, 0) {
        Some(position) => Ok((expanded, position)),
        None => Ok((score.clone(), measure_index)),
    }
}

/// Same as [`expand_for_measure`], but for a `start..=end` written range: maps
/// `start` to its earliest playback position, then maps `end` to its
/// *last* occurrence at or after that position — the final time `end` is
/// reached in the performance tail starting at `start`. This is what makes
/// "play written measure X through the last written measure" (the web app's
/// "play from current measure") follow every repeat/jump instead of
/// stopping at `end`'s first, mid-performance occurrence.
/// Falls back to the original written range if either endpoint has no
/// reachable position, or if `start_index > end_index`.
pub fn expand_for_measure_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Result<(Score, usize, usize), IrrecoverableError> {
    if start_index > end_index {
        return Ok((score.clone(), start_index, end_index));
    }
    let (expanded, origins) = expand_navigation_with_origins(score)?;
    let mapped = earliest_playback_position(&origins, start_index, 0).and_then(|start_pos| {
        origins
            .iter()
            .enumerate()
            .skip(start_pos)
            .filter(|(_, &origin)| origin == end_index)
            .next_back()
            .map(|(end_pos, _)| (start_pos, end_pos))
    });
    match mapped {
        Some((start_pos, end_pos)) => Ok((expanded, start_pos, end_pos)),
        None => Ok((score.clone(), start_index, end_index)),
    }
}

#[cfg(test)]
mod tests;
