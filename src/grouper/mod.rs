use crate::ast::grouped::{
    GroupedChordNote, GroupedMeasure, GroupedNote, GroupedPart, GroupedRest, GroupedScore,
    GroupedTrack, Metadata, NoteEvent, Notes, Score, TimeSignature,
};
use crate::ast::parsed::{
    ParsedChordNote, ParsedDocument, ParsedNote, ParsedRest, ParsedTimedTrack, ParsedTrack,
    PartKind, ScoreEvent,
};
use crate::combiner;
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span, Warning,
};

#[path = "empty_note_measures.rs"]
mod empty_note_measures;

use empty_note_measures::{align_empty_note_measures, PerMeasureErrors};

mod directive_grouper;
mod lyrics_pairing;

use directive_grouper::DirectiveGrouper;
use lyrics_pairing::attach_paired_lyrics;

pub fn group(doc: ParsedDocument) -> Result<Score, IrrecoverableError> {
    let metadata = doc.metadata;
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

    let measures = combiner::combine(&grouped_score)?;

    Ok(Score {
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
    })
}

struct PartGrouper {
    part_kind: PartKind,
    measures: Vec<GroupedMeasure>,
    current_notes: Vec<NoteEvent>,
    current_beat: u32,
    capacity: u32,
    part_name: Option<String>,
    measure_span_start: Option<usize>,
    measure_span_end: usize,
    pending_dash_after_rest_error: Option<RecoverableError>,
    pending_overflow_error: Option<Warning>,
    pending_dotted_eighth_errors: Vec<Diagnostic>,
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
            measures: Vec::new(),
            current_notes: Vec::new(),
            current_beat: 0,
            capacity,
            part_name: Some(part.abbreviation.clone()),
            measure_span_start: None,
            measure_span_end: 0,
            pending_dash_after_rest_error: None,
            pending_overflow_error: None,
            pending_dotted_eighth_errors: Vec::new(),
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
        self.measures.push(GroupedMeasure {
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
        });
        self.current_beat = 0;
        self.measure_span_start = None;
        self.measure_span_end = 0;
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
                return Err(IrrecoverableError::new(
                    IrrecoverableErrorKind::ExtensionNoPrecedingEvent {
                        span,
                        part: self.part_name.clone(),
                        chord_track: self.part_kind == PartKind::Chord,
                    },
                ));
            }
        }
        if self.current_beat >= self.capacity {
            self.flush_measure();
        }
        Ok(())
    }

    fn handle_tie_marker(&mut self, span: Span) -> Result<(), IrrecoverableError> {
        let last_event = self.current_notes.last_mut().or_else(|| {
            self.measures
                .last_mut()
                .and_then(|m| m.notes.events.last_mut())
        });
        match last_event {
            Some(NoteEvent::Note(n)) => {
                n.tie = true;
                Ok(())
            }
            Some(NoteEvent::Chord(c)) => {
                c.tie = true;
                Ok(())
            }
            _ => Err(IrrecoverableError::new(
                IrrecoverableErrorKind::TieNoPrecedingNote {
                    span,
                    part: self.part_name.clone(),
                },
            )),
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
                tie: pn.tie && pn.slur_group_close_at_duration.is_none(),
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
                tie: pc.tie && pc.slur_group_close_at_duration.is_none(),
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

    fn finish(mut self) -> GroupedPart {
        if !self.current_notes.is_empty() {
            let source_span =
                Span::new(self.measure_span_start.unwrap_or(0), self.measure_span_end);
            self.measures.push(GroupedMeasure {
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
            });
        }

        GroupedPart {
            name: self.part_name,
            kind: self.part_kind,
            measures: self.measures,
            ditto_measures: Vec::new(),
            lyrics_ditto_measures: Vec::new(),
        }
    }
}

fn group_timed_track(part: ParsedTimedTrack) -> Result<GroupedPart, IrrecoverableError> {
    let ditto_measures = part.ditto_measures.clone();
    let lyrics_ditto_measures = part.lyrics_ditto_measures.clone();
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
    let empty_note_measure_spans = part.empty_note_measure_spans.clone();
    let mut grouper = PartGrouper::new(&part);
    for spanned in part.score.events {
        grouper.process_event(spanned)?;
    }
    let mut grouped = grouper.finish();
    grouped.ditto_measures = ditto_measures;
    grouped.lyrics_ditto_measures = lyrics_ditto_measures;
    align_empty_note_measures(
        &mut grouped.measures,
        &empty_note_measure_spans,
        &PerMeasureErrors {
            beat_errors: &per_measure_beat_errors,
            dotted_eighth_errors: &per_measure_dotted_eighth_errors,
            dash_after_rest_errors: &per_measure_dash_after_rest_errors,
            chord_errors: &per_measure_chord_errors,
            lex_errors: &per_measure_lex_errors,
        },
    )?;
    for (measure, &lyrics_end) in grouped.measures.iter_mut().zip(lyrics_measure_ends.iter()) {
        measure.source_span.end = measure.source_span.end.max(lyrics_end);
    }
    if matches!(
        part.kind,
        PartKind::NotesWithLyrics | PartKind::LyricsWithNotes
    ) {
        let lyrics_spans: Vec<Span> = lyrics_measure_starts
            .iter()
            .zip(lyrics_measure_ends.iter())
            .map(|(&start, &end)| Span::new(start, end))
            .collect();
        attach_paired_lyrics(
            &mut grouped.measures,
            measure_syllables,
            lyrics_spans,
            &part.abbreviation,
        )?;
    }
    Ok(grouped)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests_lyrics.rs"]
mod tests_lyrics;
