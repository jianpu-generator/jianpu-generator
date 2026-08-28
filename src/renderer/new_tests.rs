use crate::ast::parsed::{JianPuPitch, Offset};
use crate::compiler::types::ArcKind;
use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, TextAnchor, TextSpan,
};
use crate::render_config::RenderConfig;
use crate::renderer::new_renderer::render_new;
use crate::renderer::new_types::{SvgKind, SvgVariant};

pub(super) fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 16,
        lyrics_font_size: 18,
        notes_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
        measure_number_font_size: 10,
        section_label_font_size: 12,
        part_label_font_size: 12,
        page_number_font_size: 18,
        lyric_click_target_padding_pt: 12,
    }
}

pub(super) fn cfg_with_directive_row_offset(offset: Offset) -> RenderConfig {
    RenderConfig {
        directive_row_offset: offset,
        ..cfg()
    }
}

pub(super) fn bpm_span() -> TextSpan {
    TextSpan {
        content: "120".to_string(),
        bold: false,
        italic: false,
        font_size: 12.0,
    }
}

pub(super) fn make_page(content: AbsoluteContent) -> AbsolutePage {
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
        double_dotted: false,
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
    let page = make_page(AbsoluteContent::Rest {
        dotted: false,
        double_dotted: false,
        implicit_fill: false,
    });
    let docs = render_new(&[page], &cfg());
    let has_zero = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "0"));
    assert!(has_zero);
}

#[test]
fn sharp_accidental_renders_as_part_of_the_note_s_own_text_run() {
    // The accidental is appended directly onto the note digit's own text
    // run (see `render_note_head`) rather than drawn as its own
    // separately-positioned glyph, so it shows up as part of the single
    // `NoteHead`-variant text element's content instead of a distinct
    // element.
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Sharp,
        octave: 0,
        dotted: false,
        double_dotted: false,
    });
    let note_x = 100.0_f32;
    let docs = render_new(&[page], &cfg());
    let note_head = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::NoteHead))
        .expect("note head element should be present");
    assert_eq!(note_head.x, note_x);
    assert!(
        matches!(&note_head.kind, SvgKind::Text { content, anchor, .. }
            if content == "1♯" && *anchor == TextAnchor::Start),
        "note head should render digit and accidental as one flush-left text run"
    );
}

#[test]
fn upper_octave_note_produces_dot_glyph() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 1,
        dotted: false,
        double_dotted: false,
    });
    let docs = render_new(&[page], &cfg());
    let has_dot = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "\u{b7}"));
    assert!(
        has_dot,
        "upper octave note should produce an octave dot glyph"
    );
}
