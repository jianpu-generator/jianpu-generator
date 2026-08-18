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
                    double_dotted: false,
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
                    double_dotted: false,
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
                    note_id: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: 1,
                content: ElementContent::Lyric {
                    text: "Ho".to_string(),
                    verse: 0,
                    note_id: 0,
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
        notes_font_size: 14,
        chords_font_size: 14,
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

/// A notes-only row (no lyrics) with two plain notes followed by a bar line,
/// like `notes_row_with_two_notes` but with the note ids and grid columns
/// parameterized so two of these can sit in adjacent measures of the same
/// system without colliding.
fn notes_row_two_notes(first_note_id: usize) -> MeasureRow {
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
                note_id: Some(first_note_id),
            },
            ColumnElement {
                column: 1,
                content: ElementContent::NoteHead {
                    pitch: JianPuPitch::Two,
                    accidental: Accidental::Natural,
                    octave: 0,
                    dotted: false,
                    double_dotted: false,
                },
                note_id: Some(first_note_id + 1),
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

#[test]
fn playback_cursor_targets_snap_to_bar_lines_at_measure_edges() {
    // Two consecutive measures in one system, each with two notes (ids
    // 0,1 and 2,3) followed by a bar line. Verifies that the first note's
    // left edge, the shared inter-measure bar line, and the last note's
    // right edge all snap to where the bar line is actually rendered (see
    // `compute_all_playback_cursor_targets` in `playback_cursor.rs`) rather
    // than stopping at the raw note-column boundary.
    let block = |first_note_id: usize| MeasureBlock {
        rows: vec![notes_row_two_notes(first_note_id)],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
    };

    let pages = crate::grid_layout::layout(
        &CompileResult {
            blocks: vec![block(0), block(2)],
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
    assert_eq!(targets.len(), 4, "one target per note across both measures");

    let target_for = |note_id: usize| {
        targets
            .iter()
            .find(|t| t.note_id == note_id)
            .unwrap_or_else(|| panic!("no target for note_id {note_id}"))
    };

    // First measure: note 0 is its first note, so its left edge snaps to
    // the system's leading bar line (`Start`-aligned, at `LABEL_COLS` = 1).
    assert_eq!(target_for(0).column_start, 1.0);
    // Note 1 is the first measure's last note, so its right edge snaps to
    // the inter-measure bar line, `Center`-aligned within its own column
    // (bar-line column 4, so 4.0 + 0.5).
    assert_eq!(target_for(1).column_end, 4.5);

    // Second measure: note 2 is its first note, so its left edge snaps to
    // that same shared inter-measure bar line — matching note 1's right
    // edge exactly, with no double gap between the two measures.
    assert_eq!(target_for(2).column_start, 4.5);
    // Note 3 is the last note of the system's last measure, so its right
    // edge snaps to the closing bar line, `End`-aligned (flush right) within
    // its own column (bar-line column 7, so 7.0 + 1.0).
    assert_eq!(target_for(3).column_end, 8.0);
}

#[test]
fn playback_cursor_target_snaps_to_bar_line_despite_trailing_subdivision_padding() {
    // A single-note measure whose bar line sits several raw grid columns
    // past the note's own column (column 5, not column 1) — mirroring a
    // real quarter note in a finer (e.g. sixteenth-note) subdivision grid,
    // which leaves empty trailing columns of its own before the bar line.
    // The note is still the measure's only (and thus last) note, so its
    // right edge must snap to the bar line despite not being immediately
    // adjacent to it.
    let block = MeasureBlock {
        rows: vec![MeasureRow {
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
                    column: 5,
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

    let target = &pages[0].playback_cursor_targets[0];
    // Single block in the system, so its bar line is `End`-aligned: bar-line
    // column is `MUSIC_START_COL + 5 = 7`, so `7.0 + 1.0`.
    assert_eq!(target.column_end, 8.0);
}

#[test]
fn playback_cursor_target_extends_to_next_note_without_its_own_trailing_column() {
    // A dotted note only ever gets a `ColumnElement` at its own head column
    // (no `NoteDash` continuation columns — those are only emitted for
    // non-dotted notes, to avoid a stray dash glyph right after the dot; see
    // `compile_unit` in `part_slice_unit.rs`). Mirror that here with a note
    // at column 0 and nothing else until the next note at column 3: its
    // right edge must still reach that next note's left edge, not stop short
    // at its own column boundary (which previously left a visible gap
    // between the two notes' playback-cursor rects).
    let block = MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("V".to_string()),
            group_provenance: None,
            label: String::new(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::Six,
                        accidental: Accidental::Natural,
                        octave: 0,
                        dotted: true,
                        double_dotted: false,
                    },
                    note_id: Some(0),
                },
                ColumnElement {
                    column: 3,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        accidental: Accidental::Natural,
                        octave: 0,
                        dotted: false,
                        double_dotted: false,
                    },
                    note_id: Some(1),
                },
                ColumnElement {
                    column: 4,
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
    let target_for = |note_id: usize| {
        targets
            .iter()
            .find(|t| t.note_id == note_id)
            .unwrap_or_else(|| panic!("no target for note_id {note_id}"))
    };

    // Note 1's left edge (column 3, within a block starting at
    // `MUSIC_START_COL` = 2, so 5.0) is where note 0's right edge must land
    // too, leaving no gap between the two rects.
    assert_eq!(target_for(0).column_end, target_for(1).column_start);
    assert_eq!(target_for(0).column_end, 5.0);
}
