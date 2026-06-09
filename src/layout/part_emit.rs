use crate::ast::grouped::{
    ChordSlice, GroupedChordEvent, GroupedNote, GroupedRest, NoteEvent, PartSlice,
};
use crate::ast::parsed::{JianPuPitch, Syllable};
use crate::layout::types::{
    GridContent, GridElement, GridPosition, HorizontalAlignment, VerticalAlignment,
};
use crate::utils::is_cjk_char;

use super::super::{extend_note_chains, flush_beam_buffer, format_chord_symbol, BeamBufferEntry};

pub(crate) struct PartNoteState<'a> {
    pub elements: &'a mut Vec<GridElement>,
    pub label_cols: u32,
    pub beam_buf: &'a mut Vec<BeamBufferEntry>,
    pub pending_chains: &'a mut Vec<Vec<(u32, JianPuPitch)>>,
    pub chain_row: &'a mut u32,
    pub prev_tie: &'a mut bool,
    pub prev_pitch: &'a mut Option<JianPuPitch>,
    pub cross_line_tie: &'a mut Option<JianPuPitch>,
}

pub(crate) fn emit_chord_part(
    elements: &mut Vec<GridElement>,
    chord_slice: &ChordSlice,
    main_row_cursor: u32,
    note_col_start: u32,
) {
    let mut col = note_col_start;
    for event in &chord_slice.events {
        match event {
            GroupedChordEvent::Chord(chord) => {
                let text = format_chord_symbol(chord);
                elements.push(GridElement {
                    position: GridPosition {
                        column: col,
                        row: main_row_cursor + 1,
                    },
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Center,
                    content: GridContent::ChordSymbol { text },
                });
                col += chord.duration;
            }
            GroupedChordEvent::Rest(dur) => {
                col += dur;
            }
        }
    }
}

pub(crate) fn emit_notes_part(
    state: &mut PartNoteState<'_>,
    part_slice: &PartSlice,
    part_row_offset: u32,
    note_col_start: u32,
) {
    let mut col = note_col_start;
    let measure_col_start_for_part = note_col_start;

    if state.pending_chains.is_empty() || state.pending_chains.iter().all(|c| c.is_empty()) {
        *state.chain_row = part_row_offset + 1;
    }

    let mut lyrics_iter = part_slice.lyrics.as_ref().map(|l| l.syllables.iter());

    for note_event in &part_slice.notes.events {
        match note_event {
            NoteEvent::Note(note) => {
                emit_grouped_note(
                    state,
                    note,
                    &mut col,
                    part_row_offset,
                    measure_col_start_for_part,
                    &mut lyrics_iter,
                );
            }
            NoteEvent::Rest(rest) => {
                emit_grouped_rest(
                    state,
                    rest,
                    &mut col,
                    part_row_offset,
                    measure_col_start_for_part,
                );
            }
        }
    }

    flush_beam_buffer(state.beam_buf, part_row_offset, state.elements);
}

fn push_note_head_elements(
    elements: &mut Vec<GridElement>,
    note: &GroupedNote,
    col: u32,
    part_row_offset: u32,
) {
    elements.push(GridElement {
        position: GridPosition {
            column: col,
            row: part_row_offset + 1,
        },
        horizontal_alignment: HorizontalAlignment::Center,
        vertical_alignment: VerticalAlignment::Center,
        content: GridContent::NoteHead {
            pitch: note.pitch.clone(),
            octave: note.octave,
            dotted: note.dotted,
        },
    });

    if note.octave < 0 {
        let dot_underline_count = match note.duration {
            1 => 2u8,
            2 | 3 => 1u8,
            _ => 0u8,
        };
        elements.push(GridElement {
            position: GridPosition {
                column: col,
                row: part_row_offset + 2,
            },
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Top,
            content: GridContent::LowerOctaveDots {
                count: (-note.octave) as u32,
                underline_count: dot_underline_count,
            },
        });
    }

    if note.duration > 4 {
        let extra_beats = (note.duration - 4) / 4;
        for i in 0..extra_beats {
            elements.push(GridElement {
                position: GridPosition {
                    column: col + 4 + i * 4,
                    row: part_row_offset + 1,
                },
                horizontal_alignment: HorizontalAlignment::Center,
                vertical_alignment: VerticalAlignment::Center,
                content: GridContent::Extension,
            });
        }
    }
}

