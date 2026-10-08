use crate::ast::parsed::PartKind;
use crate::compiler::types::{ElementContent, LyricLink, MeasureRow};

// ── Row classification ────────────────────────────────────────────────────────

pub(crate) fn is_lyric_row(row: &MeasureRow) -> bool {
    matches!(row.kind, PartKind::Lyrics { .. })
}

/// Whether `row` is a lyric row drawn under `target`'s row, per the score's
/// `lyric_links` — a lyric row follows its target in the block, so a row
/// absorbing the lyric rows right after it asks this of each of them.
pub(crate) fn sings_along_to(links: &[LyricLink], row: &MeasureRow, target: &MeasureRow) -> bool {
    is_lyric_row(row)
        && links
            .iter()
            .any(|link| link.lyric_row_id == row.id && link.target_row_id == target.id)
}

pub(crate) fn is_chord_only_row(row: &MeasureRow) -> bool {
    if is_lyric_row(row) {
        return false;
    }
    let has_note = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::NoteHead { .. }
                | ElementContent::Rest { .. }
                | ElementContent::PercussionHit
        )
    });
    !has_note
        && row
            .elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::ChordSymbol { .. }))
}
