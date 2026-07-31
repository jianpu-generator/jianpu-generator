use crate::compiler::types::{Decoration, MeasureBlock};
use crate::grid_layout::layout::{
    block_column_width, decoration_row_height, header_gap_row_height, header_part_list_row_height,
    header_subtitle_author_row_height, header_title_row_height, separator_row_height, LABEL_COLS,
    MUSIC_START_COL,
};
use crate::grid_layout::types::{
    GridContent, GridElement, GridRow, HAlign, Header, MeasureColumnLayout, PartListEntry, VAlign,
};

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

pub(super) fn make_decoration_row(
    system: &[MeasureBlock],
    measure_layout: &[MeasureColumnLayout],
) -> GridRow {
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let music_column_count = MUSIC_START_COL + total_musical_cols;
    let mut elements: Vec<GridElement> = Vec::new();

    // Each block's DirectiveLine is aligned to its own leading barline column
    // (for non-first blocks, this is the same column as the previous block's
    // ending barline) rather than to the first musical column. The barline
    // column is the one place in the grid whose x-position doesn't depend on
    // how many musical columns this system happens to contain (see
    // `ColumnGeometry::x_start`: at `column == LABEL_COLS` the density-scaled
    // term is zero), so anchoring there keeps a measure's label at a
    // consistent position relative to its own bar line across systems.
    //
    // For non-first blocks, only emit a DirectiveLine when there is a label
    // or another directive change.
    let mut leading_barline_col = LABEL_COLS;
    for (index, block) in system.iter().enumerate() {
        if let Some(dec) = block.decorations.first() {
            let should_emit = index == 0 || {
                let Decoration::DirectiveLine {
                    label,
                    key,
                    bpm,
                    time_signature,
                    ..
                } = dec;
                label.is_some() || key.is_some() || bpm.is_some() || time_signature.is_some()
            };
            if should_emit {
                elements.push(directive_line_element(dec, leading_barline_col));
            }
        }
        leading_barline_col += block_column_width(block);
    }

    GridRow {
        height_pt: crate::font_metrics::directive_line_row_height(),
        column_count: music_column_count,
        has_label_region: true,
        measure_layout: measure_layout.to_vec(),
        elements,
    }
}

pub(super) fn make_separator_row() -> GridRow {
    GridRow {
        height_pt: separator_row_height(),
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
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
                    has_label_region: false,
                    measure_layout: vec![],
                    elements: vec![],
                },
                GridRow {
                    height_pt: decoration_row_height(base),
                    column_count: 1,
                    has_label_region: false,
                    measure_layout: vec![],
                    elements: vec![GridElement {
                        column: 0,
                        column_span: 1,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::SequenceLine {
                            entries: entries.clone(),
                            font_size: header.sequence_font_size,
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
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: title.clone(),
                font_size: header.title_font_size,
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
                font_size: header.subtitle_font_size,
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
                font_size: header.author_font_size,
                bold: false,
                italic: false,
            },
        });
    }
    GridRow {
        height_pt: header_subtitle_author_row_height(base),
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
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
            has_label_region: false,
            measure_layout: vec![],
            elements: chunk
                .iter()
                .enumerate()
                .map(|(col_idx, entry)| GridElement {
                    column: col_idx as u32,
                    column_span: 1,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::Text {
                        content: if entry.members.is_empty() {
                            format!("{} \u{2014} {}", entry.abbreviation, entry.display_name)
                        } else {
                            format!(
                                "{} \u{2014} {} ({})",
                                entry.abbreviation,
                                entry.display_name,
                                entry.members.join(",")
                            )
                        },
                        font_size: base * 0.6,
                        bold: false,
                        italic: false,
                    },
                })
                .collect(),
        })
        .collect()
}
