// ── Augmentation-dot column-width reservation ─────────────────────────────

use crate::ast::parsed::{Accidental, JianPuPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::font_metrics;
use crate::grid_layout::layout::measure_column_weights;
use crate::render_config::RenderConfig;

fn test_config() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 48,
        lyrics_font_size: 18,
        notes_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

/// Real rendered width of a single notehead/rest digit glyph at `config`'s
/// notes font size, mirroring `layout_spacing::note_glyph_weight`.
fn notehead_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('0', config.notes_font_size())
}

/// Real rendered width of the note-dash glyph at its own fixed font size,
/// mirroring `layout_spacing::dash_weight`.
fn dash_weight() -> f32 {
    font_metrics::monospace_char_advance_width('\u{2014}', font_metrics::NOTE_DASH_FONT_SIZE)
}

/// Small dedicated clearance added on top of a dot's own measured reach,
/// mirroring `layout_spacing_weights::DOT_CLEARANCE_PT` — kept smaller than
/// a fresh column's general clearance so a dot still reads as bound tightly
/// to the note/rest/dash it decorates.
const DOT_CLEARANCE_PT: f32 = 0.5;

/// Independently recomputed expected reach of a dotted note/rest/dash's
/// rightmost dot from its own glyph's left edge, mirroring the offset/
/// spacing formula `render_note_head`/`render_rest`/`render_note_dash`
/// actually draw at (`center + note_number_width * 1.5`, further dots
/// `note_number_width * DOT_SPACING_RATIO` apart, `TextAnchor::Middle` so a
/// dot's right edge sits half its own advance width past its anchor) —
/// deliberately not just calling `font_metrics::note_ish_dot_reach` back, so
/// this test can't pass merely because production and test share a bug.
fn expected_note_ish_dot_reach(dot_count: u32, note_number_width: f32, dot_font_size: f32) -> f32 {
    let center_offset = note_number_width * 0.5;
    let last_dot_anchor = center_offset
        + note_number_width * 1.5
        + (dot_count - 1) as f32 * note_number_width * font_metrics::DOT_SPACING_RATIO;
    let half_dot_advance =
        font_metrics::monospace_char_advance_width('\u{b7}', dot_font_size) / 2.0;
    last_dot_anchor + half_dot_advance
}

fn make_block(row_id: &str, content: ElementContent, bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            group_provenance: None,
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content,
                    note_id: None,
                },
                ColumnElement {
                    column: bar_col,
                    content: ElementContent::BarLine,
                    note_id: None,
                },
            ],
            source_part_index: 0,
        }],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        source_span: crate::error::Span::new(0, 0),
    }
}

fn note_head(dotted: bool, double_dotted: bool) -> ElementContent {
    ElementContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted,
        double_dotted,
    }
}

fn rest(dotted: bool, double_dotted: bool) -> ElementContent {
    ElementContent::Rest {
        dotted,
        double_dotted,
    }
}

fn note_dash(dotted: bool, double_dotted: bool) -> ElementContent {
    ElementContent::NoteDash {
        dotted,
        double_dotted,
    }
}

#[test]
fn dotted_note_head_column_clears_its_dot_s_real_rendered_right_edge() {
    // A dotted note's rod must reach at least as far right as the dot's own
    // rendered right edge (see `render_note_head`'s `center + note_number_width
    // * 1.5` dot anchor, `TextAnchor::Middle`) plus the dedicated dot clearance —
    // not the old flat, position-unaware guess that fell short of even the
    // dot's *position*, let alone its size.
    let config = test_config();
    let plain = make_block("S", note_head(false, false), 1);
    let dotted = make_block("S", note_head(true, false), 1);
    let double_dotted = make_block("S", note_head(true, true), 1);

    let plain_weight = measure_column_weights(&plain, 2, &config)[0];
    let dotted_weight = measure_column_weights(&dotted, 2, &config)[0];
    let double_dotted_weight = measure_column_weights(&double_dotted, 2, &config)[0];

    let expected_dotted_reach =
        expected_note_ish_dot_reach(1, config.note_number_width as f32, config.notes_font_size());
    let expected_double_dotted_reach =
        expected_note_ish_dot_reach(2, config.note_number_width as f32, config.notes_font_size());

    assert_eq!(plain_weight, notehead_weight(&config));
    assert!(
        (dotted_weight - (notehead_weight(&config).max(expected_dotted_reach) + DOT_CLEARANCE_PT))
            .abs()
            < 0.001,
        "dotted_weight={dotted_weight}"
    );
    assert!(
        (double_dotted_weight
            - (notehead_weight(&config).max(expected_double_dotted_reach) + DOT_CLEARANCE_PT))
            .abs()
            < 0.001,
        "double_dotted_weight={double_dotted_weight}"
    );
    assert!(
        double_dotted_weight > dotted_weight,
        "double_dotted_weight={double_dotted_weight} should be > dotted_weight={dotted_weight}"
    );
}

#[test]
fn dotted_rest_column_clears_its_dot_s_real_rendered_right_edge() {
    let config = test_config();
    let dotted = make_block("S", rest(true, false), 1);
    let dotted_weight = measure_column_weights(&dotted, 2, &config)[0];

    let expected_reach =
        expected_note_ish_dot_reach(1, config.note_number_width as f32, config.notes_font_size());
    assert!(
        (dotted_weight - (notehead_weight(&config).max(expected_reach) + DOT_CLEARANCE_PT)).abs()
            < 0.001,
        "dotted_weight={dotted_weight}"
    );
}

#[test]
fn dotted_note_dash_column_clears_its_dot_s_real_rendered_right_edge_at_its_own_fixed_font_size() {
    // `render_note_dash` draws its dot(s) at `NOTE_DASH_FONT_SIZE`, not
    // `config`'s notes font size, so the dash's own reach must be measured at
    // that fixed size to match what's actually drawn.
    let config = test_config();
    let dotted = make_block("S", note_dash(true, false), 1);
    let dotted_weight = measure_column_weights(&dotted, 2, &config)[0];

    let expected_reach = expected_note_ish_dot_reach(
        1,
        config.note_number_width as f32,
        font_metrics::NOTE_DASH_FONT_SIZE,
    );
    assert!(
        (dotted_weight - (dash_weight().max(expected_reach) + DOT_CLEARANCE_PT)).abs() < 0.001,
        "dotted_weight={dotted_weight}"
    );
}
