use crate::compiler::types::{Decoration, MeasureBlock};
use crate::grid_layout::layout::{
    block_column_width, decoration_row_height, header_subtitle_author_row_height,
    header_title_row_height, separator_row_height, LABEL_COLS,
};
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, Header, VAlign};

const DECO_COLS: u32 = 12;

fn deco_order(d: &Decoration) -> u8 {
    match d {
        Decoration::SectionLabel(_) => 0,
        Decoration::Bpm(_) => 1,
        Decoration::TimeSignature { .. } => 2,
        Decoration::BarNumber(_) => 3,
    }
}

fn make_deco_element(dec: &Decoration, col: u32) -> GridElement {
    match dec {
        Decoration::Bpm(bpm) => GridElement {
            column: col,
            column_span: 1,
            halign: HAlign::Start,
            valign: VAlign::Center,
            content: GridContent::Bpm(*bpm),
        },
        Decoration::TimeSignature {
            numerator,
            denominator,
        } => GridElement {
            column: col,
            column_span: 1,
            halign: HAlign::Start,
            valign: VAlign::Center,
            content: GridContent::TimeSignature {
                numerator: *numerator,
                denominator: *denominator,
            },
        },
        Decoration::SectionLabel(s) => GridElement {
            column: col,
            column_span: 1,
            halign: HAlign::Start,
            valign: VAlign::Center,
            content: GridContent::SectionLabel(s.clone()),
        },
        Decoration::BarNumber(n) => GridElement {
            column: col,
            column_span: 1,
            halign: HAlign::Start,
            valign: VAlign::Bottom,
            content: GridContent::BarNumber(*n),
        },
    }
}

pub(super) fn make_decoration_row(system: &[MeasureBlock], base: f32) -> GridRow {
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let music_column_count = LABEL_COLS + total_musical_cols;
    let mut elements: Vec<GridElement> = Vec::new();

    // First block: sequential columns starting at 1 (preserves original h-stacking and spacing).
    if let Some(first) = system.first() {
        let mut sorted = first.decorations.clone();
        sorted.sort_by_key(deco_order);
        let mut dec_col: u32 = 1;
        for dec in &sorted {
            elements.push(make_deco_element(dec, dec_col));
            dec_col += 1;
        }
    }

    // Non-first blocks: only SectionLabels, mapped proportionally into the DECO_COLS space
    // so they appear above the correct measure without disturbing h-stacking of the first block.
    let mut measure_music_col = LABEL_COLS;
    for (index, block) in system.iter().enumerate() {
        if index > 0 {
            for dec in &block.decorations {
                if let Decoration::SectionLabel(_) = dec {
                    let deco_col = (measure_music_col as f32 * DECO_COLS as f32
                        / music_column_count as f32)
                        .round() as u32;
                    elements.push(make_deco_element(dec, deco_col.clamp(1, DECO_COLS - 1)));
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

pub(crate) fn make_header_rows(header: &Header, base: f32) -> Vec<GridRow> {
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

    vec![title_row, subtitle_author_row]
}
