use crate::compiler::types::{Decoration, MeasureBlock};
use crate::grid_layout::layout::{
    block_column_width, decoration_row_height, header_part_list_row_height,
    header_subtitle_author_row_height, header_title_row_height, separator_row_height, LABEL_COLS,
};
use crate::grid_layout::types::{
    GridContent, GridElement, GridRow, HAlign, Header, PartListEntry, VAlign,
};

const DECO_COLS: u32 = 12;

fn directive_line_element(dec: &Decoration, col: u32) -> GridElement {
    let Decoration::DirectiveLine {
        label,
        bar_number,
        key,
        bpm,
        time_signature,
    } = dec;
    GridElement {
        column: col,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Bottom,
        content: GridContent::DirectiveLine {
            label: label.clone(),
            bar_number: *bar_number,
            key: key.clone(),
            bpm: *bpm,
            time_signature: *time_signature,
        },
    }
}

pub(super) fn make_decoration_row(system: &[MeasureBlock], base: f32) -> GridRow {
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let music_column_count = LABEL_COLS + total_musical_cols;
    let mut elements: Vec<GridElement> = Vec::new();

    // First block: one DirectiveLine element at column 1.
    if let Some(first) = system.first() {
        if let Some(dec) = first.decorations.first() {
            elements.push(directive_line_element(dec, 1));
        }
    }

    // Non-first blocks: only emit a DirectiveLine when there is a label,
    // placed proportionally so it appears above the correct measure.
    let mut measure_music_col = LABEL_COLS;
    for (index, block) in system.iter().enumerate() {
        if index > 0 {
            if let Some(Decoration::DirectiveLine { label: Some(_), .. }) =
                block.decorations.first()
            {
                let deco_col = (measure_music_col as f32 * DECO_COLS as f32
                    / music_column_count as f32)
                    .round() as u32;
                if let Some(dec) = block.decorations.first() {
                    elements.push(directive_line_element(
                        dec,
                        deco_col.clamp(1, DECO_COLS - 1),
                    ));
                }
            }
        }
        measure_music_col += block_column_width(block);
    }

    GridRow {
        height_pt: decoration_row_height(base),
        column_count: DECO_COLS,
        elements,
    }
}

pub(super) fn make_separator_row() -> GridRow {
    GridRow {
        height_pt: separator_row_height(),
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Start,
            valign: VAlign::Center,
            content: GridContent::HorizontalLine,
        }],
    }
}

pub(crate) fn make_header_rows(
    header: &Header,
    base: f32,
    include_part_list: bool,
) -> Vec<GridRow> {
    let title_row = GridRow {
        height_pt: header_title_row_height(base),
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: header.title.clone(),
                font_size: base * 1.5,
                bold: false,
                italic: false,
            },
        }],
    };

    let mut subtitle_author_elements: Vec<GridElement> = Vec::new();
    if let Some(subtitle) = &header.subtitle {
        subtitle_author_elements.push(GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: subtitle.clone(),
                font_size: base * 0.8,
                bold: false,
                italic: true,
            },
        });
    }
    if let Some(author) = &header.author {
        subtitle_author_elements.push(GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::End,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: author.clone(),
                font_size: base * 0.6,
                bold: false,
                italic: false,
            },
        });
    }
    let subtitle_author_row = GridRow {
        height_pt: header_subtitle_author_row_height(base),
        column_count: 1,
        elements: subtitle_author_elements,
    };

    let part_list_rows: Vec<GridRow> = if include_part_list {
        let entries: Vec<&PartListEntry> = header
            .part_list
            .iter()
            .filter(|entry| entry.abbreviation != entry.display_name)
            .collect();
        make_part_list_rows(&entries, base, header.parts_list_columns)
    } else {
        vec![]
    };

    std::iter::once(title_row)
        .chain(std::iter::once(subtitle_author_row))
        .chain(part_list_rows)
        .collect()
}

fn make_part_list_rows(entries: &[&PartListEntry], base: f32, columns: u32) -> Vec<GridRow> {
    entries
        .chunks(columns as usize)
        .map(|chunk| GridRow {
            height_pt: header_part_list_row_height(base),
            column_count: columns,
            elements: chunk
                .iter()
                .enumerate()
                .map(|(col_idx, entry)| GridElement {
                    column: col_idx as u32,
                    column_span: 1,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::Text {
                        content: format!("{} \u{2014} {}", entry.abbreviation, entry.display_name),
                        font_size: base * 0.6,
                        bold: false,
                        italic: false,
                    },
                })
                .collect(),
        })
        .collect()
}
