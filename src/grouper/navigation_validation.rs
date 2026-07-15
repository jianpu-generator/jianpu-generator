use crate::ast::grouped::Score;
use crate::error::{Diagnostic, RecoverableError};
use crate::navigation_markers::{self, NavigationMarkerError};

fn push_error_at(score: &mut Score, error: NavigationMarkerError) {
    if let Some(measure) = score.measures.get_mut(error.measure_idx) {
        let span = measure.source_span;
        measure
            .diagnostics
            .push(Diagnostic::Error(RecoverableError::general(
                span,
                error.message,
            )));
    }
}

fn validate(score: &Score) -> Result<(), NavigationMarkerError> {
    let Some(raw) = navigation_markers::scheme_indices(score)? else {
        return Ok(());
    };
    navigation_markers::validate_marker_counts(&raw)?;

    let repeat_after = navigation_markers::repeat_after(&raw);
    let repeat_from = navigation_markers::repeat_from(&raw);
    navigation_markers::validate_segno_order(&raw, repeat_after, repeat_from)?;
    navigation_markers::validate_terminal_order(&raw.terminal_indices, repeat_from)
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
    if let Err(error) = validate(score) {
        push_error_at(score, error);
    }
}
