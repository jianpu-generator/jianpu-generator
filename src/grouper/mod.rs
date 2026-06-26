use crate::ast::grouped::{
    GroupedChordNote, GroupedMeasure, GroupedNote, GroupedPart, GroupedRest, GroupedScore,
    GroupedTrack, Metadata, NoteEvent, Notes, Score, TimeSignature,
};
use crate::ast::parsed::{
    JianPuPitch, ParsedChordNote, ParsedDocument, ParsedMeasureSlot, ParsedNote, ParsedRest,
    ParsedTimedTrack, ParsedTrack, PartKind, ScoreEvent, Soundfont,
};
use crate::combiner;
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Warning};

#[path = "empty_note_measures.rs"]
mod empty_note_measures;

use empty_note_measures::{align_empty_note_measures, MeasureSlot, PerMeasureErrors};

mod directive_grouper;
mod lyrics_pairing;

use directive_grouper::DirectiveGrouper;
use lyrics_pairing::attach_paired_lyrics;

pub fn group(doc: ParsedDocument) -> Result<Score, IrrecoverableError> {
    let metadata = doc.metadata;
    let document_diagnostics: Vec<Diagnostic> = doc
        .section_structure_errors
        .into_iter()
        .chain(doc.metadata_parse_errors)
        .chain(doc.parts_parse_errors)
        .map(Diagnostic::Error)
        .collect();
    let mut grouped_tracks = Vec::new();
    for track in doc.tracks {
        grouped_tracks.push(match track {
            ParsedTrack::Timed(part) => GroupedTrack::Timed(group_timed_track(part)?),
        });
    }

    let measure_directives = DirectiveGrouper::new().process_all(&doc.directive_events_per_measure);

    let grouped_score = GroupedScore {
        measure_directives,
        parts: grouped_tracks,
        per_measure_parse_errors: doc.per_measure_parse_errors,
    };

    let (measures, combiner_diagnostics) = combiner::combine(&grouped_score);

    let mut score = Score {
        metadata: Metadata {
            title: metadata.title,
            subtitle: metadata.subtitle,
            author: metadata.author,
            row_height: metadata.row_height.unwrap_or(24),
            max_columns: metadata.max_columns.unwrap_or(28),
            label_width: metadata.label_width.unwrap_or(40),
            note_number_width: metadata.note_number_width.unwrap_or(8),
        },
        measures,
        document_diagnostics: document_diagnostics
            .into_iter()
            .chain(combiner_diagnostics)
            .collect(),
    };
    validate_ties(&mut score);
    Ok(score)
}

struct NoteInfo {
    measure_idx: usize,
    event_idx: usize,
    pitch: JianPuPitch,
    octave: i8,
    tie_to_next: bool,
    span: Span,
}

struct TieCorrection {
    measure_idx: usize,
    event_idx: usize,
    error: RecoverableError,
}

fn pitch_to_char(pitch: &JianPuPitch) -> char {
    match pitch {
        JianPuPitch::One => '1',
        JianPuPitch::Two => '2',
        JianPuPitch::Three => '3',
        JianPuPitch::Four => '4',
        JianPuPitch::Five => '5',
        JianPuPitch::Six => '6',
        JianPuPitch::Seven => '7',
    }
}

fn format_pitch_octave(pitch: &JianPuPitch, octave: i8) -> String {
    let ch = pitch_to_char(pitch);
    let octave_suffix = match octave.cmp(&0) {
        std::cmp::Ordering::Greater => "'".repeat(octave as usize),
        std::cmp::Ordering::Less => ",".repeat((-octave) as usize),
        std::cmp::Ordering::Equal => String::new(),
    };
    format!("{ch}{octave_suffix}")
}

