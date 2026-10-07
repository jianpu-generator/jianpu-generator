use crate::compiler::types::{CompileResult, MeasureBlock, MeasureRow};

pub fn consolidate(mut result: CompileResult) -> CompileResult {
    result.blocks = result.blocks.into_iter().map(consolidate_block).collect();
    result
}

fn consolidate_block(mut block: MeasureBlock) -> MeasureBlock {
    let merge_across_parts = block.merge_duplicate_measures_across_parts;
    block.rows = consolidate_rows(block.rows, merge_across_parts);
    block
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
