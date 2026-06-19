use super::beam::{flush_beam_buffer, BeamEntry};
use super::slur_chains::{extend_note_chains, PendingSlurOpen, SlurKey};
use super::PartSliceResult;
use crate::ast::grouped::{GroupedChordNote, GroupedNote, GroupedRest, NoteEvent, PartSlice};
use crate::ast::parsed::{JianPuPitch, PartKind, Syllable};
use crate::compiler::types::{ColumnElement, ElementContent, SlurSpan};

// ── Part slice compiler ───────────────────────────────────────────────────────

struct PartState<'a> {
    elements: &'a mut Vec<ColumnElement>,
    beam_buf: &'a mut Vec<BeamEntry>,
    pending_chains: &'a mut Vec<Vec<(u32, SlurKey)>>,
    pending_slur_opens: &'a mut Vec<Option<PendingSlurOpen>>,
    slur_spans: &'a mut Vec<SlurSpan>,
    prev_tie: &'a mut bool,
    prev_tie_column: &'a mut Option<u32>,
    prev_tie_measure: &'a mut Option<usize>,
    prev_slur_key: &'a mut Option<SlurKey>,
    col: &'a mut u32,
    cross_measure_open: &'a mut bool,
    measure_index: usize,
    part_index: usize,
}

// ── Shared compile-unit abstraction ──────────────────────────────────────────

trait HeadContent {
    fn into_element_content(self, dotted: bool) -> ElementContent;
}

struct NoteUnit {
    pitch: JianPuPitch,
    octave: i8,
}

struct ChordUnit {
    symbol: String,
}

impl HeadContent for NoteUnit {
    fn into_element_content(self, dotted: bool) -> ElementContent {
        ElementContent::NoteHead {
            pitch: self.pitch,
            octave: self.octave,
            dotted,
        }
    }
}

impl HeadContent for ChordUnit {
    fn into_element_content(self, _dotted: bool) -> ElementContent {
        ElementContent::ChordSymbol(self.symbol)
    }
}

struct CompiledUnit<H> {
    duration: u32,
    dotted: bool,
    tie: bool,
    group_membership: u8,
    group_continuation: u8,
    slur_close_at: Option<u32>,
    slur_key: SlurKey,
    head: H,
}

fn compile_unit<H: HeadContent>(
    state: &mut PartState<'_>,
    unit: CompiledUnit<H>,
    measure_col_start: u32,
) {
    state.elements.push(ColumnElement {
        column: *state.col,
        content: unit.head.into_element_content(unit.dotted),
    });

    let underline_count = match unit.duration {
        1 => 2,
        2 | 3 => 1,
        _ => 0,
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    extend_note_chains(
        state.pending_chains,
        state.pending_slur_opens,
        unit.group_membership,
        unit.group_continuation,
        *state.col,
        &unit.slur_key,
        state.slur_spans,
        state.measure_index,
        state.part_index,
    );

    if let Some(close_offset) = unit.slur_close_at {
        if unit.group_membership > 0 {
            extend_note_chains(
                state.pending_chains,
                state.pending_slur_opens,
                unit.group_membership,
                0,
                *state.col + close_offset,
                &SlurKey::Rest,
                state.slur_spans,
                state.measure_index,
                state.part_index,
            );
        }
    }

    if unit.tie {
        *state.prev_tie_column = Some(*state.col);
        *state.prev_tie_measure = Some(state.measure_index);
    } else {
        *state.prev_tie_column = None;
        *state.prev_tie_measure = None;
    }
    *state.prev_tie = unit.tie;
    *state.prev_slur_key = Some(unit.slur_key);

    if !unit.dotted {
        let note_col = *state.col;
        for dash_col in (note_col + 4..note_col + unit.duration).step_by(4) {
            state.elements.push(ColumnElement {
                column: dash_col,
                content: ElementContent::NoteDash,
            });
        }
    }

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: unit.duration,
        });
    }

    *state.col += unit.duration;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}

// ── Top-level entry point ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_part_slice(
    slice: &PartSlice,
    initial_prev_tie: bool,
    initial_prev_tie_column: Option<u32>,
    initial_prev_tie_measure: Option<usize>,
    initial_prev_slur_key: Option<SlurKey>,
    initial_pending_opens: Vec<Option<PendingSlurOpen>>,
    measure_index: usize,
    part_index: usize,
    slur_spans: &mut Vec<SlurSpan>,
) -> PartSliceResult {
    let mut elements: Vec<ColumnElement> = Vec::new();
    let mut beam_buf: Vec<BeamEntry> = Vec::new();
    let mut pending_chains: Vec<Vec<(u32, SlurKey)>> = Vec::new();
    let mut pending_slur_opens: Vec<Option<PendingSlurOpen>> = initial_pending_opens;
    let mut prev_tie = initial_prev_tie;
    let mut prev_tie_column: Option<u32> = initial_prev_tie_column;
    let mut prev_tie_measure: Option<usize> = initial_prev_tie_measure;
    let mut prev_slur_key: Option<SlurKey> = initial_prev_slur_key;
    let mut col: u32 = 0;
    let measure_col_start: u32 = 0;
    let mut cross_measure_open = initial_prev_tie;

    let mut lyrics_iter = slice.lyrics.as_ref().map(|l| l.syllables.iter());

    let (final_tie, final_tie_column, final_tie_measure, final_key) = {
        let mut state = PartState {
            elements: &mut elements,
            beam_buf: &mut beam_buf,
            pending_chains: &mut pending_chains,
            pending_slur_opens: &mut pending_slur_opens,
            slur_spans,
            prev_tie: &mut prev_tie,
            prev_tie_column: &mut prev_tie_column,
            prev_tie_measure: &mut prev_tie_measure,
            prev_slur_key: &mut prev_slur_key,
            col: &mut col,
            cross_measure_open: &mut cross_measure_open,
            measure_index,
            part_index,
        };

        for event in &slice.notes.events {
            match event {
                NoteEvent::Note(note) => {
                    compile_note(
                        &mut state,
                        note,
                        measure_col_start,
                        &mut lyrics_iter,
                        slice.kind,
                    );
                }
                NoteEvent::Rest(rest) => {
                    compile_rest(&mut state, rest, measure_col_start);
                }
                NoteEvent::Chord(chord) => {
                    compile_chord(&mut state, chord, measure_col_start);
                }
            }
        }

        flush_beam_buffer(state.beam_buf, state.elements);

        (
            *state.prev_tie,
            *state.prev_tie_column,
            *state.prev_tie_measure,
            state.prev_slur_key.clone(),
        )
    };

    preserve_cross_measure_slur_opens(&pending_chains, &mut pending_slur_opens, measure_index);

    elements.push(ColumnElement {
        column: col,
        content: ElementContent::BarLine,
    });

    PartSliceResult {
        elements,
        final_tie,
        final_tie_column,
        final_tie_measure,
        final_slur_key: final_key,
        final_pending_opens: pending_slur_opens,
    }
}

