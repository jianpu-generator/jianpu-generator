//! Scheme-agnostic gathering and validation of D.C./D.S. al Coda/Fine
//! navigation markers, shared between `midi::navigation` (which additionally
//! expands the score into playback order for MIDI/WAV export) and
//! `grouper::navigation_validation` (which only validates, attaching
//! diagnostics for SVG/PDF rendering).
//!
//! Every function here is pure and score-agnostic beyond gathering marker
//! indices: errors are reported as [`NavigationMarkerError`], a bare measure
//! index plus message, which each call site wraps into its own error type
//! (`IrrecoverableError` or a `RecoverableError` diagnostic).

use crate::ast::grouped::{MultiPartMeasure, Score};

fn marker_indices(score: &Score, marker: impl Fn(&MultiPartMeasure) -> bool) -> Vec<usize> {
    score
        .measures
        .iter()
        .enumerate()
        .filter(|(_, m)| marker(m))
        .map(|(i, _)| i)
        .collect()
}

/// An error found while validating navigation markers: the offending measure
/// index and the diagnostic message to attach to it.
pub(crate) struct NavigationMarkerError {
    pub measure_idx: usize,
    pub message: String,
}

/// All eight navigation marker measure indices, gathered before any scheme
/// resolution or validation.
pub(crate) struct MarkerIndices {
    pub dc: Vec<usize>,
    pub segno: Vec<usize>,
    pub ds: Vec<usize>,
    pub to_coda: Vec<usize>,
    pub coda: Vec<usize>,
    pub dc_fine: Vec<usize>,
    pub ds_fine: Vec<usize>,
    pub fine: Vec<usize>,
}

