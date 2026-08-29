use crate::compiler::types::{ElementContent, MeasureRow};

// ── Row classification ────────────────────────────────────────────────────────

pub(crate) fn is_lyric_row(row: &MeasureRow) -> bool {
    let has_lyric = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::Lyric { .. } | ElementContent::LyricLine { .. }
        )
    });
    let has_note = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::NoteHead { .. } | ElementContent::Rest { .. }
        )
    });
    has_lyric && !has_note
}

pub(crate) fn has_lyrics(row: &MeasureRow) -> bool {
    row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::Lyric { .. } | ElementContent::LyricLine { .. }
        )
    })
}

/// The verse number (0-indexed) an `is_lyric_row` row renders, read from its
/// own content. A part's multiple verses each compile into their own sibling
/// `MeasureRow` (see `ElementContent::Lyric`'s doc comment), so every element
/// in one such row shares the same `verse` field — the first one found is
/// authoritative for the whole row. `None` for a row with no lyric content at
/// all (shouldn't occur for an `is_lyric_row` row in practice).
pub(crate) fn lyric_row_verse(row: &MeasureRow) -> Option<usize> {
    row.elements.iter().find_map(|e| match &e.content {
        ElementContent::Lyric { verse, .. } => Some(*verse),
        ElementContent::LyricLine { verse, .. } => Some(*verse),
        _ => None,
    })
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