fn preserve_cross_measure_slur_opens(
    pending_chains: &[Vec<(u32, SlurKey)>],
    pending_slur_opens: &mut Vec<Option<PendingSlurOpen>>,
    measure_index: usize,
) {
    for (depth, chain) in pending_chains.iter().enumerate() {
        if chain.len() > 1 {
            let origin = pending_slur_opens
                .get(depth)
                .and_then(|o| o.as_ref())
                .map(|o| (o.measure_index, o.from_column))
                .or_else(|| chain.first().map(|(column, _)| (measure_index, *column)));
            while pending_slur_opens.len() <= depth {
                pending_slur_opens.push(None);
            }
            if let (Some(origin), Some(slot)) = (origin, pending_slur_opens.get_mut(depth)) {
                *slot = Some(PendingSlurOpen {
                    measure_index: origin.0,
                    from_column: origin.1,
                });
            }
        } else if let Some((chain_col, _)) = chain.first() {
            while pending_slur_opens.len() <= depth {
                pending_slur_opens.push(None);
            }
            if pending_slur_opens
                .get(depth)
                .and_then(|o| o.as_ref())
                .is_none()
            {
                if let Some(slot) = pending_slur_opens.get_mut(depth) {
                    *slot = Some(PendingSlurOpen {
                        measure_index,
                        from_column: *chain_col,
                    });
                }
            }
        }
    }
}

fn compile_note(
    state: &mut PartState<'_>,
    note: &GroupedNote,
    measure_col_start: u32,
    lyrics_iter: &mut Option<std::slice::Iter<'_, Syllable>>,
    kind: PartKind,
) {
    let slur_key = SlurKey::Pitch(note.pitch.clone());
    let is_tie_continuation = *state.prev_tie && state.prev_slur_key.as_ref() == Some(&slur_key);

    if *state.cross_measure_open && is_tie_continuation {
        if let (Some(from_col), Some(from_measure)) =
            (*state.prev_tie_column, *state.prev_tie_measure)
        {
            state.slur_spans.push(SlurSpan {
                part_index: state.part_index,
                from_measure,
                from_column: from_col,
                to_measure: state.measure_index,
                to_column: *state.col,
            });
        }
        *state.cross_measure_open = false;
    }

    if kind == PartKind::NotesWithLyrics && !is_tie_continuation {
        if let Some(ref mut iter) = lyrics_iter {
            if let Some(syllable) = iter.next() {
                state.elements.push(ColumnElement {
                    column: *state.col,
                    content: ElementContent::Lyric(syllable.text.clone()),
                });
            }
        }
    }

    compile_unit(
        state,
        CompiledUnit {
            duration: note.duration,
            dotted: note.dotted,
            tie: note.tie,
            group_membership: note.group_membership,
            group_continuation: note.group_continuation,
            slur_close_at: note.slur_group_close_at_duration,
            slur_key,
            head: NoteUnit {
                pitch: note.pitch.clone(),
                octave: note.octave,
            },
        },
        measure_col_start,
    );
}

fn compile_rest(state: &mut PartState<'_>, rest: &GroupedRest, measure_col_start: u32) {
    let underline_count = match rest.duration {
        1 => 2,
        2 => 1,
        _ => 0,
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    state.elements.push(ColumnElement {
        column: *state.col,
        content: ElementContent::Rest {
            dotted: rest.dotted,
        },
    });

    if rest.group_membership > 0 {
        extend_note_chains(
            state.pending_chains,
            state.pending_slur_opens,
            rest.group_membership,
            rest.group_continuation,
            *state.col,
            &SlurKey::Rest,
            state.slur_spans,
            state.measure_index,
            state.part_index,
        );
    }

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: rest.duration,
        });
    }

    *state.col += rest.duration;
    *state.prev_tie = false;
    *state.prev_tie_column = None;
    *state.prev_tie_measure = None;
    *state.prev_slur_key = None;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}

fn compile_chord(state: &mut PartState<'_>, chord: &GroupedChordNote, measure_col_start: u32) {
    let slur_key = SlurKey::from_chord(chord);
    compile_unit(
        state,
        CompiledUnit {
            duration: chord.duration,
            dotted: chord.dotted,
            tie: chord.tie,
            group_membership: chord.group_membership,
            group_continuation: chord.group_continuation,
            slur_close_at: chord.slur_group_close_at_duration,
            slur_key,
            head: ChordUnit {
                symbol: chord.format_symbol(),
            },
        },
        measure_col_start,
    );
}
