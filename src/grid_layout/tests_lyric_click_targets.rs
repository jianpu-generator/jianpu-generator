use crate::ast::parsed::{Accidental, JianPuPitch, Offset};
use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::types::Header;
use crate::render_config::RenderConfig;

fn no_header() -> Header {
    Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: 36.0,
        subtitle_font_size: 19.0,
        author_font_size: 14.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
    }
}

fn test_render_config() -> RenderConfig {
    RenderConfig {
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        notes_font_size: 14,
        chords_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

/// A `notes+lyrics` part's notes row: one two-beat note (a note head at
/// column 0 followed by its own dash-continuation column at column 1) then
/// a plain one-beat note at column 2, followed by a bar line — sharing
/// `source_part_index` with `lyric_verse_row`.
fn notes_row_with_a_two_beat_note() -> MeasureRow {
    MeasureRow {
        id: RowId("V".to_string()),
        group_provenance: None,
        label: String::new(),
        elements: vec![
            ColumnElement {
                column: 0,
                content: ElementContent::NoteHead {
                    pitch: JianPuPitch::One,
                    accidental: Accidental::Natural,
                    octave: 0,
                    dotted: false,
                    double_dotted: false,
                },
                note_id: Some(0),
            },
            ColumnElement {
                column: 1,
                content: ElementContent::NoteDash {
                    dotted: false,
                    double_dotted: false,
                },
                note_id: Some(0),
            },
            ColumnElement {
                column: 2,
                content: ElementContent::NoteHead {
                    pitch: JianPuPitch::Two,
                    accidental: Accidental::Natural,
                    octave: 0,
                    dotted: false,
                    double_dotted: false,
                },
                note_id: Some(1),
            },
            ColumnElement {
                column: 3,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 0,
    }
}

/// The sibling lyric-verse row that follows `notes_row_with_a_two_beat_note`
/// — one syllable per note, each placed at its note's own start column (see
/// `ElementContent::Lyric`'s doc comment for why lyrics live in their own row
/// rather than the notes row's own elements).
fn lyric_verse_row() -> MeasureRow {
    MeasureRow {
        id: RowId("V-lyrics-0".to_string()),
        group_provenance: None,
        label: String::new(),
        elements: vec![
            ColumnElement {
                column: 0,
                content: ElementContent::Lyric {
                    text: "Hoo".to_string(),
                    verse: 0,
                    note_id: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: 2,
                content: ElementContent::Lyric {
                    text: "ray".to_string(),
                    verse: 0,
                    note_id: 1,
                },
                note_id: None,
            },
            ColumnElement {
                column: 3,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 0,
    }
}

/// A lyric syllable's own hover/click box must span its whole note's written
/// duration, not just the single column its text is drawn in — otherwise a
/// two-beat note's syllable is only hoverable/selectable over its first
/// beat. Regression test for the bug where `column_end` was always
/// `column_start + 1.0` regardless of the note's dash-continuation columns.
#[test]
fn lyric_click_target_spans_its_note_full_column_width() {
    let block = MeasureBlock {
        rows: vec![notes_row_with_a_two_beat_note(), lyric_verse_row()],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
    };

    let pages = crate::grid_layout::layout(
        &CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
            tuplet_spans: vec![],
        },
        &test_render_config(),
        &no_header(),
        595.0,
        842.0,
        None,
    );

    let targets = &pages[0].lyric_click_targets;
    assert_eq!(targets.len(), 2, "one click target per syllable");

    let two_beat = targets
        .iter()
        .find(|t| t.note_id == 0)
        .expect("syllable for the two-beat note");
    assert_eq!(
        two_beat.column_end - two_beat.column_start,
        2.0,
        "the two-beat note's syllable box should span both of its columns \
         (attack + dash-continuation), not just the attack column"
    );

    let one_beat = targets
        .iter()
        .find(|t| t.note_id == 1)
        .expect("syllable for the one-beat note");
    assert_eq!(
        one_beat.column_end - one_beat.column_start,
        1.0,
        "a plain one-beat note's syllable box should still span exactly one column"
    );
}
