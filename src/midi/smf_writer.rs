use midly::num::{u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};

use super::{RawEvent, RawKind, TPQ, VELOCITY};

pub(super) fn sort_raw_events(raw: &mut [RawEvent]) {
    raw.sort_by_key(|e| {
        let priority: u8 = match e.kind {
            RawKind::Tempo(_) | RawKind::ProgramChange { .. } | RawKind::ControlChange { .. } => 0,
            RawKind::NoteOff { .. } => 1,
            RawKind::NoteOn { .. } => 2,
        };
        (e.tick, priority)
    });
}

pub(super) fn build_track_events(raw: &[RawEvent]) -> Vec<TrackEvent<'static>> {
    let mut track: Vec<TrackEvent> = Vec::new();
    let mut last_tick: u32 = 0;

    for event in raw {
        let delta = event.tick - last_tick;
        last_tick = event.tick;
        track.push(raw_event_to_track_event(event, delta));
    }

    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    track
}

fn raw_event_to_track_event(event: &RawEvent, delta: u32) -> TrackEvent<'static> {
    match &event.kind {
        RawKind::Tempo(micros) => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(*micros))),
        },
        RawKind::ProgramChange { channel, program } => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Midi {
                channel: u4::from(*channel),
                message: MidiMessage::ProgramChange {
                    program: u7::from(*program),
                },
            },
        },
        RawKind::NoteOn { channel, note } => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Midi {
                channel: u4::from(*channel),
                message: MidiMessage::NoteOn {
                    key: u7::from(*note),
                    vel: u7::from(VELOCITY),
                },
            },
        },
        RawKind::NoteOff { channel, note } => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Midi {
                channel: u4::from(*channel),
                message: MidiMessage::NoteOff {
                    key: u7::from(*note),
                    vel: u7::from(0u8),
                },
            },
        },
        RawKind::ControlChange {
            channel,
            controller,
            value,
        } => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Midi {
                channel: u4::from(*channel),
                message: MidiMessage::Controller {
                    controller: u7::from(*controller),
                    value: u7::from(*value),
                },
            },
        },
    }
}

pub(super) fn write_smf(track: Vec<TrackEvent<'static>>) -> Result<Vec<u8>, IrrecoverableError> {
    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(u15::from(TPQ)),
        },
        tracks: vec![track],
    };

    let mut buf = Vec::new();
    smf.write_std(&mut buf).map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::MidiWriteFailed {
            span: Span::new(0, 0),
        })
    })?;
    Ok(buf)
}
