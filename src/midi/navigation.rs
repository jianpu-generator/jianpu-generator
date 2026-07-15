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
    /// The `dcalcoda`/`dsalcoda`/`dcalfine`/`dsalfine` measure: pass 1 plays
    /// through this measure.
    repeat_after: usize,
    /// Where pass 2 restarts: measure 0 for `dcalcoda`/`dcalfine`, the
    /// `segno` measure for `dsalcoda`/`dsalfine`.
    repeat_from: usize,
    /// Where and how pass 2 ends.
    terminal: Terminal,
}

/// How the second pass ends: cutting to a `coda` section, or stopping dead
/// at `fine`.
enum Terminal {
    /// The `tocoda` measure: pass 2 plays through this measure before jumping
    /// to `coda`. The `coda` measure: pass 2 resumes here and continues to
    /// the literal end.
    ToCoda { to_coda: usize, coda: usize },
    /// The `fine` measure: pass 2 stops here.
    Fine { fine: usize },
}

/// The raw (unvalidated) marker measure indices for whichever navigation
/// scheme is in use.
struct RawMarkerIndices {
    /// `dc_indices`/`ds_indices` (coda schemes) or `dc_fine_indices`/`ds_fine_indices` (fine schemes).
    repeat_after_indices: Vec<usize>,
    /// `[0]` (dcalcoda/dcalfine schemes) or `segno_indices` (dsalcoda/dsalfine schemes).
    repeat_from_indices: Vec<usize>,
    /// `to_coda_indices`/`coda_indices` (coda schemes) or `fine_indices` (fine schemes).
    terminal_indices: TerminalIndices,
    uses_ds_scheme: bool,
    uses_fine_scheme: bool,
}

enum TerminalIndices {
    ToCoda {
        to_coda_indices: Vec<usize>,
        coda_indices: Vec<usize>,
    },
    Fine {
        fine_indices: Vec<usize>,
    },
}

/// All eight navigation marker measure indices, gathered before any scheme
/// resolution or validation.
struct MarkerIndices {
    dc: Vec<usize>,
    segno: Vec<usize>,
    ds: Vec<usize>,
    to_coda: Vec<usize>,
    coda: Vec<usize>,
    dc_fine: Vec<usize>,
    ds_fine: Vec<usize>,
    fine: Vec<usize>,
}

fn gather_marker_indices(score: &Score) -> MarkerIndices {
    MarkerIndices {
        dc: marker_indices(score, |m| m.dc_al_coda),
        segno: marker_indices(score, |m| m.segno),
        ds: marker_indices(score, |m| m.ds_al_coda),
        to_coda: marker_indices(score, |m| m.to_coda),
        coda: marker_indices(score, |m| m.coda),
        dc_fine: marker_indices(score, |m| m.dc_al_fine),
        ds_fine: marker_indices(score, |m| m.ds_al_fine),
        fine: marker_indices(score, |m| m.fine),
    }
}

fn no_markers_present(m: &MarkerIndices) -> bool {
    m.dc.is_empty()
        && m.segno.is_empty()
        && m.ds.is_empty()
        && m.to_coda.is_empty()
        && m.coda.is_empty()
        && m.dc_fine.is_empty()
        && m.ds_fine.is_empty()
        && m.fine.is_empty()
}

/// Errors if more than one of `dcalcoda`, `segno`+`dsalcoda`, `dcalfine`, or
/// `segno`+`dsalfine` is in use at once: more than one anchor marker present,
/// or a `dcalcoda`/`dcalfine` anchor combined with `segno` (which only ever
/// belongs to a `dsalcoda`/`dsalfine` scheme).
fn check_scheme_mixing(score: &Score, m: &MarkerIndices) -> Result<(), IrrecoverableError> {
    let uses_dc_anchor = !m.dc.is_empty() || !m.dc_fine.is_empty();
    let uses_ds_anchor = !m.ds.is_empty() || !m.ds_fine.is_empty();
    let uses_segno = !m.segno.is_empty();
    let anchor_count = [
        !m.dc.is_empty(),
        !m.ds.is_empty(),
        !m.dc_fine.is_empty(),
        !m.ds_fine.is_empty(),
    ]
    .iter()
    .filter(|used| **used)
    .count();

    if anchor_count <= 1 && !(uses_dc_anchor && (uses_ds_anchor || uses_segno)) {
        return Ok(());
    }

    let measure_idx = [&m.dc, &m.ds, &m.dc_fine, &m.ds_fine, &m.segno]
        .iter()
        .find_map(|indices| indices.first())
        .copied()
        .unwrap_or(0);
    Err(invalid_at(
        score,
        measure_idx,
        "dcalcoda, dsalcoda, dcalfine, and dsalfine cannot appear together in the same score"
            .to_string(),
    ))
}

