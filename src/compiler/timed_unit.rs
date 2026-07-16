use super::slur_chains::SlurKey;
use crate::ast::grouped::{GroupedChordNote, GroupedNote, GroupedPercussionHit};
use crate::compiler::types::ElementContent;

// ── TimedUnit trait ───────────────────────────────────────────────────────────

pub(super) trait TimedUnit {
    fn duration(&self) -> u32;
    fn dotted(&self) -> bool;
    fn group_membership(&self) -> u8;
    fn group_continuation(&self) -> u8;
    fn slur_close_at(&self) -> Option<u32>;
    fn slur_key(&self) -> SlurKey;
    fn tie_to_next(&self) -> bool;
    fn element_content(&self) -> ElementContent;
}

impl TimedUnit for GroupedNote {
    fn duration(&self) -> u32 {
        self.duration
    }
    fn dotted(&self) -> bool {
        self.dotted
    }
    fn group_membership(&self) -> u8 {
        self.group_membership
    }
    fn group_continuation(&self) -> u8 {
        self.group_continuation
    }
    fn slur_close_at(&self) -> Option<u32> {
        self.slur_group_close_at_duration
    }
    fn slur_key(&self) -> SlurKey {
        SlurKey::Pitch(self.pitch.clone())
    }
    fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
    fn element_content(&self) -> ElementContent {
        ElementContent::NoteHead {
            pitch: self.pitch.clone(),
            accidental: self.accidental.clone(),
            octave: self.octave,
            dotted: self.dotted,
        }
    }
}

impl TimedUnit for GroupedChordNote {
    fn duration(&self) -> u32 {
        self.duration
    }
    fn dotted(&self) -> bool {
        self.dotted
    }
    fn group_membership(&self) -> u8 {
        self.group_membership
    }
    fn group_continuation(&self) -> u8 {
        self.group_continuation
    }
    fn slur_close_at(&self) -> Option<u32> {
        self.slur_group_close_at_duration
    }
    fn slur_key(&self) -> SlurKey {
        SlurKey::from_chord(self)
    }
    fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
    fn element_content(&self) -> ElementContent {
        ElementContent::ChordSymbol(self.format_symbol())
    }
}

impl TimedUnit for GroupedPercussionHit {
    fn duration(&self) -> u32 {
        self.duration
    }
    fn dotted(&self) -> bool {
        self.dotted
    }
    fn group_membership(&self) -> u8 {
        self.group_membership
    }
    fn group_continuation(&self) -> u8 {
        self.group_continuation
    }
    fn slur_close_at(&self) -> Option<u32> {
        self.slur_group_close_at_duration
    }
    fn slur_key(&self) -> SlurKey {
        SlurKey::Rest
    }
    fn tie_to_next(&self) -> bool {
        self.tie_to_next_span.is_some()
    }
    fn element_content(&self) -> ElementContent {
        ElementContent::PercussionHit
    }
}
