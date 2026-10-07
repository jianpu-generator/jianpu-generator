use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use std::collections::HashSet;

pub fn consolidate(result: CompileResult) -> CompileResult {
    consolidate_with_lyrics_only(result, &[])
}

/// Like [`consolidate`], but each part named in `lyrics_only_parts` keeps only
/// its lyric rows: its notes row is dropped (and with it the part's slur and
/// tuplet arcs), while every lyric element stays at its original column. A
/// lyrics-only part's measure with no lyrics has no row at all, which
/// `grid_layout::layout_systems`'s union-of-parts packing pads with a blank
/// lyric row whenever a neighbouring measure in the system has lyrics.
pub fn consolidate_with_lyrics_only(
    mut result: CompileResult,
    lyrics_only_parts: &[String],
) -> CompileResult {
    let lyrics_only_indices: HashSet<usize> = result
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .filter(|row| lyrics_only_parts.contains(&row.label))
        .map(|row| row.source_part_index)
        .collect();
    result
        .slur_spans
        .retain(|span| !lyrics_only_indices.contains(&span.part_index));
    result
        .tuplet_spans
        .retain(|span| !lyrics_only_indices.contains(&span.part_index));
    result.blocks = result
        .blocks
        .into_iter()
        .map(|block| consolidate_block(block, lyrics_only_parts))
        .collect();
    result
}

fn consolidate_block(mut block: MeasureBlock, lyrics_only_parts: &[String]) -> MeasureBlock {
    let merge_across_parts = block.merge_duplicate_measures_across_parts;
    block.rows = consolidate_rows(
        expand_mixed_rows(block.rows, lyrics_only_parts),
        merge_across_parts,
    );
    block
}

fn expand_mixed_rows(rows: Vec<MeasureRow>, lyrics_only_parts: &[String]) -> Vec<MeasureRow> {
    rows.into_iter()
        .flat_map(|row| {
            if lyrics_only_parts.contains(&row.label) {
                return lyrics_rows(&row);
            }
            let has_note_or_rest = row_has_note_or_rest(&row);
            let verse_count = lyric_verse_count(&row);
            if has_note_or_rest && verse_count > 0 {
                // A notes-with-lyrics row: split into the notes row plus one row per verse.
                let mut expanded = vec![notes_row(&row)];
                expanded.extend(lyrics_rows(&row));
                expanded
            } else if !has_note_or_rest && verse_count > 1 {
                // No paired notes but multiple verses — one row per verse.
                lyrics_rows(&row)
            } else {
                vec![row]
            }
        })
        .collect()
}

fn row_has_note_or_rest(row: &MeasureRow) -> bool {
    row.elements.iter().any(|element| {
        matches!(
            element.content,
            ElementContent::NoteHead { .. } | ElementContent::Rest { .. }
        )
    })
}