/// Picks which scheme is in use as `(uses_ds_scheme, uses_fine_scheme)`. When
/// no anchor is present at all (a bare `segno`/`tocoda`/`coda`/`fine` with
/// nothing to anchor it), defaults to whichever scheme the present terminal
/// markers suggest, so later length checks report a clear "must appear
/// together" error.
fn pick_scheme(m: &MarkerIndices) -> (bool, bool) {
    if !m.dc.is_empty() {
        (false, false)
    } else if !m.dc_fine.is_empty() {
        (false, true)
    } else if !m.ds.is_empty() {
        (true, false)
    } else if !m.ds_fine.is_empty() {
        (true, true)
    } else {
        (
            true,
            !m.fine.is_empty() && m.to_coda.is_empty() && m.coda.is_empty(),
        )
    }
}

/// Errors if terminal markers from the *other* kind of scheme are also
/// present (`tocoda`/`coda` alongside a fine scheme, or `fine` alongside a
/// coda scheme), rather than silently ignoring them.
fn check_terminal_contamination(
    score: &Score,
    m: &MarkerIndices,
    uses_fine_scheme: bool,
) -> Result<(), IrrecoverableError> {
    if uses_fine_scheme && (!m.to_coda.is_empty() || !m.coda.is_empty()) {
        let measure_idx = m
            .to_coda
            .first()
            .or_else(|| m.coda.first())
            .copied()
            .unwrap_or(0);
        return Err(invalid_at(
            score,
            measure_idx,
            "tocoda/coda cannot appear together with dcalfine or dsalfine".to_string(),
        ));
    }
    if !uses_fine_scheme && !m.fine.is_empty() {
        return Err(invalid_at(
            score,
            m.fine.first().copied().unwrap_or(0),
            "fine cannot appear together with dcalcoda or dsalcoda".to_string(),
        ));
    }
    Ok(())
}

fn build_raw_marker_indices(
    m: MarkerIndices,
    uses_ds_scheme: bool,
    uses_fine_scheme: bool,
) -> RawMarkerIndices {
    let repeat_after_indices = if uses_fine_scheme {
        if uses_ds_scheme {
            m.ds_fine
        } else {
            m.dc_fine
        }
    } else if uses_ds_scheme {
        m.ds
    } else {
        m.dc
    };
    let repeat_from_indices = if uses_ds_scheme { m.segno } else { vec![0] };
    let terminal_indices = if uses_fine_scheme {
        TerminalIndices::Fine {
            fine_indices: m.fine,
        }
    } else {
        TerminalIndices::ToCoda {
            to_coda_indices: m.to_coda,
            coda_indices: m.coda,
        }
    };

    RawMarkerIndices {
        repeat_after_indices,
        repeat_from_indices,
        terminal_indices,
        uses_ds_scheme,
        uses_fine_scheme,
    }
}

/// Determines which navigation scheme (if any) is in use and gathers its raw
/// marker indices. Errors if more than one of `dcalcoda`, `segno`+`dsalcoda`,
/// `dcalfine`, or `segno`+`dsalfine` is in use at once.
fn scheme_indices(score: &Score) -> Result<Option<RawMarkerIndices>, IrrecoverableError> {
    let m = gather_marker_indices(score);
    if no_markers_present(&m) {
        return Ok(None);
    }
    check_scheme_mixing(score, &m)?;
    let (uses_ds_scheme, uses_fine_scheme) = pick_scheme(&m);
    check_terminal_contamination(score, &m, uses_fine_scheme)?;
    Ok(Some(build_raw_marker_indices(
        m,
        uses_ds_scheme,
        uses_fine_scheme,
    )))
}

