use crate::compiler::types::{Decoration, MeasureBlock};
use crate::grid_layout::layout::{
    block_column_width, decoration_row_height, header_gap_row_height, header_part_list_row_height,
    header_subtitle_author_row_height, header_title_row_height, separator_row_height, LABEL_COLS,
};
use crate::grid_layout::types::{
    GridContent, GridElement, GridRow, HAlign, Header, PartListEntry, VAlign,
};

fn directive_line_element(dec: &Decoration, col: u32) -> GridElement {
    let Decoration::DirectiveLine {
        label,
        bar_number,
        key,
        bpm,
        time_signature,
        dc_al_coda,
        to_coda,
        coda,
        segno,
        ds_al_coda,
        dc_al_fine,
        fine,
        ds_al_fine,
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
            dc_al_coda: *dc_al_coda,
            to_coda: *to_coda,
            coda: *coda,
            segno: *segno,
            ds_al_coda: *ds_al_coda,
            dc_al_fine: *dc_al_fine,
            fine: *fine,
            ds_al_fine: *ds_al_fine,
        },
    }
}

fn decoration_has_navigation_marker(dec: &Decoration) -> bool {
    let Decoration::DirectiveLine {
        dc_al_coda,
        to_coda,
        coda,
        segno,
        ds_al_coda,
        dc_al_fine,
        fine,
        ds_al_fine,
        ..
    } = dec;
    *dc_al_coda || *to_coda || *coda || *segno || *ds_al_coda || *dc_al_fine || *fine || *ds_al_fine
}

pub(super) fn make_decoration_row(system: &[MeasureBlock], base: f32) -> GridRow {
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let music_column_count = LABEL_COLS + total_musical_cols;
    let mut elements: Vec<GridElement> = Vec::new();

    // First block: one DirectiveLine element aligned to the left edge of the first measure.
    if let Some(first) = system.first() {
        if let Some(dec) = first.decorations.first() {
            elements.push(directive_line_element(dec, LABEL_COLS));
        }
    }

    // Non-first blocks: only emit a DirectiveLine when there is a label or a
    // navigation marker, aligned to the left edge of the measure it belongs
    // to. This uses the same column grid as the music rows so the label
    // lines up exactly with the measure's bar line.
    let mut measure_music_col = LABEL_COLS;
    for (index, block) in system.iter().enumerate() {
        if index > 0 {
            if let Some(dec) = block.decorations.first() {
                let has_label = matches!(dec, Decoration::DirectiveLine { label: Some(_), .. });
                if has_label || decoration_has_navigation_marker(dec) {
                    elements.push(directive_line_element(dec, measure_music_col));
                }
            }
        }
        measure_music_col += block_column_width(block);
    }

    GridRow {
        height_pt: decoration_row_height(base),
        column_count: music_column_count,
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

fn make_sequence_rows(header: &Header, base: f32, include_part_list: bool) -> Vec<GridRow> {
    header
        .sequence
        .as_ref()
        .filter(|_| include_part_list)
        .map(|entries| {
            vec![
                GridRow {
                    height_pt: header_gap_row_height(base),
                    column_count: 1,
                    elements: vec![],
                },
                GridRow {
                    height_pt: decoration_row_height(base),
                    column_count: 1,
                    elements: vec![GridElement {
                        column: 0,
                        column_span: 1,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::SequenceLine {
                            entries: entries.clone(),
                        },
                    }],
                },
            ]
        })
        .unwrap_or_default()
}

fn make_title_row(header: &Header, base: f32) -> Option<GridRow> {
    header.title.as_ref().map(|title| GridRow {
        height_pt: header_title_row_height(base),
        column_count: 1,
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: title.clone(),
                font_size: base * 1.5,
                bold: false,
                italic: false,
            },
        }],
    })
}

fn make_subtitle_author_row(header: &Header, base: f32) -> GridRow {
    let mut elements: Vec<GridElement> = Vec::new();
    if let Some(subtitle) = &header.subtitle {
        elements.push(GridElement {
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
        elements.push(GridElement {
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
    GridRow {
        height_pt: header_subtitle_author_row_height(base),
        column_count: 1,
        elements,
    }
}

pub(crate) fn make_header_rows(
    header: &Header,
    base: f32,
    include_part_list: bool,
) -> Vec<GridRow> {
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

    make_title_row(header, base)
        .into_iter()
        .chain(std::iter::once(make_subtitle_author_row(header, base)))
        .chain(part_list_rows)
        .chain(make_sequence_rows(header, base, include_part_list))
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
