use crate::ast::grouped::Score;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use crate::navigation_markers::{self, NavigationMarkerError, RawMarkerIndices, TerminalIndices};

fn invalid_at(score: &Score, measure_idx: usize, detail: String) -> IrrecoverableError {
    let span = score
        .measures
        .get(measure_idx)
        .map(|m| m.source_span)
        .unwrap_or(Span::new(0, 0));
    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(span, detail))
}

fn into_irrecoverable_error(score: &Score, error: NavigationMarkerError) -> IrrecoverableError {
    invalid_at(score, error.measure_idx, error.message)
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

/// Resolves the terminal indices into a validated [`Terminal`]: `tocoda`
/// must precede `coda`; `fine` must occur at or after `repeat_from`.
fn resolve_terminal(
    terminal_indices: TerminalIndices,
    repeat_from: usize,
) -> Result<Terminal, NavigationMarkerError> {
    navigation_markers::validate_terminal_order(&terminal_indices, repeat_from)?;
    Ok(match terminal_indices {
        TerminalIndices::ToCoda {
            to_coda_indices,
            coda_indices,
        } => Terminal::ToCoda {
            to_coda: to_coda_indices.first().copied().unwrap_or_default(),
            coda: coda_indices.first().copied().unwrap_or_default(),
        },
        TerminalIndices::Fine { fine_indices } => Terminal::Fine {
            fine: fine_indices.first().copied().unwrap_or_default(),
        },
    })
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
fn resolve_marker_indices(
    score: &Score,
) -> Result<Option<NavigationMarkers>, NavigationMarkerError> {
    let Some(raw) = navigation_markers::scheme_indices(score)? else {
        return Ok(None);
    };
    navigation_markers::validate_marker_counts(&raw)?;

    // Each `.first()` is `Some` here: `validate_marker_counts` already
    // confirmed exactly one element in each of these vecs.
    let repeat_after = navigation_markers::repeat_after(&raw);
    let repeat_from = navigation_markers::repeat_from(&raw);
    navigation_markers::validate_segno_order(&raw, repeat_after, repeat_from)?;
    let RawMarkerIndices {
        terminal_indices, ..
    } = raw;
    let terminal = resolve_terminal(terminal_indices, repeat_from)?;

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

    let markers =
        resolve_marker_indices(score).map_err(|error| into_irrecoverable_error(score, error))?;
    let Some(markers) = markers else {
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
