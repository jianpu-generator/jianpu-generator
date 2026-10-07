use crate::compiler::types::{ElementContent, MeasureRow, RowKind};

// ── Row classification ────────────────────────────────────────────────────────

pub(crate) fn is_lyric_row(row: &MeasureRow) -> bool {
    matches!(row.kind, RowKind::Lyrics { .. })
}

/// Whether `row` is a lyric row drawn under `target`'s row — a lyric row
/// follows its target in the block, so a row absorbing the lyric rows right
/// after it asks this of each of them.
pub(crate) fn is_lyric_row_of(row: &MeasureRow, target: &MeasureRow) -> bool {
    matches!(&row.kind, RowKind::Lyrics { target: id } if *id == target.id)
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
