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

/// Resolved measure indices of a validated navigation marker set.
struct NavigationMarkers {
    /// The `dcalcoda`/`dsalcoda` measure: pass 1 plays through this measure.
    repeat_after: usize,
    /// Where pass 2 restarts: measure 0 for `dcalcoda`, the `segno` measure for `dsalcoda`.
    repeat_from: usize,
    /// The `tocoda` measure: pass 2 plays through this measure before jumping to `coda`.
    to_coda: usize,
    /// The `coda` measure: pass 2 resumes here and continues to the literal end.
    coda: usize,
}

/// The raw (unvalidated) marker measure indices for whichever navigation
/// scheme is in use, plus the fixed `tocoda`/`coda` indices shared by both.
struct RawMarkerIndices {
    /// `dc_indices` (dcalcoda scheme) or `ds_indices` (dsalcoda scheme).
    repeat_after_indices: Vec<usize>,
    /// `[0]` (dcalcoda scheme) or `segno_indices` (dsalcoda scheme).
    repeat_from_indices: Vec<usize>,
    to_coda_indices: Vec<usize>,
    coda_indices: Vec<usize>,
    uses_ds_scheme: bool,
}

/// Determines which navigation scheme (if any) is in use and gathers its raw
/// marker indices. Errors if `dcalcoda` is mixed with `segno`/`dsalcoda`.
fn scheme_indices(score: &Score) -> Result<Option<RawMarkerIndices>, IrrecoverableError> {
    let dc_indices = marker_indices(score, |m| m.dc_al_coda);
    let segno_indices = marker_indices(score, |m| m.segno);
    let ds_indices = marker_indices(score, |m| m.ds_al_coda);
    let to_coda_indices = marker_indices(score, |m| m.to_coda);
    let coda_indices = marker_indices(score, |m| m.coda);

    let uses_dc_scheme = !dc_indices.is_empty();
    let uses_ds_scheme = !segno_indices.is_empty() || !ds_indices.is_empty();

    if !uses_dc_scheme && !uses_ds_scheme && to_coda_indices.is_empty() && coda_indices.is_empty() {
        return Ok(None);
    }

    if uses_dc_scheme && uses_ds_scheme {
        return Err(invalid_at(
            score,
            dc_indices.first().copied().unwrap_or(0),
            "dcalcoda cannot appear together with segno or dsalcoda".to_string(),
        ));
    }

    let (repeat_after_indices, repeat_from_indices) = if uses_ds_scheme {
        (ds_indices, segno_indices)
    } else {
        (dc_indices, vec![0])
    };

    Ok(Some(RawMarkerIndices {
        repeat_after_indices,
        repeat_from_indices,
        to_coda_indices,
        coda_indices,
        uses_ds_scheme,
    }))
}

/// Validates the navigation markers and returns their resolved measure
/// indices.
///
/// Exactly one of two marker schemes is accepted (or neither, for no
/// navigation at all):
/// - `dcalcoda`/`tocoda`/`coda`, each exactly once.
/// - `segno`/`dsalcoda`/`tocoda`/`coda`, each exactly once, with `segno` at
///   or before `dsalcoda`.
///
/// In both schemes `tocoda` must precede `coda`. Mixing `dcalcoda` with
/// `segno`/`dsalcoda` is an error.
fn resolve_marker_indices(score: &Score) -> Result<Option<NavigationMarkers>, IrrecoverableError> {
    let Some(raw) = scheme_indices(score)? else {
        return Ok(None);
    };
    let index_groups = [
        &raw.repeat_after_indices,
        &raw.repeat_from_indices,
        &raw.to_coda_indices,
        &raw.coda_indices,
    ];
    if index_groups.iter().any(|indices| indices.len() != 1) {
        let measure_idx = index_groups
            .iter()
            .find_map(|indices| indices.first())
            .copied()
            .unwrap_or(0);
        let marker_names = if raw.uses_ds_scheme {
            "segno, dsalcoda, tocoda, and coda"
        } else {
            "dcalcoda, tocoda, and coda"
        };
        return Err(invalid_at(
            score,
            measure_idx,
            format!(
                "{marker_names} must each appear exactly once, and they must all appear together"
            ),
        ));
    }

    // Each `.first()` is `Some` here: the length check above already
    // confirmed exactly one element in each of these four vecs.
    let repeat_after = raw
        .repeat_after_indices
        .first()
        .copied()
        .unwrap_or_default();
    let repeat_from = raw.repeat_from_indices.first().copied().unwrap_or_default();
    let to_coda = raw.to_coda_indices.first().copied().unwrap_or_default();
    let coda = raw.coda_indices.first().copied().unwrap_or_default();

    if raw.uses_ds_scheme && repeat_from > repeat_after {
        return Err(invalid_at(
            score,
            repeat_from,
            "segno must occur at or before dsalcoda".to_string(),
        ));
    }

    if to_coda >= coda {
        return Err(invalid_at(
            score,
            to_coda,
            "tocoda must occur before coda".to_string(),
        ));
    }

    Ok(Some(NavigationMarkers {
        repeat_after,
        repeat_from,
        to_coda,
        coda,
    }))
}

/// Rebuilds `score.measures` to reflect D.C. al Coda navigation
/// (`dcalcoda`/`tocoda`/`coda`, or `segno`/`dsalcoda`/`tocoda`/`coda`,
/// markers), so downstream MIDI/WAV generation replays measures in actual
/// playback order instead of written order.
///
/// - No markers present: returns the score unchanged.
/// - `dcalcoda`/`tocoda`/`coda` all present exactly once, with `tocoda`
///   before `coda`: returns a score whose measures are the expanded playback
///   sequence (pass 1 through D.C. al Coda, then pass 2 from the start
///   through To Coda, then Coda through the literal end).
/// - `segno`/`dsalcoda`/`tocoda`/`coda` all present exactly once, with
///   `segno` at or before `dsalcoda` and `tocoda` before `coda`: same as
///   above, but pass 2 restarts from `segno` instead of the start.
/// - Any other combination (partial set, duplicates, mixing `dcalcoda` with
///   `segno`/`dsalcoda`, or `tocoda` at/after `coda`) is an error.
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

    let Some(markers) = resolve_marker_indices(score)? else {
        let origins = (0..score.measures.len()).collect();
        return Ok((score.clone(), origins));
    };

    let last = score.measures.len() - 1;
    let mut idx: Vec<usize> = Vec::new();
    idx.extend(0..=markers.repeat_after);
    idx.extend(markers.repeat_from..=markers.to_coda);
    idx.extend(markers.coda..=last);

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
