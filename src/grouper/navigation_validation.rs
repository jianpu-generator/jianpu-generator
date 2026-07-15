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

/// An error found while validating navigation markers: the offending measure
/// index and the diagnostic message to attach to it.
type MarkerError = (usize, String);

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
fn check_scheme_mixing(m: &MarkerIndices) -> Result<(), MarkerError> {
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
    Err((
        measure_idx,
        "dcalcoda, dsalcoda, dcalfine, and dsalfine cannot appear together in the same score"
            .to_string(),
    ))
}

/// Picks which scheme is in use as `(uses_ds_scheme, uses_fine_scheme)`. When
/// no anchor is present at all (a bare `segno`/`tocoda`/`coda`/`fine` with
/// nothing to anchor it), defaults to whichever scheme the present terminal
/// markers suggest, so later checks report a clear "must appear together"
/// error.
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
    m: &MarkerIndices,
    uses_fine_scheme: bool,
) -> Result<(), MarkerError> {
    if uses_fine_scheme && (!m.to_coda.is_empty() || !m.coda.is_empty()) {
        let measure_idx = m
            .to_coda
            .first()
            .or_else(|| m.coda.first())
            .copied()
            .unwrap_or(0);
        return Err((
            measure_idx,
            "tocoda/coda cannot appear together with dcalfine or dsalfine".to_string(),
        ));
    }
    if !uses_fine_scheme && !m.fine.is_empty() {
        return Err((
            m.fine.first().copied().unwrap_or(0),
            "fine cannot appear together with dcalcoda or dsalcoda".to_string(),
        ));
    }
    Ok(())
}

fn repeat_after_indices(
    m: &MarkerIndices,
    uses_ds_scheme: bool,
    uses_fine_scheme: bool,
) -> &Vec<usize> {
    match (uses_ds_scheme, uses_fine_scheme) {
        (false, false) => &m.dc,
        (false, true) => &m.dc_fine,
        (true, false) => &m.ds,
        (true, true) => &m.ds_fine,
    }
}

/// Errors unless every marker group in the resolved scheme has exactly one
/// occurrence.
fn check_marker_counts(
    m: &MarkerIndices,
    uses_ds_scheme: bool,
    uses_fine_scheme: bool,
) -> Result<(), MarkerError> {
    let repeat_after = repeat_after_indices(m, uses_ds_scheme, uses_fine_scheme);
    let repeat_from: &[usize] = if uses_ds_scheme { &m.segno } else { &[0] };
    let terminal_groups: Vec<&Vec<usize>> = if uses_fine_scheme {
        vec![&m.fine]
    } else {
        vec![&m.to_coda, &m.coda]
    };

    let all_present = std::iter::once(repeat_after.as_slice())
        .chain(std::iter::once(repeat_from))
        .chain(terminal_groups.iter().map(|v| v.as_slice()))
        .all(|indices| indices.len() == 1);
    if all_present {
        return Ok(());
    }

    let measure_idx = std::iter::once(repeat_after.as_slice())
        .chain(std::iter::once(repeat_from))
        .chain(terminal_groups.iter().map(|v| v.as_slice()))
        .find_map(|indices| indices.first())
        .copied()
        .unwrap_or(0);
    let marker_names = match (uses_ds_scheme, uses_fine_scheme) {
        (true, true) => "segno, dsalfine, and fine",
        (true, false) => "segno, dsalcoda, tocoda, and coda",
        (false, true) => "dcalfine and fine",
        (false, false) => "dcalcoda, tocoda, and coda",
    };
    Err((
        measure_idx,
        format!("{marker_names} must each appear exactly once, and they must all appear together"),
    ))
}

/// Errors if `segno` (`repeat_from`) occurs after the `dsalcoda`/`dsalfine`
/// measure (`repeat_after`) it's meant to jump back to.
fn check_segno_order(
    uses_ds_scheme: bool,
    uses_fine_scheme: bool,
    repeat_after: usize,
    repeat_from: usize,
) -> Result<(), MarkerError> {
    if !uses_ds_scheme || repeat_from <= repeat_after {
        return Ok(());
    }
    let marker = if uses_fine_scheme {
        "dsalfine"
    } else {
        "dsalcoda"
    };
    Err((
        repeat_from,
        format!("segno must occur at or before {marker}"),
    ))
}

/// Errors if `tocoda` occurs at/after `coda` (coda schemes), or `fine` occurs
/// before `repeat_from` (fine schemes).
fn check_terminal_order(
    m: &MarkerIndices,
    uses_fine_scheme: bool,
    repeat_from: usize,
) -> Result<(), MarkerError> {
    if uses_fine_scheme {
        let fine = m.fine.first().copied().unwrap_or_default();
        if fine < repeat_from {
            return Err((fine, "fine must occur at or after segno".to_string()));
        }
    } else {
        let to_coda = m.to_coda.first().copied().unwrap_or_default();
        let coda = m.coda.first().copied().unwrap_or_default();
        if to_coda >= coda {
            return Err((to_coda, "tocoda must occur before coda".to_string()));
        }
    }
    Ok(())
}

fn validate(score: &Score) -> Result<(), MarkerError> {
    let m = gather_marker_indices(score);
    if no_markers_present(&m) {
        return Ok(());
    }
    check_scheme_mixing(&m)?;
    let (uses_ds_scheme, uses_fine_scheme) = pick_scheme(&m);
    check_terminal_contamination(&m, uses_fine_scheme)?;
    check_marker_counts(&m, uses_ds_scheme, uses_fine_scheme)?;

    let repeat_after = repeat_after_indices(&m, uses_ds_scheme, uses_fine_scheme)
        .first()
        .copied()
        .unwrap_or_default();
    let repeat_from = if uses_ds_scheme { &m.segno } else { &[0][..] }
        .first()
        .copied()
        .unwrap_or_default();
    check_segno_order(uses_ds_scheme, uses_fine_scheme, repeat_after, repeat_from)?;
    check_terminal_order(&m, uses_fine_scheme, repeat_from)
}

/// Validates the D.C./D.S. al Coda/Fine navigation markers (`dcalcoda`,
/// `tocoda`, `coda`, `segno`, `dsalcoda`, `dcalfine`, `fine`, `dsalfine`) and
/// reports a recoverable diagnostic on the offending measure for any invalid
/// combination:
/// - `dcalcoda`/`tocoda`/`coda`, `segno`/`dsalcoda`/`tocoda`/`coda`,
///   `dcalfine`/`fine`, and `segno`/`dsalfine`/`fine` must each appear all
///   together or not at all, at most once each.
/// - `segno` must occur at or before `dsalcoda`/`dsalfine`.
/// - `tocoda` must occur before `coda`.
/// - `fine` must occur at or after `segno` (when using `dsalfine`).
/// - Exactly one of `dcalcoda`, `dsalcoda`, `dcalfine`, `dsalfine` may be
///   used per score; mixing them is an error.
///
/// This mirrors the validation in `midi::navigation`, which only runs when
/// generating MIDI/WAV audio; this pass ensures the same errors surface for
/// SVG/PDF rendering as well.
pub(super) fn validate_navigation_markers(score: &mut Score) {
    if let Err((measure_idx, message)) = validate(score) {
        push_error_at(score, measure_idx, message);
    }
}