/// Errors unless every marker group in the resolved scheme has exactly one
/// occurrence.
fn validate_marker_counts(score: &Score, raw: &RawMarkerIndices) -> Result<(), IrrecoverableError> {
    let terminal_index_groups: Vec<&Vec<usize>> = match &raw.terminal_indices {
        TerminalIndices::ToCoda {
            to_coda_indices,
            coda_indices,
        } => vec![to_coda_indices, coda_indices],
        TerminalIndices::Fine { fine_indices } => vec![fine_indices],
    };
    let index_groups: Vec<&Vec<usize>> = std::iter::once(&raw.repeat_after_indices)
        .chain(std::iter::once(&raw.repeat_from_indices))
        .chain(terminal_index_groups)
        .collect();
    if index_groups.iter().all(|indices| indices.len() == 1) {
        return Ok(());
    }

    let measure_idx = index_groups
        .iter()
        .find_map(|indices| indices.first())
        .copied()
        .unwrap_or(0);
    let marker_names = match (raw.uses_ds_scheme, raw.uses_fine_scheme) {
        (true, true) => "segno, dsalfine, and fine",
        (true, false) => "segno, dsalcoda, tocoda, and coda",
        (false, true) => "dcalfine and fine",
        (false, false) => "dcalcoda, tocoda, and coda",
    };
    Err(invalid_at(
        score,
        measure_idx,
        format!("{marker_names} must each appear exactly once, and they must all appear together"),
    ))
}

/// Errors if `segno` (`repeat_from`) occurs after the `dsalcoda`/`dsalfine`
/// measure (`repeat_after`) it's meant to jump back to.
fn validate_segno_order(
    score: &Score,
    raw: &RawMarkerIndices,
    repeat_after: usize,
    repeat_from: usize,
) -> Result<(), IrrecoverableError> {
    if !raw.uses_ds_scheme || repeat_from <= repeat_after {
        return Ok(());
    }
    let marker = if raw.uses_fine_scheme {
        "dsalfine"
    } else {
        "dsalcoda"
    };
    Err(invalid_at(
        score,
        repeat_from,
        format!("segno must occur at or before {marker}"),
    ))
}

/// Resolves the terminal indices into a validated [`Terminal`]: `tocoda`
/// must precede `coda`; `fine` must occur at or after `repeat_from`.
fn resolve_terminal(
    score: &Score,
    terminal_indices: TerminalIndices,
    repeat_from: usize,
) -> Result<Terminal, IrrecoverableError> {
    match terminal_indices {
        TerminalIndices::ToCoda {
            to_coda_indices,
            coda_indices,
        } => {
            let to_coda = to_coda_indices.first().copied().unwrap_or_default();
            let coda = coda_indices.first().copied().unwrap_or_default();
            if to_coda >= coda {
                return Err(invalid_at(
                    score,
                    to_coda,
                    "tocoda must occur before coda".to_string(),
                ));
            }
            Ok(Terminal::ToCoda { to_coda, coda })
        }
        TerminalIndices::Fine { fine_indices } => {
            let fine = fine_indices.first().copied().unwrap_or_default();
            if fine < repeat_from {
                return Err(invalid_at(
                    score,
                    fine,
                    "fine must occur at or after segno".to_string(),
                ));
            }
            Ok(Terminal::Fine { fine })
        }
    }
}

/// Validates the navigation markers and returns their resolved measure
/// indices.
///
/// Exactly one of four marker schemes is accepted (or neither, for no
/// navigation at all):
/// - `dcalcoda`/`tocoda`/`coda`, each exactly once.
/// - `segno`/`dsalcoda`/`tocoda`/`coda`, each exactly once, with `segno` at
///   or before `dsalcoda`.
/// - `dcalfine`/`fine`, each exactly once.
/// - `segno`/`dsalfine`/`fine`, each exactly once, with `segno` at or before
///   `dsalfine`.
///
/// In the coda schemes, `tocoda` must precede `coda`. In the fine schemes,
/// `fine` must occur at or after `segno` (dsalfine) or at or after measure 0
/// (dcalfine, trivially true). Mixing schemes is an error.
fn resolve_marker_indices(score: &Score) -> Result<Option<NavigationMarkers>, IrrecoverableError> {
    let Some(raw) = scheme_indices(score)? else {
        return Ok(None);
    };
    validate_marker_counts(score, &raw)?;

    // Each `.first()` is `Some` here: `validate_marker_counts` already
    // confirmed exactly one element in each of these vecs.
    let repeat_after = raw
        .repeat_after_indices
        .first()
        .copied()
        .unwrap_or_default();
    let repeat_from = raw.repeat_from_indices.first().copied().unwrap_or_default();
    validate_segno_order(score, &raw, repeat_after, repeat_from)?;
    let terminal = resolve_terminal(score, raw.terminal_indices, repeat_from)?;

    Ok(Some(NavigationMarkers {
        repeat_after,
        repeat_from,
        terminal,
    }))
}