fn lyric_verse_count(row: &MeasureRow) -> usize {
    row.elements
        .iter()
        .filter_map(|element| match &element.content {
            ElementContent::Lyric { verse, .. } | ElementContent::LyricLine { verse, .. } => {
                Some(*verse + 1)
            }
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

fn notes_row(row: &MeasureRow) -> MeasureRow {
    MeasureRow {
        id: row.id.clone(),
        label: row.label.clone(),
        elements: row
            .elements
            .iter()
            .filter(|element| {
                !matches!(
                    element.content,
                    ElementContent::Lyric { .. } | ElementContent::LyricLine { .. }
                )
            })
            .cloned()
            .collect(),
        source_part_index: row.source_part_index,
        absorbed_rows: row.absorbed_rows.clone(),
    }
}

/// The lyric part abbreviation of `verse` in `row`, falling back to the row's
/// own label when no syllable of that verse carries one.
fn verse_label(row: &MeasureRow, verse: usize) -> String {
    row.elements
        .iter()
        .find_map(|element| match &element.content {
            ElementContent::Lyric {
                verse: v,
                verse_label,
                ..
            } if *v == verse && !verse_label.is_empty() => Some(verse_label.clone()),
            _ => None,
        })
        .unwrap_or_else(|| row.label.clone())
}

/// Splits a mixed row's lyric elements into one `MeasureRow` per verse, in verse
/// order, each with a distinct `RowId` so a verse-count change between two
/// measures of the same part is a row-structure change (see `pack_into_systems`
/// in `grid_layout::layout`), forcing a new system rather than silently
/// dropping or misaligning verses.
fn lyrics_rows(row: &MeasureRow) -> Vec<MeasureRow> {
    let bar_line = row
        .elements
        .iter()
        .find(|element| matches!(element.content, ElementContent::BarLine))
        .cloned();
    let verse_count = lyric_verse_count(row);
    (0..verse_count)
        .map(|verse| {
            let mut elements: Vec<ColumnElement> = row
                .elements
                .iter()
                .filter(|element| {
                    matches!(
                        &element.content,
                        ElementContent::Lyric { verse: v, .. }
                            | ElementContent::LyricLine { verse: v, .. }
                        if *v == verse
                    )
                })
                .cloned()
                .collect();
            if let Some(bar_line) = &bar_line {
                elements.push(bar_line.clone());
            }
            MeasureRow {
                id: RowId(format!("{}-lyrics-{verse}", row.id.0)),
                label: verse_label(row, verse),
                elements,
                source_part_index: row.source_part_index,
                absorbed_rows: row.absorbed_rows.clone(),
            }
        })
        .collect()
}

/// Merges rows with identical content within a single measure block, but
/// deliberately leaves `label` untouched — it stays each
/// row's own, per-part identity (as compiled) regardless of merging. A block
/// is consolidated in isolation, before systems (and thus the multi-measure
/// context a display label needs) exist: whether a coincidentally-identical
/// row is "genuinely" merged for the whole system, or only matched by
/// per-measure accident, can only be decided once `grid_layout::layout_systems`
/// knows every measure in the system (see its `resolve_label`, which folds
/// each row's own identity with its still-genuinely-absorbed rows once that
/// context exists — this function only decides *which* rows to fold content
/// into, not what to call the result).
fn consolidate_rows(mut rows: Vec<MeasureRow>, merge_across_parts: bool) -> Vec<MeasureRow> {
    let mut index = 0;
    while index < rows.len() {
        let mut inner = index + 1;
        let mut merged = false;
        while inner < rows.len() {
            let equal = rows
                .get(index)
                .zip(rows.get(inner))
                .is_some_and(|(left, right)| {
                    let is_cross_part = left.source_part_index != right.source_part_index;
                    (merge_across_parts || !is_cross_part) && content_equal(left, right)
                });
            if equal {
                let removed = rows.remove(inner);
                if let Some(row) = rows.get_mut(index) {
                    // `removed` disappears from `rows` entirely, so its own
                    // content (and anything already merged into it) has to be
                    // recorded here — otherwise a later pass has no way to
                    // tell this part's content apart from one that's
                    // genuinely absent, or to re-render it on its own (see
                    // `MeasureRow::absorbed_rows`).
                    let mut removed_own = removed;
                    let nested = std::mem::take(&mut removed_own.absorbed_rows);
                    row.absorbed_rows.push(removed_own);
                    row.absorbed_rows.extend(nested);
                }
                merged = true;
                break;
            }
            inner += 1;
        }
        if !merged {
            index += 1;
        }
    }
    rows
}

/// Elements are compared by `column`/`content` only, ignoring `note_id`: it's
/// a per-part running counter (not reset per measure), so two parts with
/// visually identical notes can carry different `note_id`s once their event
/// counts have diverged in an earlier measure.
fn content_equal(left: &MeasureRow, right: &MeasureRow) -> bool {
    left.elements.len() == right.elements.len()
        && left
            .elements
            .iter()
            .zip(right.elements.iter())
            .all(|(l, r)| l.column == r.column && l.content == r.content)
}

#[cfg(test)]
mod tests;