fn collect_notes_for_part(score: &Score, part_idx: usize) -> Vec<NoteInfo> {
    score
        .measures
        .iter()
        .enumerate()
        .flat_map(|(measure_idx, measure)| -> Vec<NoteInfo> {
            let Some(part_row) = measure.parts.get(part_idx) else {
                return Vec::new();
            };
            let span = measure.source_span;
            part_row
                .slice()
                .notes
                .events
                .iter()
                .enumerate()
                .filter_map(move |(event_idx, event)| {
                    if let NoteEvent::Note(n) = event {
                        Some(NoteInfo {
                            measure_idx,
                            event_idx,
                            pitch: n.pitch.clone(),
                            octave: n.octave,
                            tie_to_next: n.tie_to_next,
                            span,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

fn tie_corrections(notes: &[NoteInfo]) -> Vec<TieCorrection> {
    notes
        .iter()
        .enumerate()
        .filter(|(_, note)| note.tie_to_next)
        .filter_map(|(i, note)| {
            let next = notes.get(i + 1);
            let error = match next {
                None => Some(RecoverableError::dangling_tie(note.span)),
                Some(next_note)
                    if next_note.pitch != note.pitch || next_note.octave != note.octave =>
                {
                    let expected = format_pitch_octave(&note.pitch, note.octave);
                    let got = format_pitch_octave(&next_note.pitch, next_note.octave);
                    Some(RecoverableError::tie_pitch_mismatch(
                        note.span, expected, got,
                    ))
                }
                Some(_) => None,
            };
            error.map(|err| TieCorrection {
                measure_idx: note.measure_idx,
                event_idx: note.event_idx,
                error: err,
            })
        })
        .collect()
}

fn apply_tie_corrections(score: &mut Score, part_idx: usize, corrections: Vec<TieCorrection>) {
    for correction in corrections {
        if let Some(measure) = score.measures.get_mut(correction.measure_idx) {
            measure
                .diagnostics
                .push(Diagnostic::Error(correction.error));
            if let Some(part_row) = measure.parts.get_mut(part_idx) {
                if let Some(NoteEvent::Note(n)) = part_row
                    .slice_mut()
                    .notes
                    .events
                    .get_mut(correction.event_idx)
                {
                    n.tie_to_next = false;
                }
            }
        }
    }
}

fn validate_ties(score: &mut Score) {
    let num_parts = score.measures.first().map_or(0, |m| m.parts.len());
    for part_idx in 0..num_parts {
        let notes = collect_notes_for_part(score, part_idx);
        let corrections = tie_corrections(&notes);
        apply_tie_corrections(score, part_idx, corrections);
    }
}

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
            paired_lyrics: None,
            lyrics_error: None,
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
                octave: pn.octave,
                duration: pn.duration,
                slur: pn.slur && pn.slur_group_close_at_duration.is_none(),
                tie_to_next: pn.tie_to_next,
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
                group_membership: pc.group_membership,
                group_continuation: pc.group_continuation,
                dotted: pc.dotted,
                slur_group_close_at_duration: pc.slur_group_close_at_duration,
            }),
            "chord",
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
            ScoreEvent::BpmChange(_) | ScoreEvent::KeyChange(_) | ScoreEvent::LabelChange(_) => {
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
                paired_lyrics: None,
                lyrics_error: None,
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

fn group_timed_track(part: ParsedTimedTrack) -> Result<GroupedPart, IrrecoverableError> {
    let lyrics_measure_ends: Vec<usize> = part
        .lyrics
        .as_ref()
        .map(|l| l.measure_ends.clone())
        .unwrap_or_default();
    let lyrics_measure_starts: Vec<usize> = part
        .lyrics
        .as_ref()
        .map(|l| l.measure_starts.clone())
        .unwrap_or_default();
    let measure_syllables = part.lyrics.as_ref().map(|l| l.measure_syllables.clone());
    let per_measure_beat_errors = part.per_measure_beat_errors.clone();
    let per_measure_dotted_eighth_errors = part.per_measure_dotted_eighth_errors.clone();
    let per_measure_dash_after_rest_errors = part.per_measure_dash_after_rest_errors.clone();
    let per_measure_chord_errors = part.per_measure_chord_errors.clone();
    let per_measure_lex_errors = part.per_measure_lex_errors.clone();
    let per_measure_lyrics_errors = part.per_measure_lyrics_errors.clone();
    let part_abbreviation = part.abbreviation.clone();
    let part_kind = part.kind;
    let mut grouper = PartGrouper::new(&part);
    for slot in part.measure_slots {
        match slot {
            ParsedMeasureSlot::EmptyNote { span } => grouper.push_empty_note_slot(span),
            ParsedMeasureSlot::Real { events } => {
                for spanned in events {
                    grouper.process_event(spanned)?;
                }
            }
        }
    }
    let (slots, name, kind, soundfont) = grouper.finish();
    let mut measures = align_empty_note_measures(
        slots,
        &PerMeasureErrors {
            beat_errors: &per_measure_beat_errors,
            dotted_eighth_errors: &per_measure_dotted_eighth_errors,
            dash_after_rest_errors: &per_measure_dash_after_rest_errors,
            chord_errors: &per_measure_chord_errors,
            lex_errors: &per_measure_lex_errors,
            lyrics_errors: &per_measure_lyrics_errors,
        },
    )?;
    for (measure, &lyrics_end) in measures.iter_mut().zip(lyrics_measure_ends.iter()) {
        measure.source_span.end = measure.source_span.end.max(lyrics_end);
    }
    let mut grouped = GroupedPart {
        name,
        kind,
        soundfont,
        measures,
    };
    if matches!(part_kind, PartKind::NotesWithLyrics) {
        let lyrics_spans: Vec<Span> = lyrics_measure_starts
            .iter()
            .zip(lyrics_measure_ends.iter())
            .map(|(&start, &end)| Span::new(start, end))
            .collect();
        attach_paired_lyrics(
            &mut grouped.measures,
            measure_syllables,
            lyrics_spans,
            &part_abbreviation,
        )?;
    }
    Ok(grouped)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests_lyrics.rs"]
mod tests_lyrics;

#[cfg(test)]
#[path = "tests_tie.rs"]
mod tests_tie;