fn emit_grouped_note(
    state: &mut PartNoteState<'_>,
    note: &GroupedNote,
    col: &mut u32,
    part_row_offset: u32,
    measure_col_start_for_part: u32,
    lyrics_iter: &mut Option<std::slice::Iter<'_, Syllable>>,
) {
    push_note_head_elements(state.elements, note, *col, part_row_offset);

    let underline_count = match note.duration {
        1 => 2,
        2 | 3 => 1,
        _ => 0,
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, part_row_offset, state.elements);
    }

    extend_note_chains(
        state.pending_chains,
        note.group_membership,
        note.group_continuation,
        *state.chain_row,
        *col,
        &note.pitch,
        state.elements,
    );

    let is_tie_continuation = *state.prev_tie && state.prev_pitch.as_ref() == Some(&note.pitch);

    if state.cross_line_tie.is_some() {
        if is_tie_continuation && *col > state.label_cols {
            state.elements.push(GridElement {
                position: GridPosition {
                    column: state.label_cols,
                    row: *state.chain_row,
                },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
                content: GridContent::TieOrSlurCurve {
                    from_column: state.label_cols,
                    to_column: *col,
                },
            });
        }
        *state.cross_line_tie = None;
    }

    if !is_tie_continuation {
        if let Some(ref mut iter) = lyrics_iter {
            if let Some(syllable) = iter.next() {
                let is_cjk = syllable
                    .text
                    .chars()
                    .next()
                    .map(is_cjk_char)
                    .unwrap_or(false);
                state.elements.push(GridElement {
                    position: GridPosition {
                        column: *col,
                        row: part_row_offset + 3,
                    },
                    horizontal_alignment: HorizontalAlignment::Center,
                    vertical_alignment: VerticalAlignment::Top,
                    content: GridContent::Lyric {
                        text: syllable.text.clone(),
                        is_cjk,
                    },
                });
            }
        }
    }
    *state.prev_tie = note.tie;
    *state.prev_pitch = Some(note.pitch.clone());

    if underline_count > 0 {
        state.beam_buf.push(BeamBufferEntry {
            column: *col,
            underline_count,
            duration: note.duration,
        });
    }

    *col += note.duration;

    let beat_position = *col - measure_col_start_for_part;
    if underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, part_row_offset, state.elements);
    }
}

fn emit_grouped_rest(
    state: &mut PartNoteState<'_>,
    rest: &GroupedRest,
    col: &mut u32,
    part_row_offset: u32,
    measure_col_start_for_part: u32,
) {
    let rest_underline_count = match rest.duration {
        1 => 2,
        2 => 1,
        _ => 0,
    };
    if rest_underline_count == 0 {
        flush_beam_buffer(state.beam_buf, part_row_offset, state.elements);
    }
    state.elements.push(GridElement {
        position: GridPosition {
            column: *col,
            row: part_row_offset + 1,
        },
        horizontal_alignment: HorizontalAlignment::Center,
        vertical_alignment: VerticalAlignment::Center,
        content: GridContent::Rest,
    });
    if rest_underline_count > 0 {
        state.beam_buf.push(BeamBufferEntry {
            column: *col,
            underline_count: rest_underline_count,
            duration: rest.duration,
        });
    }
    *col += rest.duration;
    *state.prev_tie = false;
    *state.cross_line_tie = None;
    let beat_position = *col - measure_col_start_for_part;
    if rest_underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, part_row_offset, state.elements);
    }
}
