//! A note head, a rest, a percussion hit, a chord symbol, a Latin lyric
//! syllable, and a CJK lyric syllable can all share the same column. Every
//! one of these six flush-left content types now draws `TextAnchor::Start`
//! at exactly the anchor `resolve()` computes (`x_start(column) +
//! GLYPH_LEFT_PADDING - bearing(leading_char)` — see
//! `resolve::flush_left_padding`), so every ink-left-edge has the same
//! shape: `anchor + bearing(leading_char)`. This test computes each
//! element's predicted ink-left-edge that way and asserts they coincide for
//! elements sharing one column — the perfect left-edge alignment this
//! anchor-mode standardization exists to make achievable exactly, not just
//! approximately.

use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{resolve, LyricFontSizes};
use crate::font_metrics::{cjk_glyph_left_bearing, monospace_glyph_left_bearing};
use crate::grid_layout::types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};

const NOTE_NUMBER_WIDTH: f32 = 12.0;
const NOTES_FONT_SIZE: f32 = 12.0;
const CHORDS_FONT_SIZE: f32 = 12.0;
const LYRIC_FONT_SIZES: LyricFontSizes = LyricFontSizes {
    base: 14.4,
    cjk: 17.28,
};

fn single_row_page(element: GridElement) -> GridPage {
    GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![element],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        bar_number_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    }
}

/// Resolves a single element at column 0 and returns its `AbsoluteElement`'s
/// `x` (the anchor the renderer draws `TextAnchor::Start` at).
fn resolve_anchor_x(content: GridContent) -> f32 {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content,
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        NOTE_NUMBER_WIDTH,
        40.0,
        LYRIC_FONT_SIZES,
        NOTES_FONT_SIZE,
        CHORDS_FONT_SIZE,
    )
    .unwrap();
    abs[0]
        .elements
        .iter()
        .find(|e| {
            !matches!(
                e.content,
                AbsoluteContent::Underline { .. }
                    | AbsoluteContent::TieOrSlur { .. }
                    | AbsoluteContent::TupletBracket { .. }
            )
        })
        .expect("should have exactly one non-span-marking element")
        .x
}

/// Ink-left-edge of a `TextAnchor::Start`-drawn glyph: ink starts `bearing`
/// to the right of the anchor. The same shape for every flush-left content
/// type now that they all share one anchor mode — only the font face
/// (monospace vs. CJK) and font size behind `bearing` differ per type.
fn ink_left_edge(anchor: f32, bearing: f32) -> f32 {
    anchor + bearing
}

#[test]
fn note_head_and_latin_lyric_syllable_have_the_same_ink_left_edge_in_one_column() {
    let note_anchor = resolve_anchor_x(GridContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
    });
    let lyric_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "la".to_string(),
        source_part_index: 0,
        note_id: 0,
        verse: 0,
    });

    let note_ink_left = ink_left_edge(
        note_anchor,
        monospace_glyph_left_bearing('1', NOTES_FONT_SIZE),
    );
    let lyric_ink_left = ink_left_edge(
        lyric_anchor,
        cjk_glyph_left_bearing('l', LYRIC_FONT_SIZES.base),
    );

    assert!(
        (note_ink_left - lyric_ink_left).abs() < 0.01,
        "note_ink_left={note_ink_left} lyric_ink_left={lyric_ink_left} should coincide: a note \
         and a Latin lyric syllable sharing a column should have perfectly aligned left ink \
         edges"
    );
}

#[test]
fn note_head_and_cjk_lyric_syllable_have_the_same_ink_left_edge_in_one_column() {
    let note_anchor = resolve_anchor_x(GridContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
    });
    let lyric_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "春".to_string(),
        source_part_index: 0,
        note_id: 0,
        verse: 0,
    });

    let note_ink_left = ink_left_edge(
        note_anchor,
        monospace_glyph_left_bearing('1', NOTES_FONT_SIZE),
    );
    let lyric_ink_left = ink_left_edge(
        lyric_anchor,
        cjk_glyph_left_bearing('春', LYRIC_FONT_SIZES.cjk),
    );

    assert!(
        (note_ink_left - lyric_ink_left).abs() < 0.01,
        "note_ink_left={note_ink_left} lyric_ink_left={lyric_ink_left} should coincide: a note \
         and a CJK lyric syllable sharing a column should have perfectly aligned left ink edges"
    );
}

#[test]
fn note_head_and_chord_symbol_have_the_same_ink_left_edge_in_one_column() {
    let note_anchor = resolve_anchor_x(GridContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
    });
    let chord_anchor = resolve_anchor_x(GridContent::ChordSymbol {
        text: "1m".to_string(),
        dotted: false,
        double_dotted: false,
    });

    let note_ink_left = ink_left_edge(
        note_anchor,
        monospace_glyph_left_bearing('1', NOTES_FONT_SIZE),
    );
    let chord_ink_left = ink_left_edge(
        chord_anchor,
        monospace_glyph_left_bearing('1', CHORDS_FONT_SIZE),
    );

    assert!(
        (note_ink_left - chord_ink_left).abs() < 0.01,
        "note_ink_left={note_ink_left} chord_ink_left={chord_ink_left} should coincide: a note \
         and a chord symbol sharing a column should have perfectly aligned left ink edges"
    );
}

#[test]
fn latin_and_cjk_lyric_syllables_have_the_same_ink_left_edge_in_one_column() {
    let latin_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "la".to_string(),
        source_part_index: 0,
        note_id: 0,
        verse: 0,
    });
    let cjk_anchor = resolve_anchor_x(GridContent::LyricSyllable {
        text: "春".to_string(),
        source_part_index: 0,
        note_id: 0,
        verse: 0,
    });

    let latin_ink_left = ink_left_edge(
        latin_anchor,
        cjk_glyph_left_bearing('l', LYRIC_FONT_SIZES.base),
    );
    let cjk_ink_left = ink_left_edge(
        cjk_anchor,
        cjk_glyph_left_bearing('春', LYRIC_FONT_SIZES.cjk),
    );

    assert!(
        (latin_ink_left - cjk_ink_left).abs() < 0.01,
        "latin_ink_left={latin_ink_left} cjk_ink_left={cjk_ink_left} should coincide: a Latin \
         and a CJK lyric syllable sharing a column should have perfectly aligned left ink edges"
    );
}
