// ── Sharp/flat accidental column-width reservation ────────────────────────────

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
        measure_number_font_size: 10,
        section_label_font_size: 12,
        part_label_font_size: 12,
        page_number_font_size: 18,
        lyric_click_target_padding_pt: 12,
        notes_horizontal_padding_pt: 4,
        chords_horizontal_padding_pt: 4,
        lyrics_horizontal_padding_pt: 4,
        note_dash_horizontal_padding_pt: 4,
    }
}

/// Real rendered width of a single notehead digit glyph at `config`'s notes
/// font size, mirroring `layout_spacing::note_glyph_weight`.
fn notehead_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('0', config.notes_font_size())
}

/// Expected total column weight of a `NoteHead` carrying `symbol` (`"♯"` or
/// `"♭"`) as its accidental, mirroring `layout_spacing::accidental_extra_weight`:
/// the accidental now renders as part of the note digit's own text run, at
/// the same font size as the digit (see `render_note_head`), so its own real
/// rendered width at that size is exactly the extra room needed.
fn accidental_note_weight(symbol: &str, config: &RenderConfig) -> f32 {
    notehead_weight(config) + font_metrics::monospace_text_width(symbol, config.notes_font_size())
}

fn make_block_with_accidental_note(
    row_id: &str,
    accidental: Accidental,
    bar_col: u32,
) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            absorbed_rows: Vec::new(),
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        accidental,
                        octave: 0,
                        dotted: false,
                        double_dotted: false,
                    },
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
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

#[test]
fn measure_column_weights_gives_sharp_or_flat_note_wider_weight_than_plain_note() {
    // A sharp/flat glyph renders as part of the note head's own text run
    // (see `render_note_head` in `glyph_renderers.rs`) and needs its own
    // reserved room, unlike a natural note which appends no accidental
    // character at all.
    let config = test_config();
    let plain = make_block_with_accidental_note("S", Accidental::Natural, 1);
    let sharp = make_block_with_accidental_note("S", Accidental::Sharp, 1);
    let flat = make_block_with_accidental_note("S", Accidental::Flat, 1);

    let plain_weight = measure_column_weights(&plain, 2, &config)[0];
    let sharp_weight = measure_column_weights(&sharp, 2, &config)[0];
    let flat_weight = measure_column_weights(&flat, 2, &config)[0];

    assert_eq!(plain_weight, notehead_weight(&config));
    assert!(
        sharp_weight > plain_weight,
        "sharp_weight={sharp_weight} should be > plain_weight={plain_weight}"
    );
    assert!(
        flat_weight > plain_weight,
        "flat_weight={flat_weight} should be > plain_weight={plain_weight}"
    );
    assert_eq!(sharp_weight, accidental_note_weight("\u{266F}", &config));
    assert_eq!(flat_weight, accidental_note_weight("\u{266D}", &config));
}
