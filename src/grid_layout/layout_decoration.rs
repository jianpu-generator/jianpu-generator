use crate::compiler::types::{Decoration, MeasureBlock};
use crate::grid_layout::layout::{
    decoration_row_height, header_subtitle_author_row_height, header_title_row_height,
    separator_row_height,
};
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, Header, VAlign};

const DECO_COLS: u32 = 12;

pub(super) fn make_decoration_row(system: &[MeasureBlock], base: f32) -> GridRow {
    let Some(first) = system.first() else {
        return GridRow {
            height_pt: decoration_row_height(base),
            column_count: DECO_COLS,
            elements: vec![],
        };
    };
    let mut elements: Vec<GridElement> = Vec::new();
    let mut dec_col: u32 = 1;

    fn deco_order(d: &Decoration) -> u8 {
        match d {
            Decoration::SectionLabel(_) => 0,
            Decoration::Bpm(_) => 1,
            Decoration::TimeSignature { .. } => 2,
            Decoration::BarNumber(_) => 3,
        }
    }
    let mut sorted_decorations = first.decorations.clone();
    sorted_decorations.sort_by_key(deco_order);

    for dec in &sorted_decorations {
        let col = dec_col;
        dec_col += 1;
        match dec {
            Decoration::Bpm(bpm) => elements.push(GridElement {
                column: col,
                column_span: 1,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::Bpm(*bpm),
            }),
            Decoration::TimeSignature {
                numerator,
                denominator,
            } => elements.push(GridElement {
                column: col,
                column_span: 1,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::TimeSignature {
                    numerator: *numerator,
                    denominator: *denominator,
                },
            }),
            Decoration::SectionLabel(s) => elements.push(GridElement {
                column: col,
                column_span: 1,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::SectionLabel(s.clone()),
            }),
            Decoration::BarNumber(n) => elements.push(GridElement {
                column: col,
                column_span: 1,
                halign: HAlign::Start,
                valign: VAlign::Bottom,
                content: GridContent::BarNumber(*n),
            }),
        }
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
    subtitle_author_elements.push(GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::End,
        valign: VAlign::Center,
        content: GridContent::Text {
            content: header.author.clone(),
            font_size: base * 0.6,
            bold: false,
            italic: false,
        },
    });
    let subtitle_author_row = GridRow {
        height_pt: header_subtitle_author_row_height(base),
        column_count: 1,
        elements: subtitle_author_elements,
    };

    vec![title_row, subtitle_author_row]
}
