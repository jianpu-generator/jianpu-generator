use super::groups::HasGroupDepth;
use crate::ast::parsed::ScoreEvent;
use crate::error::Spanned;

/// A thin wrapper over `Spanned<ScoreEvent>` that holds mutable group-depth fields so that the
/// generic `HasGroupDepth`-based helpers (`apply_closed_group_depth`, `apply_open_group_depth`)
/// can operate on them.  After depth is applied the wrapper is consumed into the final event list.
pub(super) struct DepthEvent {
    pub(super) spanned: Spanned<ScoreEvent>,
    group_membership: u8,
    group_continuation: u8,
}

impl DepthEvent {
    pub(super) fn new(spanned: Spanned<ScoreEvent>) -> Self {
        Self {
            spanned,
            group_membership: 0,
            group_continuation: 0,
        }
    }

    /// Flush accumulated depth into the underlying `ScoreEvent` and return the `Spanned` value.
    pub(super) fn into_spanned(mut self) -> Spanned<ScoreEvent> {
        apply_depth_to_event(
            &mut self.spanned.value,
            self.group_membership,
            self.group_continuation,
        );
        self.spanned
    }
}

impl HasGroupDepth for DepthEvent {
    fn group_membership(&self) -> u8 {
        self.group_membership
    }

    fn group_continuation(&self) -> u8 {
        self.group_continuation
    }

    fn set_group_membership(&mut self, value: u8) {
        self.group_membership = value;
    }

    fn set_group_continuation(&mut self, value: u8) {
        self.group_continuation = value;
    }
}

/// Push `group_membership` and `group_continuation` depth values into the event's inner struct
/// (only `Note` and `Chord` carry these fields; other variants are unaffected).
fn apply_depth_to_event(event: &mut ScoreEvent, membership: u8, continuation: u8) {
    match event {
        ScoreEvent::Note(n) => {
            n.group_membership = n.group_membership.saturating_add(membership);
            n.group_continuation = n.group_continuation.saturating_add(continuation);
            n.slur = n.group_continuation > 0;
        }
        ScoreEvent::Chord(c) => {
            c.group_membership = c.group_membership.saturating_add(membership);
            c.group_continuation = c.group_continuation.saturating_add(continuation);
            c.slur = c.group_continuation > 0;
        }
        ScoreEvent::Rest(r) => {
            r.group_membership = r.group_membership.saturating_add(membership);
            r.group_continuation = r.group_continuation.saturating_add(continuation);
        }
        _ => {}
    }
}

/// When a slur group's last element is an Extension (i.e., `)` follows a `-`), the arc should
/// end at the extension dash position rather than at the note head. This function scans the
/// group slice (after `apply_closed_group_depth` has run), finds such a pattern, and sets
/// `slur_group_close_at_duration` on the last Note/Chord in the group so the compiler can
/// close the arc at the right column.
pub(super) fn annotate_slur_close_via_extension(group_slice: &mut [DepthEvent]) {
    // Check if the last element in the group is a closing Extension (continuation == 0).
    let last_is_closing_ext = group_slice
        .last()
        .map(|e| matches!(e.spanned.value, ScoreEvent::Extension) && e.group_continuation == 0)
        .unwrap_or(false);

    if !last_is_closing_ext {
        return;
    }

    // Find the last Note or Chord in the group slice — this is the note being extended.
    let last_note_idx = group_slice
        .iter()
        .rposition(|e| matches!(e.spanned.value, ScoreEvent::Note(_) | ScoreEvent::Chord(_)));

    let Some(note_idx) = last_note_idx else {
        return;
    };

    // Count Extension events with continuation > 0 that appear after the note — these are
    // the "continuing" extensions that precede the final closing extension.
    let num_continuing_exts = group_slice
        .get(note_idx + 1..)
        .unwrap_or_default()
        .iter()
        .filter(|e| matches!(e.spanned.value, ScoreEvent::Extension) && e.group_continuation > 0)
        .count() as u32;

    let Some(note_event) = group_slice.get(note_idx) else {
        return;
    };
    let note_initial_duration = match &note_event.spanned.value {
        ScoreEvent::Note(n) => n.duration,
        ScoreEvent::Chord(c) => c.duration,
        _ => return,
    };

    // close_offset = position of the last extension dash relative to the note's start col.
    let close_offset = note_initial_duration + num_continuing_exts * 4;

    let Some(note_event) = group_slice.get_mut(note_idx) else {
        return;
    };
    match &mut note_event.spanned.value {
        ScoreEvent::Note(n) => n.slur_group_close_at_duration = Some(close_offset),
        ScoreEvent::Chord(c) => c.slur_group_close_at_duration = Some(close_offset),
        _ => {}
    }
}
