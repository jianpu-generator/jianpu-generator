use crate::ast::grouped::{
    GroupedChordNote, GroupedMeasure, GroupedNote, GroupedPart, GroupedPercussionHit, GroupedRest,
    NoteEvent, Notes, TimeSignature,
};
use crate::ast::parsed::{
    ParsedChordNote, ParsedMeasureSlot, ParsedNote, ParsedPercussionHit, ParsedRest,
    ParsedTimedTrack, PartKind, ScoreEvent, Soundfont,
};
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Warning};

use super::empty_note_measures::{align_empty_note_measures, MeasureSlot, PerMeasureErrors};
use super::lyrics_pairing::attach_paired_lyrics;

#[path = "part_grouper_group.rs"]
mod part_grouper_group;
pub(super) use part_grouper_group::group_timed_track;

struct PartGrouper {
    part_kind: PartKind,
    soundfont: Soundfont,
    slots: Vec<MeasureSlot>,
    current_notes: Vec<NoteEvent>,
    current_beat: u32,
    capacity: u32,
    part_name: Option<String>,
    measure_span_start: Option<usize>,
    measure_span_end: usize,
    pending_dash_after_rest_error: Option<RecoverableError>,
    pending_overflow_error: Option<Warning>,
    pending_dotted_eighth_errors: Vec<Diagnostic>,
    pending_extension_no_preceding_event_error: Option<RecoverableError>,
}

impl PartGrouper {
    fn new(part: &ParsedTimedTrack) -> Self {
        let current_time_sig = TimeSignature {
            numerator: 4,
            denominator: 4,
        };
        let capacity = Self::measure_capacity(&current_time_sig);

        Self {
            part_kind: part.kind,
            soundfont: part.soundfont,
            slots: Vec::new(),
            current_notes: Vec::new(),
            current_beat: 0,
            capacity,
            part_name: Some(part.abbreviation.clone()),
            measure_span_start: None,
            measure_span_end: 0,
            pending_dash_after_rest_error: None,
            pending_overflow_error: None,
            pending_dotted_eighth_errors: Vec::new(),
            pending_extension_no_preceding_event_error: None,
        }
    }

    fn measure_capacity(ts: &TimeSignature) -> u32 {
        (ts.numerator as u32) * 16 / (ts.denominator as u32)
    }

    fn flush_measure(&mut self) {
        if self.current_notes.is_empty() {
            return;
        }
        let source_span = Span::new(self.measure_span_start.unwrap_or(0), self.measure_span_end);
        self.slots.push(MeasureSlot::Real(Box::new(GroupedMeasure {
            notes: Notes {
                events: std::mem::take(&mut self.current_notes),
            },
            source_span,
            group_provenance: None,
            paired_lyrics: Vec::new(),
            lyrics_error: Vec::new(),
            beat_overflow_error: self.pending_overflow_error.take(),
            dash_after_rest_error: self.pending_dash_after_rest_error.take(),
            dotted_eighth_errors: std::mem::take(&mut self.pending_dotted_eighth_errors),
            chord_errors: Vec::new(),
            lex_error: None,
            lyrics_parse_error: None,
            extension_no_preceding_event_error: self
                .pending_extension_no_preceding_event_error
                .take(),
        })));
        self.current_beat = 0;
        self.measure_span_start = None;
        self.measure_span_end = 0;
    }

    fn push_empty_note_slot(&mut self, span: Span) {
        self.slots.push(MeasureSlot::EmptyNote { span });
    }

    fn flush_if_full(&mut self) {
        if self.current_beat >= self.capacity {
            self.flush_measure();
        }
    }

    fn with_part_prefix(&self, message: String) -> String {
        match &self.part_name {
            Some(name) => format!("[{name}] {message}"),
            None => message,
        }
    }

    fn push_timed_event(
        &mut self,
        span: Span,
        duration: u32,
        event: NoteEvent,
        overflow_label: &str,
    ) -> Result<(), IrrecoverableError> {
        self.flush_if_full();
        if self.measure_span_start.is_none() {
            self.measure_span_start = Some(span.start);
        }
        self.measure_span_end = span.end;
        self.current_notes.push(event);
        self.current_beat += duration;
        if self.current_beat > self.capacity {
            self.current_notes.pop();
            self.current_beat -= duration;
            let message = self.with_part_prefix(format!(
                "beat overflow: {overflow_label} exceeds measure capacity of {} quarter-beats; note dropped",
                self.capacity,
            ));
            self.pending_overflow_error
                .get_or_insert_with(|| Warning::new(span, message));
            self.flush_measure();
            return Ok(());
        }
        if self.current_beat == self.capacity {
            self.flush_measure();
        }
        Ok(())
    }

    fn handle_extension(&mut self, span: Span) -> Result<(), IrrecoverableError> {
        self.measure_span_end = span.end.max(self.measure_span_end);
        match self.current_notes.last_mut() {
            Some(NoteEvent::Note(n)) => {
                n.duration += 4;
                self.current_beat += 4;
            }
            Some(NoteEvent::Chord(c)) => {
                c.duration += 4;
                self.current_beat += 4;
            }
            Some(NoteEvent::Percussion(p)) => {
                p.duration += 4;
                self.current_beat += 4;
            }
            Some(NoteEvent::Rest(_)) => {
                if self.pending_dash_after_rest_error.is_none() {
                    self.pending_dash_after_rest_error =
                        Some(RecoverableError::dash_after_rest(span));
                }
                return Ok(());
            }
            None => {
                let chord_track = self.part_kind == PartKind::Chords;
                self.pending_extension_no_preceding_event_error
                    .get_or_insert_with(|| {
                        RecoverableError::extension_no_preceding_event(span, chord_track)
                    });
                return Ok(());
            }
        }
        if self.current_beat >= self.capacity {
            self.flush_measure();
        }
        Ok(())
    }

