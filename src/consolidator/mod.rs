use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};

pub fn consolidate(mut result: CompileResult) -> CompileResult {
    result.blocks = result.blocks.into_iter().map(consolidate_block).collect();
    result
}

fn consolidate_block(mut block: MeasureBlock) -> MeasureBlock {
    block.rows = consolidate_rows(expand_mixed_rows(block.rows));
    block
}

fn expand_mixed_rows(rows: Vec<MeasureRow>) -> Vec<MeasureRow> {
    rows.into_iter()
        .flat_map(|row| {
            if is_mixed_row(&row) {
                vec![notes_row(&row), lyrics_row(&row)]
            } else {
                vec![row]
            }
        })
        .collect()
}

fn is_mixed_row(row: &MeasureRow) -> bool {
    let has_note_or_rest = row.elements.iter().any(|element| {
        matches!(
            element.content,
            ElementContent::NoteHead { .. } | ElementContent::Rest { .. }
        )
    });
    let has_lyric = row
        .elements
        .iter()
        .any(|element| matches!(element.content, ElementContent::Lyric(_)));
    has_note_or_rest && has_lyric
}

fn notes_row(row: &MeasureRow) -> MeasureRow {
    MeasureRow {
        id: row.id.clone(),
        label: row.label.clone(),
        elements: row
            .elements
            .iter()
            .filter(|element| !matches!(element.content, ElementContent::Lyric(_)))
            .cloned()
            .collect(),
    }
}

fn lyrics_row(row: &MeasureRow) -> MeasureRow {
    let bar_line = row
        .elements
        .iter()
        .find(|element| matches!(element.content, ElementContent::BarLine))
        .cloned();
    let mut elements: Vec<ColumnElement> = row
        .elements
        .iter()
        .filter(|element| matches!(element.content, ElementContent::Lyric(_)))
        .cloned()
        .collect();
    if let Some(bar_line) = bar_line {
        elements.push(bar_line);
    }
    MeasureRow {
        id: RowId(format!("{}-lyrics", row.id.0)),
        label: row.label.clone(),
        elements,
    }
}

fn consolidate_rows(mut rows: Vec<MeasureRow>) -> Vec<MeasureRow> {
    let mut index = 0;
    while index < rows.len() {
        let mut inner = index + 1;
        let mut merged = false;
        while inner < rows.len() {
            let equal = rows
                .get(index)
                .zip(rows.get(inner))
                .is_some_and(|(left, right)| content_equal(left, right));
            if equal {
                let duplicate_label = rows.get(inner).map(|row| row.label.clone());
                if let (Some(row), Some(label)) = (rows.get_mut(index), duplicate_label) {
                    row.label = format!("{} {}", row.label, label);
                }
                rows.remove(inner);
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

fn content_equal(left: &MeasureRow, right: &MeasureRow) -> bool {
    left.elements == right.elements
}

#[cfg(test)]
mod tests;