/// Rebuilds `score.measures` to reflect D.C./D.S. al Coda/Fine navigation
/// (`dcalcoda`/`tocoda`/`coda`, `segno`/`dsalcoda`/`tocoda`/`coda`,
/// `dcalfine`/`fine`, or `segno`/`dsalfine`/`fine` markers), so downstream
/// MIDI/WAV generation replays measures in actual playback order instead of
/// written order.
///
/// - No markers present: returns the score unchanged.
/// - `dcalcoda`/`tocoda`/`coda` all present exactly once, with `tocoda`
///   before `coda`: returns a score whose measures are the expanded playback
///   sequence (pass 1 through D.C. al Coda, then pass 2 from the start
///   through To Coda, then Coda through the literal end).
/// - `segno`/`dsalcoda`/`tocoda`/`coda` all present exactly once, with
///   `segno` at or before `dsalcoda` and `tocoda` before `coda`: same as
///   above, but pass 2 restarts from `segno` instead of the start.
/// - `dcalfine`/`fine` all present exactly once: pass 1 through D.C. al
///   Fine, then pass 2 from the start through Fine, then stops (no coda
///   section).
/// - `segno`/`dsalfine`/`fine` all present exactly once, with `segno` at or
///   before `dsalfine` and at or before `fine`: same as above, but pass 2
///   restarts from `segno` instead of the start.
/// - Any other combination (partial set, duplicates, mixing schemes, or
///   `tocoda` at/after `coda`) is an error.
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
    match markers.terminal {
        Terminal::ToCoda { to_coda, coda } => {
            idx.extend(markers.repeat_from..=to_coda);
            idx.extend(coda..=last);
        }
        Terminal::Fine { fine } => {
            idx.extend(markers.repeat_from..=fine);
        }
    }

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
/// `start` to its earliest playback position at or after position 0.
///
/// - If `extend_to_last_occurrence` is `true`, `end` is mapped to its *last*
///   occurrence at or after `start`'s position — the final time `end` is
///   reached in the performance tail starting at `start`. This is what makes
///   "play written measure X through the last written measure" (the web
///   app's "play from current measure", which always passes the score's
///   literal last written measure as `end`) follow every repeat/jump instead
///   of stopping at `end`'s first occurrence.
/// - If `false`, `end` is mapped to its *earliest* occurrence at or after
///   `start`'s position, so that selecting an exact written range (e.g. the
///   web app's "play current measure", where `start == end`) plays only
///   that occurrence instead of overrunning into a later repeat/jump pass —
///   including when `end` is itself a navigation marker measure (`coda`,
///   `dcalcoda`/`dsalcoda`) that also happens to be the score's last
///   written measure, which is the normal case for a `coda` section.
///
/// Falls back to the original written range if either endpoint has no
/// reachable position, or if `start_index > end_index`.
pub fn expand_for_measure_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
) -> Result<(Score, usize, usize), IrrecoverableError> {
    if start_index > end_index {
        return Ok((score.clone(), start_index, end_index));
    }
    let (expanded, origins) = expand_navigation_with_origins(score)?;
    let mapped = earliest_playback_position(&origins, start_index, 0).and_then(|start_pos| {
        let mut end_positions = origins
            .iter()
            .enumerate()
            .skip(start_pos)
            .filter(|(_, &origin)| origin == end_index);
        let end_pos = if extend_to_last_occurrence {
            end_positions.next_back()
        } else {
            end_positions.next()
        };
        end_pos.map(|(pos, _)| (start_pos, pos))
    });
    match mapped {
        Some((start_pos, end_pos)) => Ok((expanded, start_pos, end_pos)),
        None => Ok((score.clone(), start_index, end_index)),
    }
}

#[cfg(test)]
mod tests;