    fn handle_tie_marker(&mut self, _span: Span) -> Result<(), IrrecoverableError> {
        let last_event = self.current_notes.last_mut().or_else(|| {
            self.slots.iter_mut().rev().find_map(|slot| match slot {
                MeasureSlot::Real(m) => m.notes.events.last_mut(),
                MeasureSlot::EmptyNote { .. } => None,
            })
        });
        match last_event {
            Some(NoteEvent::Note(n)) => {
                n.slur = true;
                Ok(())
            }
            Some(NoteEvent::Chord(c)) => {
                c.slur = true;
                Ok(())
            }
            // TieMarker is a legacy event that is never emitted by the parser;
            // this arm is dead code but kept for exhaustiveness.
            _ => Ok(()),
        }
    }

    fn handle_note(&mut self, span: Span, pn: ParsedNote) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            pn.duration,
            NoteEvent::Note(GroupedNote {
                pitch: pn.pitch,
                accidental: pn.accidental,
                octave: pn.octave,
                duration: pn.duration,
                slur: pn.slur && pn.slur_group_close_at_duration.is_none(),
                tie_to_next_span: pn.tie_to_next_span,
                event_span: span,
                group_membership: pn.group_membership,
                group_continuation: pn.group_continuation,
                dotted: pn.dotted,
                slur_group_close_at_duration: pn.slur_group_close_at_duration,
            }),
            "note",
        )
    }

    fn handle_chord(&mut self, span: Span, pc: ParsedChordNote) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            pc.duration,
            NoteEvent::Chord(GroupedChordNote {
                degree: pc.degree,
                accidental: pc.accidental,
                triad: pc.triad,
                extension: pc.extension,
                bass: pc.bass,
                duration: pc.duration,
                slur: pc.slur && pc.slur_group_close_at_duration.is_none(),
                tie_to_next_span: pc.tie_to_next_span,
                event_span: span,
                group_membership: pc.group_membership,
                group_continuation: pc.group_continuation,
                dotted: pc.dotted,
                slur_group_close_at_duration: pc.slur_group_close_at_duration,
            }),
            "chord",
        )
    }

    fn handle_percussion_hit(
        &mut self,
        span: Span,
        ph: &ParsedPercussionHit,
    ) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            ph.duration,
            NoteEvent::Percussion(GroupedPercussionHit {
                duration: ph.duration,
                slur: ph.slur && ph.slur_group_close_at_duration.is_none(),
                tie_to_next_span: ph.tie_to_next_span,
                event_span: span,
                group_membership: ph.group_membership,
                group_continuation: ph.group_continuation,
                dotted: ph.dotted,
                slur_group_close_at_duration: ph.slur_group_close_at_duration,
            }),
            "percussion hit",
        )
    }

    fn handle_rest(&mut self, span: Span, pr: &ParsedRest) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            pr.duration,
            NoteEvent::Rest(GroupedRest {
                duration: pr.duration,
                dotted: pr.dotted,
                group_membership: pr.group_membership,
                group_continuation: pr.group_continuation,
            }),
            "rest",
        )
    }

    fn process_event(
        &mut self,
        spanned: crate::error::Spanned<ScoreEvent>,
    ) -> Result<(), IrrecoverableError> {
        match spanned.value {
            ScoreEvent::BpmChange(_)
            | ScoreEvent::KeyChange(_)
            | ScoreEvent::LabelChange(_)
            | ScoreEvent::DcAlCoda
            | ScoreEvent::ToCoda
            | ScoreEvent::Coda
            | ScoreEvent::Segno
            | ScoreEvent::DsAlCoda
            | ScoreEvent::DcAlFine
            | ScoreEvent::Fine
            | ScoreEvent::DsAlFine => {
                Ok(()) // handled by DirectiveGrouper
            }
            ScoreEvent::TimeSignatureChange {
                numerator,
                denominator,
            } => {
                self.capacity = (numerator as u32) * 16 / (denominator as u32);
                Ok(())
            }
            ScoreEvent::Extension => self.handle_extension(spanned.span),
            ScoreEvent::TieMarker => self.handle_tie_marker(spanned.span),
            ScoreEvent::Note(pn) => self.handle_note(spanned.span, pn),
            ScoreEvent::Chord(pc) => self.handle_chord(spanned.span, pc),
            ScoreEvent::PercussionHit(ph) => self.handle_percussion_hit(spanned.span, &ph),
            ScoreEvent::Rest(pr) => self.handle_rest(spanned.span, &pr),
        }
    }

    fn finish(mut self) -> (Vec<MeasureSlot>, Option<String>, PartKind, Soundfont) {
        if !self.current_notes.is_empty() {
            let source_span =
                Span::new(self.measure_span_start.unwrap_or(0), self.measure_span_end);
            self.slots.push(MeasureSlot::Real(Box::new(GroupedMeasure {
                notes: Notes {
                    events: std::mem::take(&mut self.current_notes),
                },
                source_span,
                group_provenance: None,
                paired_lyrics: Vec::new(),
                lyrics_error: Vec::new(),
                beat_overflow_error: None,
                dash_after_rest_error: self.pending_dash_after_rest_error.take(),
                dotted_eighth_errors: std::mem::take(&mut self.pending_dotted_eighth_errors),
                chord_errors: Vec::new(),
                lex_error: None,
                lyrics_parse_error: None,
                extension_no_preceding_event_error: self
                    .pending_extension_no_preceding_event_error
                    .take(),
            })));
        }

        (self.slots, self.part_name, self.part_kind, self.soundfont)
    }
}
