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
    }
}

/// A `notes+lyrics` part's notes row: two plain notes followed by a bar
/// line, sharing `source_part_index` with `lyric_verse_row`.
fn notes_row_with_two_notes() -> MeasureRow {
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
                },
                note_id: Some(0),
            },
            ColumnElement {
                column: 1,
                content: ElementContent::NoteHead {
                    pitch: JianPuPitch::Two,
                    accidental: Accidental::Natural,
                    octave: 0,
                    dotted: false,
                },
                note_id: Some(1),
            },
            ColumnElement {
                column: 2,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 0,
    }
}

/// The sibling lyric-verse row that follows `notes_row_with_two_notes` —
/// see `ElementContent::Lyric`'s doc comment for why lyrics live in their
/// own row rather than the notes row's own elements.
fn lyric_verse_row() -> MeasureRow {
    MeasureRow {
        id: RowId("V-lyrics-0".to_string()),
        group_provenance: None,
        label: String::new(),
        elements: vec![
            ColumnElement {
                column: 0,
                content: ElementContent::Lyric {
                    text: "Ho".to_string(),
                    verse: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: 1,
                content: ElementContent::Lyric {
                    text: "Ho".to_string(),
                    verse: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: 2,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 0,
    }
}

fn test_render_config() -> RenderConfig {
    RenderConfig {
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

#[test]
fn playback_cursor_target_extends_over_its_lyric_verse_row() {
    // A `notes+lyrics` part compiles to two sibling `MeasureRow`s sharing
    // `source_part_index` — a notes row and a separate lyric-verse row (see
    // `ElementContent::Lyric`'s doc comment) — rather than lyrics being
    // mixed into the notes row's own elements. Each note's highlight target
    // must still reach down through its part's verse row(s) so the
    // highlight rect covers the lyric text too, not just the note glyph.
    let block = MeasureBlock {
        rows: vec![notes_row_with_two_notes(), lyric_verse_row()],
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

    let targets = &pages[0].playback_cursor_targets;
    assert_eq!(
        targets.len(),
        2,
        "one highlight target per note, none for the lyric-only row itself"
    );
    for target in targets {
        assert_eq!(
            target.row_end - target.row_start,
            6,
            "note's highlight span should cover its 6 note sub-rows (no \
             tuplet in this block, so no tuplet_bracket sub-row) plus the 1 \
             lyric-verse row that follows them, not just the note sub-rows"
        );
    }
}