pub(crate) fn gather_marker_indices(score: &Score) -> MarkerIndices {
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

pub(crate) fn no_markers_present(m: &MarkerIndices) -> bool {
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
pub(crate) fn check_scheme_mixing(m: &MarkerIndices) -> Result<(), NavigationMarkerError> {
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
    Err(NavigationMarkerError {
        measure_idx,
        message:
            "dcalcoda, dsalcoda, dcalfine, and dsalfine cannot appear together in the same score"
                .to_string(),
    })
}

/// Which navigation scheme is in use.
#[derive(Clone, Copy)]
pub(crate) struct Scheme {
    pub uses_ds_scheme: bool,
    pub uses_fine_scheme: bool,
}

/// Picks which scheme is in use. When no anchor is present at all (a bare
/// `segno`/`tocoda`/`coda`/`fine` with nothing to anchor it), defaults to
/// whichever scheme the present terminal markers suggest, so later checks
/// report a clear "must appear together" error.
pub(crate) fn pick_scheme(m: &MarkerIndices) -> Scheme {
    let (uses_ds_scheme, uses_fine_scheme) = if !m.dc.is_empty() {
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
    };
    Scheme {
        uses_ds_scheme,
        uses_fine_scheme,
    }
}

/// Errors if terminal markers from the *other* kind of scheme are also
/// present (`tocoda`/`coda` alongside a fine scheme, or `fine` alongside a
/// coda scheme), rather than silently ignoring them.
pub(crate) fn check_terminal_contamination(
    m: &MarkerIndices,
    uses_fine_scheme: bool,
) -> Result<(), NavigationMarkerError> {
    if uses_fine_scheme && (!m.to_coda.is_empty() || !m.coda.is_empty()) {
        let measure_idx = m
            .to_coda
            .first()
            .or_else(|| m.coda.first())
            .copied()
            .unwrap_or(0);
        return Err(NavigationMarkerError {
            measure_idx,
            message: "tocoda/coda cannot appear together with dcalfine or dsalfine".to_string(),
        });
    }
    if !uses_fine_scheme && !m.fine.is_empty() {
        return Err(NavigationMarkerError {
            measure_idx: m.fine.first().copied().unwrap_or(0),
            message: "fine cannot appear together with dcalcoda or dsalcoda".to_string(),
        });
    }
    Ok(())
}

/// Where and how pass 2 ends, before validation resolves it to concrete
/// indices.
pub(crate) enum TerminalIndices {
    ToCoda {
        to_coda_indices: Vec<usize>,
        coda_indices: Vec<usize>,
    },
    Fine {
        fine_indices: Vec<usize>,
    },
}

/// The raw (unvalidated) marker measure indices for whichever navigation
/// scheme is in use.
pub(crate) struct RawMarkerIndices {
    /// `dc_indices`/`ds_indices` (coda schemes) or `dc_fine_indices`/`ds_fine_indices` (fine schemes).
    pub repeat_after_indices: Vec<usize>,
    /// `[0]` (dcalcoda/dcalfine schemes) or `segno_indices` (dsalcoda/dsalfine schemes).
    pub repeat_from_indices: Vec<usize>,
    /// `to_coda_indices`/`coda_indices` (coda schemes) or `fine_indices` (fine schemes).
    pub terminal_indices: TerminalIndices,
    pub uses_ds_scheme: bool,
    pub uses_fine_scheme: bool,
}

fn build_raw_marker_indices(m: MarkerIndices, scheme: Scheme) -> RawMarkerIndices {
    let Scheme {
        uses_ds_scheme,
        uses_fine_scheme,
    } = scheme;
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
pub(crate) fn scheme_indices(
    score: &Score,
) -> Result<Option<RawMarkerIndices>, NavigationMarkerError> {
    let m = gather_marker_indices(score);
    if no_markers_present(&m) {
        return Ok(None);
    }
    check_scheme_mixing(&m)?;
    let scheme = pick_scheme(&m);
    check_terminal_contamination(&m, scheme.uses_fine_scheme)?;
    Ok(Some(build_raw_marker_indices(m, scheme)))
}

/// Errors unless every marker group in the resolved scheme has exactly one
/// occurrence.
pub(crate) fn validate_marker_counts(raw: &RawMarkerIndices) -> Result<(), NavigationMarkerError> {
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
    Err(NavigationMarkerError {
        measure_idx,
        message: format!(
            "{marker_names} must each appear exactly once, and they must all appear together"
        ),
    })
}

/// The `repeat_after` measure index (the `dcalcoda`/`dsalcoda`/`dcalfine`/
/// `dsalfine` measure): callable only once [`validate_marker_counts`] has
/// confirmed `repeat_after_indices` has exactly one element.
pub(crate) fn repeat_after(raw: &RawMarkerIndices) -> usize {
    raw.repeat_after_indices
        .first()
        .copied()
        .unwrap_or_default()
}

/// The `repeat_from` measure index (measure 0, or the `segno` measure):
/// callable only once [`validate_marker_counts`] has confirmed
/// `repeat_from_indices` has exactly one element.
pub(crate) fn repeat_from(raw: &RawMarkerIndices) -> usize {
    raw.repeat_from_indices.first().copied().unwrap_or_default()
}

/// Errors if `segno` (`repeat_from`) occurs after the `dsalcoda`/`dsalfine`
/// measure (`repeat_after`) it's meant to jump back to.
pub(crate) fn validate_segno_order(
    raw: &RawMarkerIndices,
    repeat_after: usize,
    repeat_from: usize,
) -> Result<(), NavigationMarkerError> {
    if !raw.uses_ds_scheme || repeat_from <= repeat_after {
        return Ok(());
    }
    let marker = if raw.uses_fine_scheme {
        "dsalfine"
    } else {
        "dsalcoda"
    };
    Err(NavigationMarkerError {
        measure_idx: repeat_from,
        message: format!("segno must occur at or before {marker}"),
    })
}

/// Errors if `tocoda` occurs at/after `coda` (coda schemes), or `fine` occurs
/// before `repeat_from` (fine schemes).
pub(crate) fn validate_terminal_order(
    terminal_indices: &TerminalIndices,
    repeat_from: usize,
) -> Result<(), NavigationMarkerError> {
    match terminal_indices {
        TerminalIndices::ToCoda {
            to_coda_indices,
            coda_indices,
        } => {
            let to_coda = to_coda_indices.first().copied().unwrap_or_default();
            let coda = coda_indices.first().copied().unwrap_or_default();
            if to_coda < repeat_from {
                return Err(NavigationMarkerError {
                    measure_idx: to_coda,
                    message: "tocoda must occur at or after segno".to_string(),
                });
            }
            if to_coda >= coda {
                return Err(NavigationMarkerError {
                    measure_idx: to_coda,
                    message: "tocoda must occur before coda".to_string(),
                });
            }
            Ok(())
        }
        TerminalIndices::Fine { fine_indices } => {
            let fine = fine_indices.first().copied().unwrap_or_default();
            if fine < repeat_from {
                return Err(NavigationMarkerError {
                    measure_idx: fine,
                    message: "fine must occur at or after segno".to_string(),
                });
            }
            Ok(())
        }
    }
}
