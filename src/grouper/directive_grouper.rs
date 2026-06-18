use crate::ast::grouped::{MeasureDirectives, TimeSignature};
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::error::Spanned;

pub(super) struct DirectiveGrouper {
    current_bpm: u32,
    current_time_sig: TimeSignature,
    current_key: KeyChange,
    bpm_changed: bool,
    time_sig_changed: bool,
    key_changed: bool,
}

impl DirectiveGrouper {
    pub(super) fn new() -> Self {
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

    pub(super) fn process_all(
        mut self,
        directive_events_per_measure: &[Vec<Spanned<ScoreEvent>>],
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
