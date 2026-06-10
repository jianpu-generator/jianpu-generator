#![allow(clippy::indexing_slicing)]

use crate::error::{JianPuError, Span};

/// Tracks an unfinished `(…` group that continues in a later measure.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GroupParseState {
    pub open: bool,
    pub open_note_count: usize,
}

pub fn validate_group_note_count(count: usize, span: &Span) -> Result<(), JianPuError> {
    if count < 2 {
        return Err(JianPuError::new(
            span.clone(),
            "tie/slur group `(…)` must contain at least 2 notes".to_string(),
        ));
    }
    Ok(())
}

pub fn find_closing_paren(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

pub trait HasGroupDepth {
    fn group_membership(&self) -> u8;
    fn group_continuation(&self) -> u8;
    fn set_group_membership(&mut self, value: u8);
    fn set_group_continuation(&mut self, value: u8);
}

pub fn apply_closed_group_depth<T: HasGroupDepth>(atoms: &mut [T]) {
    let continuation_count = atoms.len().saturating_sub(1);
    for atom in atoms.iter_mut() {
        atom.set_group_membership(atom.group_membership().saturating_add(1));
    }
    for atom in atoms.iter_mut().take(continuation_count) {
        atom.set_group_continuation(atom.group_continuation().saturating_add(1));
    }
}

pub fn apply_open_group_depth<T: HasGroupDepth>(atoms: &mut [T]) {
    for atom in atoms.iter_mut() {
        atom.set_group_membership(atom.group_membership().saturating_add(1));
        atom.set_group_continuation(atom.group_continuation().saturating_add(1));
    }
}

pub fn apply_closing_segment_depth<T: HasGroupDepth>(atoms: &mut [T], group_still_open: bool) {
    for atom in atoms.iter_mut() {
        atom.set_group_membership(atom.group_membership().saturating_add(1));
    }
    let continuation_count = if group_still_open {
        atoms.len()
    } else {
        atoms.len().saturating_sub(1)
    };
    for atom in atoms.iter_mut().take(continuation_count) {
        atom.set_group_continuation(atom.group_continuation().saturating_add(1));
    }
}
