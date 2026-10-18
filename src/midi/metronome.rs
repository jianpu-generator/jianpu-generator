use crate::ast::grouped::{MultiPartMeasure, TimeSignature};

use super::{RawEvent, RawKind, PERCUSSION_CHANNEL, TPQ};

/// General MIDI percussion key for "Metronome Bell", used on the first beat of a measure.
pub(super) const ACCENT_KEY: u8 = 34;
/// General MIDI percussion key for "Metronome Click", used on every other beat.
pub(super) const BEAT_KEY: u8 = 33;

const TICK_LENGTH: u32 = TPQ as u32 / 8;
const DEFAULT_TIME_SIGNATURE_NUMERATOR: u8 = 4;
const DEFAULT_TIME_SIGNATURE_DENOMINATOR: u8 = 4;

/// One click per beat across every measure, accented on the first beat of each measure.
///
/// `measure_start_ticks` holds each measure's start tick plus the end tick of the last
/// measure, so its length is `measures.len() + 1`. The time signature carries forward
/// from the last measure that set one, falling back to 4/4.
pub(super) fn metronome_events(
    measures: &[MultiPartMeasure],
    measure_start_ticks: &[u32],
) -> Vec<RawEvent> {
    let mut active_time_signature: Option<&TimeSignature> = None;
    measures
        .iter()
        .zip(
            measure_start_ticks
                .iter()
                .zip(measure_start_ticks.iter().skip(1)),
        )
        .flat_map(|(measure, (&start_tick, &end_tick))| {
            if let Some(time_signature) = &measure.time_signature {
                active_time_signature = Some(time_signature);
            }
            let beat_ticks = beat_ticks(active_time_signature);
            (start_tick..end_tick)
                .step_by(beat_ticks)
                .enumerate()
                .flat_map(|(beat_index, tick)| {
                    let note = if beat_index == 0 {
                        ACCENT_KEY
                    } else {
                        BEAT_KEY
                    };
                    [
                        RawEvent {
                            tick,
                            kind: RawKind::NoteOn {
                                channel: PERCUSSION_CHANNEL,
                                note,
                            },
                        },
                        RawEvent {
                            tick: tick + TICK_LENGTH,
                            kind: RawKind::NoteOff {
                                channel: PERCUSSION_CHANNEL,
                                note,
                            },
                        },
                    ]
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Length of one beat in ticks. Compound meters (6/8, 9/8, 12/8) beat on the dotted quarter.
fn beat_ticks(time_signature: Option<&TimeSignature>) -> usize {
    let (numerator, denominator) = time_signature.map_or(
        (
            DEFAULT_TIME_SIGNATURE_NUMERATOR,
            DEFAULT_TIME_SIGNATURE_DENOMINATOR,
        ),
        |t| (t.numerator, t.denominator),
    );
    let denominator_unit_ticks = u32::from(TPQ) * 4 / u32::from(denominator.max(1));
    let is_compound = denominator == 8 && numerator > 3 && numerator % 3 == 0;
    let ticks = if is_compound {
        denominator_unit_ticks * 3
    } else {
        denominator_unit_ticks
    };
    ticks.max(1) as usize
}
