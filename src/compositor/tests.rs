use super::*;
use crate::ast::parsed::JianPuPitch;
use crate::compiler::types::{
    ColumnElement, Decoration, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::layout::new_layout::layout_new;
use crate::layout::new_types::Header;
use crate::render_config::RenderConfig;

fn make_note_block(row_id: &str) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: 3,
                    content: ElementContent::BarLine,
                },
            ],
        }],
        decorations: vec![],
    }
}

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_columns: 16,
    }
}

#[test]
fn compose_produces_one_page_per_input() {
    let blocks = vec![make_note_block("S")];
    let hdr = Header {
        title: "T".to_string(),
        subtitle: None,
        author: "A".to_string(),
    };
    let pages = layout_new(&blocks, &cfg(), &hdr, 595.0, 842.0);
    let abs_pages = compose(&pages, &cfg());
    assert_eq!(abs_pages.len(), 1);
}

#[test]
fn note_head_has_positive_x_y() {
    let blocks = vec![make_note_block("S")];
    let hdr = Header {
        title: "T".to_string(),
        subtitle: None,
        author: "A".to_string(),
    };
    let pages = layout_new(&blocks, &cfg(), &hdr, 595.0, 842.0);
    let abs = compose(&pages, &cfg());
    let note = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }))
        .expect("should have a NoteHead");
    assert!(note.x > 0.0, "x={}", note.x);
    assert!(note.y > 0.0, "y={}", note.y);
}

#[test]
fn non_empty_page_has_title_text() {
    let blocks = vec![make_note_block("S")];
    let hdr = Header {
        title: "My Song".to_string(),
        subtitle: None,
        author: "Me".to_string(),
    };
    let pages = layout_new(&blocks, &cfg(), &hdr, 595.0, 842.0);
    let abs = compose(&pages, &cfg());
    let has_title = abs[0].elements.iter().any(
        |e| matches!(&e.content, AbsoluteContent::Text { content, .. } if content == "My Song"),
    );
    assert!(has_title, "should have title text element");
}

#[test]
fn bar_line_has_positive_height() {
    let blocks = vec![make_note_block("S")];
    let hdr = Header {
        title: "T".to_string(),
        subtitle: None,
        author: "A".to_string(),
    };
    let pages = layout_new(&blocks, &cfg(), &hdr, 595.0, 842.0);
    let abs = compose(&pages, &cfg());
    let bar = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::BarLine { .. }))
        .expect("should have a BarLine");
    if let AbsoluteContent::BarLine { height } = bar.content {
        assert!(
            height > 0.0,
            "bar height should be positive, got {}",
            height
        );
    }
}

#[test]
fn compose_empty_pages_returns_empty_vec() {
    let abs = compose(&[], &cfg());
    assert!(abs.is_empty());
}

#[test]
fn with_bpm_decoration_emits_bpm_text() {
    let block = MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("S".to_string()),
            label: "S".to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: 3,
                    content: ElementContent::BarLine,
                },
            ],
        }],
        decorations: vec![Decoration::Bpm(120)],
    };
    let hdr = Header {
        title: "T".to_string(),
        subtitle: None,
        author: "A".to_string(),
    };
    let pages = layout_new(&[block], &cfg(), &hdr, 595.0, 842.0);
    let abs = compose(&pages, &cfg());
    let has_bpm = abs[0].elements.iter().any(
        |e| matches!(&e.content, AbsoluteContent::Text { content, .. } if content.contains("120")),
    );
    assert!(has_bpm, "should emit BPM text element");
}
