use crate::ast::parsed::{JianPuPitch, Offset};
use crate::compiler::types::ArcKind;
use crate::compositor::types::{AbsoluteContent, AbsoluteElement, AbsolutePage, TextAnchor};
use crate::render_config::RenderConfig;
use crate::renderer::new_renderer::render_new;
use crate::renderer::new_types::{SvgKind, SvgVariant};

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_measures_per_system: 16,
        lyrics_font_size: 18,
        hide_system_dividers: false,
        section_label_offset: Offset::default(),
    }
}

fn make_page(content: AbsoluteContent) -> AbsolutePage {
    AbsolutePage {
        width_pt: 595.0,
        height_pt: 842.0,
        elements: vec![AbsoluteElement {
            x: 100.0,
            y: 200.0,
            content,
        }],
    }
}

#[test]
fn note_head_produces_text_element() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 0,
        dotted: false,
    });
    let docs = render_new(&[page], &cfg());
    assert_eq!(docs.len(), 1);
    let has_text = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "1"));
    assert!(has_text);
}

#[test]
fn bar_line_produces_vertical_line() {
    let page = make_page(AbsoluteContent::BarLine { height: 60.0 });
    let docs = render_new(&[page], &cfg());
    let has_line = docs[0]
        .elements
        .iter()
        .any(|e| matches!(e.kind, SvgKind::Line { .. }));
    assert!(has_line);
}

#[test]
fn tie_produces_path() {
    let page = make_page(AbsoluteContent::TieOrSlur {
        kind: ArcKind::Slur,
        width: 40.0,
    });
    let docs = render_new(&[page], &cfg());
    let has_path = docs[0]
        .elements
        .iter()
        .any(|e| matches!(e.kind, SvgKind::Path { .. }));
    assert!(has_path);
}

#[test]
fn rest_produces_zero_text() {
    let page = make_page(AbsoluteContent::Rest { dotted: false });
    let docs = render_new(&[page], &cfg());
    let has_zero = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "0"));
    assert!(has_zero);
}

#[test]
fn sharp_accidental_renders_to_the_right_of_note() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Sharp,
        octave: 0,
        dotted: false,
    });
    let note_number_width = cfg().note_number_width as f32;
    let note_x = 100.0_f32;
    let docs = render_new(&[page], &cfg());
    let accidental = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::NoteHeadAccidental));
    let accidental = accidental.expect("accidental element should be present");
    assert!(
        accidental.x > note_x,
        "accidental x ({}) should be to the right of the note x ({})",
        accidental.x,
        note_x
    );
    assert_eq!(accidental.x, note_x + note_number_width * 0.5);
    assert!(
        matches!(&accidental.kind, SvgKind::Text { anchor, .. } if *anchor == TextAnchor::Start),
        "accidental should use TextAnchor::Start"
    );
}

#[test]
fn upper_octave_note_produces_circle() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 1,
        dotted: false,
    });
    let docs = render_new(&[page], &cfg());
    let has_circle = docs[0]
        .elements
        .iter()
        .any(|e| matches!(e.kind, SvgKind::Circle { .. }));
    assert!(has_circle, "upper octave note should produce a dot circle");
}
