use crate::ast::grouped::{
    GroupedChordNote, GroupedMeasure, GroupedNote, GroupedPart, GroupedRest, GroupedScore,
    GroupedTrack, MeasureDirectives, Metadata, NoteEvent, Notes, Score, TimeSignature,
};
use crate::ast::parsed::{
    Accidental, JianPuPitch, KeyChange, Note, NoteName, ParsedChordNote, ParsedDocument,
    ParsedNote, ParsedRest, ParsedTimedTrack, ParsedTrack, PartKind, ScoreEvent, Syllable,
};
use crate::combiner;
use crate::error::{JianPuError, Span};

pub fn group(doc: ParsedDocument) -> Result<Score, JianPuError> {
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

struct DirectiveGrouper {
    current_bpm: u32,
    current_time_sig: TimeSignature,
    current_key: KeyChange,
    bpm_changed: bool,
    time_sig_changed: bool,
    key_changed: bool,
}

impl DirectiveGrouper {
    fn new() -> Self {
        Self {
            current_bpm: 120,
            current_time_sig: TimeSignature {
                numerator: 4,
                denominator: 4,
            },
            current_key: KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            },
            bpm_changed: true,
            time_sig_changed: true,
            key_changed: true,
        }
    }

    fn process_all(
        mut self,
        directive_events_per_measure: &[Vec<crate::error::Spanned<ScoreEvent>>],
    ) -> Vec<MeasureDirectives> {
        let mut result = Vec::new();
        for events in directive_events_per_measure {
            let mut pending_label: Option<String> = None;
            for event in events {
                match &event.value {
                    ScoreEvent::BpmChange(bpm) => {
                        self.current_bpm = *bpm;
                        self.bpm_changed = true;
                    }
                    ScoreEvent::TimeSignatureChange {
                        numerator,
                        denominator,
                    } => {
                        self.current_time_sig = TimeSignature {
                            numerator: *numerator,
                            denominator: *denominator,
                        };
                        self.time_sig_changed = true;
                    }
                    ScoreEvent::KeyChange(kc) => {
                        self.current_key = kc.clone();
                        self.key_changed = true;
                    }
                    ScoreEvent::LabelChange(text) => {
                        pending_label = Some(text.clone());
                    }
                    _ => {}
                }
            }
            result.push(MeasureDirectives {
                bpm: if self.bpm_changed {
                    Some(self.current_bpm)
                } else {
                    None
                },
                time_signature: if self.time_sig_changed {
                    Some(TimeSignature {
                        numerator: self.current_time_sig.numerator,
                        denominator: self.current_time_sig.denominator,
                    })
                } else {
                    None
                },
                key: if self.key_changed {
                    Some(self.current_key.clone())
                } else {
                    None
                },
                label: pending_label,
            });
            self.bpm_changed = false;
            self.time_sig_changed = false;
            self.key_changed = false;
        }
        result
    }
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
    ) -> Result<(), JianPuError> {
        self.flush_if_full();
        if self.measure_span_start.is_none() {
            self.measure_span_start = Some(span.start);
        }
        self.measure_span_end = span.end;
        self.current_notes.push(event);
        self.current_beat += duration;
        if self.current_beat > self.capacity {
            return Err(JianPuError::new(
                span,
                self.with_part_prefix(format!(
                    "{overflow_label} duration {duration} overflows the current measure (capacity {} quarter-beats, {} used)",
                    self.capacity, self.current_beat
                )),
            ));
        }
        if self.current_beat == self.capacity {
            self.flush_measure();
        }
        Ok(())
    }

    fn handle_extension(&mut self, span: Span) -> Result<(), JianPuError> {
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
                let mut error = JianPuError::dash_after_rest(span);
                error.message = self.with_part_prefix(error.message);
                return Err(error);
            }
            None => {
                let message = if self.part_kind == PartKind::Chord {
                    "chord extension '-' with no preceding event".to_string()
                } else {
                    "extension `-` without a preceding note or rest; if it follows a measure boundary, cross-measure extension is not supported".to_string()
                };
                return Err(JianPuError::new(span, self.with_part_prefix(message)));
            }
        }
        if self.current_beat >= self.capacity {
            self.flush_measure();
        }
        Ok(())
    }

    fn handle_tie_marker(&mut self, span: Span) -> Result<(), JianPuError> {
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
            _ => Err(JianPuError::new(
                span,
                self.with_part_prefix("tie `~` without a preceding note".to_string()),
            )),
        }
    }

    fn handle_note(&mut self, span: Span, pn: ParsedNote) -> Result<(), JianPuError> {
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

    fn handle_chord(&mut self, span: Span, pc: ParsedChordNote) -> Result<(), JianPuError> {
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

    fn handle_rest(&mut self, span: Span, pr: &ParsedRest) -> Result<(), JianPuError> {
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
    ) -> Result<(), JianPuError> {
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

fn group_timed_track(part: ParsedTimedTrack) -> Result<GroupedPart, JianPuError> {
    let ditto_measures = part.ditto_measures.clone();
    let lyrics_ditto_measures = part.lyrics_ditto_measures.clone();
    let lyrics_measure_ends: Vec<usize> = part
        .lyrics
        .as_ref()
        .map(|l| l.measure_ends.clone())
        .unwrap_or_default();
    let measure_syllables = part.lyrics.as_ref().map(|l| l.measure_syllables.clone());
    let mut grouper = PartGrouper::new(&part);
    for spanned in part.score.events {
        grouper.process_event(spanned)?;
    }
    let mut grouped = grouper.finish();
    grouped.ditto_measures = ditto_measures;
    grouped.lyrics_ditto_measures = lyrics_ditto_measures;
    for (measure, &lyrics_end) in grouped.measures.iter_mut().zip(lyrics_measure_ends.iter()) {
        measure.source_span.end = measure.source_span.end.max(lyrics_end);
    }
    if matches!(part.kind, PartKind::NotesWithLyrics) {
        attach_paired_lyrics(&mut grouped.measures, measure_syllables, &part.abbreviation)?;
    }
    Ok(grouped)
}

/// Pair each measure's raw lyric line to its note lyric slots (tie-aware).
/// Underflow is recovered by padding empty syllables and recording an error.
fn attach_paired_lyrics(
    measures: &mut [GroupedMeasure],
    measure_syllables: Option<Vec<Vec<Syllable>>>,
    part_name: &str,
) -> Result<(), JianPuError> {
    let Some(measure_syllables) = measure_syllables else {
        return Ok(());
    };
    if measure_syllables.len() != measures.len() {
        return Err(JianPuError::new(
            Span::new(0, 0),
            format!(
                "[{part_name}] internal invariant: {} lyric lines but {} grouped measures",
                measure_syllables.len(),
                measures.len()
            ),
        ));
    }
    let mut prev_tie = false;
    let mut prev_pitch: Option<JianPuPitch> = None;
    for (measure, raw_syllables) in measures.iter_mut().zip(measure_syllables) {
        let (paired, error, next_tie, next_pitch) = pair_lyrics_to_notes(
            &measure.notes.events,
            &raw_syllables,
            &measure.source_span,
            prev_tie,
            prev_pitch,
            part_name,
        );
        measure.paired_lyrics = Some(paired);
        measure.lyrics_error = error;
        prev_tie = next_tie;
        prev_pitch = next_pitch;
    }
    Ok(())
}

fn pair_lyrics_to_notes(
    events: &[NoteEvent],
    raw_syllables: &[Syllable],
    source_span: &Span,
    mut prev_tie: bool,
    mut prev_pitch: Option<JianPuPitch>,
    part_name: &str,
) -> (
    Vec<Syllable>,
    Option<JianPuError>,
    bool,
    Option<JianPuPitch>,
) {
    let mut syllable_idx = 0;
    let mut paired = Vec::new();
    let mut underflow_detected = false;

    for event in events {
        match event {
            NoteEvent::Note(note) => {
                let is_continuation = prev_tie && prev_pitch.as_ref() == Some(&note.pitch);
                if !is_continuation {
                    if let Some(syllable) = raw_syllables.get(syllable_idx) {
                        paired.push(syllable.clone());
                        syllable_idx += 1;
                    } else {
                        paired.push(Syllable {
                            text: String::new(),
                            held: false,
                        });
                        underflow_detected = true;
                    }
                }
                prev_tie = note.tie;
                prev_pitch = Some(note.pitch.clone());
            }
            NoteEvent::Rest(_) | NoteEvent::Chord(_) => {
                prev_tie = false;
            }
        }
    }

    let overflow_count = raw_syllables.len().saturating_sub(syllable_idx);
    let error = if underflow_detected {
        Some(JianPuError::new(
            source_span.clone(),
            format!(
                "[{part_name}] lyrics underflow: ran out of syllables at syllable {} (fewer syllables than notes)",
                syllable_idx
            ),
        ))
    } else if overflow_count > 0 {
        Some(JianPuError::new(
            source_span.clone(),
            format!(
                "[{part_name}] lyrics overflow: {} extra syllable{} after all notes are consumed",
                overflow_count,
                if overflow_count == 1 { "" } else { "s" }
            ),
        ))
    } else {
        None
    };

    (paired, error, prev_tie, prev_pitch)
}

#[cfg(test)]
mod tests;
