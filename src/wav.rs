use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use hound::{SampleFormat, WavSpec, WavWriter};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use oxisynth::{MidiEvent, SoundFont, Synth, SynthDescriptor};
use std::io::Cursor;

const SAMPLE_RATE: u32 = 44100;
/// Target peak level before encoding (0.95 ≈ −0.4 dBFS), matching typical mastered music.
const TARGET_PEAK: f32 = 0.95;

fn init_synth(sf2_bytes: &[u8]) -> Result<Synth, IrrecoverableError> {
    let mut synth = Synth::new(SynthDescriptor {
        sample_rate: SAMPLE_RATE as f32,
        ..Default::default()
    })
    .map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavSynthInitFailed {
            span: Span::new(0, 0),
        })
    })?;

    let sf = SoundFont::load(&mut Cursor::new(sf2_bytes)).map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavSoundfontLoadFailed {
            span: Span::new(0, 0),
        })
    })?;
    synth.add_font(sf, true);
    Ok(synth)
}

fn handle_midi_message(synth: &mut Synth, channel: u8, message: &MidiMessage) {
    let ch = channel;
    match message {
        MidiMessage::ProgramChange { program } => {
            synth
                .send_event(MidiEvent::ProgramChange {
                    channel: ch,
                    program_id: program.as_int(),
                })
                .ok();
        }
        MidiMessage::NoteOn { key, vel } => {
            synth
                .send_event(MidiEvent::NoteOn {
                    channel: ch,
                    key: key.as_int(),
                    vel: vel.as_int(),
                })
                .ok();
        }
        MidiMessage::NoteOff { key, .. } => {
            synth
                .send_event(MidiEvent::NoteOff {
                    channel: ch,
                    key: key.as_int(),
                })
                .ok();
        }
        _ => {}
    }
}

fn render_track(
    synth: &mut Synth,
    track: &midly::Track<'_>,
    tpq: u32,
    all_l: &mut Vec<f32>,
    all_r: &mut Vec<f32>,
) {
    let mut micros_per_beat: u32 = 500_000;
    for event in track.iter() {
        let delta = event.delta.as_int();
        if delta > 0 {
            let n = ticks_to_samples(delta, tpq, micros_per_beat);
            render_samples(synth, n, all_l, all_r);
        }
        match &event.kind {
            TrackEventKind::Meta(MetaMessage::Tempo(t)) => {
                micros_per_beat = t.as_int();
            }
            TrackEventKind::Midi { channel, message } => {
                handle_midi_message(synth, channel.as_int(), message);
            }
            _ => {}
        }
    }
}

pub fn write_wav(midi_bytes: &[u8], sf2_bytes: &[u8]) -> Result<Vec<u8>, IrrecoverableError> {
    let smf = Smf::parse(midi_bytes).map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavInvalidMidiBytes {
            span: Span::new(0, 0),
        })
    })?;
    let tpq = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u32,
        Timing::Timecode(..) => 480,
    };

    let mut synth = init_synth(sf2_bytes)?;
    let mut all_l: Vec<f32> = Vec::new();
    let mut all_r: Vec<f32> = Vec::new();

    let track = smf.tracks.first().ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
            Span::new(0, 0),
            "internal invariant: MIDI file has no tracks",
        ))
    })?;
    render_track(&mut synth, track, tpq, &mut all_l, &mut all_r);

    // Render 1 second of tail so reverb fully decays
    render_samples(&mut synth, SAMPLE_RATE as usize, &mut all_l, &mut all_r);

    normalize_peak(&mut all_l, &mut all_r);
    encode_wav(&all_l, &all_r)
}

pub fn write_preview_wav(
    program_number: u8,
    sf2_bytes: &[u8],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut synth = init_synth(sf2_bytes)?;
    let mut all_l = Vec::new();
    let mut all_r = Vec::new();

    // do re mi sol (scale degrees 1 2 3 5 in C major)
    let melody: [u8; 4] = [60, 62, 64, 67];
    let sustain_samples = (SAMPLE_RATE as f32 * 0.35) as usize;
    let gap_samples = (SAMPLE_RATE as f32 * 0.05) as usize;

    synth
        .send_event(MidiEvent::ProgramChange {
            channel: 0,
            program_id: program_number,
        })
        .ok();
    for key in melody {
        synth
            .send_event(MidiEvent::NoteOn {
                channel: 0,
                key,
                vel: 80,
            })
            .ok();
        render_samples(&mut synth, sustain_samples, &mut all_l, &mut all_r);
        synth
            .send_event(MidiEvent::NoteOff { channel: 0, key })
            .ok();
        render_samples(&mut synth, gap_samples, &mut all_l, &mut all_r);
    }
    render_samples(&mut synth, SAMPLE_RATE as usize / 2, &mut all_l, &mut all_r); // 0.5 s tail

    normalize_peak(&mut all_l, &mut all_r);
    encode_wav(&all_l, &all_r)
}

fn normalize_peak(left: &mut [f32], right: &mut [f32]) {
    let peak = left
        .iter()
        .chain(right.iter())
        .map(|sample| sample.abs())
        .fold(0.0f32, f32::max);
    if peak <= 0.0 {
        return;
    }
    let gain = TARGET_PEAK / peak;
    for sample in left.iter_mut().chain(right.iter_mut()) {
        *sample *= gain;
    }
}

fn ticks_to_samples(ticks: u32, tpq: u32, micros_per_beat: u32) -> usize {
    ((ticks as f64 * SAMPLE_RATE as f64 * micros_per_beat as f64) / (tpq as f64 * 1_000_000.0))
        as usize
}

fn render_samples(synth: &mut Synth, n: usize, l: &mut Vec<f32>, r: &mut Vec<f32>) {
    let prev = l.len();
    l.resize(prev + n, 0.0);
    r.resize(prev + n, 0.0);
    let l_tail = l.split_at_mut(prev).1;
    let r_tail = r.split_at_mut(prev).1;
    synth.write_f32(n, l_tail, 0, 1, r_tail, 0, 1);
}

fn encode_wav(l: &[f32], r: &[f32]) -> Result<Vec<u8>, IrrecoverableError> {
    let spec = WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = WavWriter::new(Cursor::new(&mut buf), spec).map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavWriterCreateFailed {
            span: Span::new(0, 0),
            source: e.to_string(),
        })
    })?;
    for (ls, rs) in l.iter().zip(r.iter()) {
        writer
            .write_sample((ls.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::WavWriteSampleFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
        writer
            .write_sample((rs.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::WavWriteSampleFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
    }
    writer.finalize().map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavFinalizeFailed {
            span: Span::new(0, 0),
            source: e.to_string(),
        })
    })?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_to_samples_quarter_note_at_120bpm() {
        // 120 BPM = 500_000 µs/beat, TPQ = 480
        // quarter note = 480 ticks = 0.5 s = 22050 samples @ 44100 Hz
        assert_eq!(ticks_to_samples(480, 480, 500_000), 22050);
    }

    #[test]
    fn ticks_to_samples_half_note_at_120bpm() {
        assert_eq!(ticks_to_samples(960, 480, 500_000), 44100);
    }

    #[test]
    fn encode_wav_has_riff_wave_header() {
        let l = vec![0.0f32; 44100];
        let r = vec![0.0f32; 44100];
        let bytes = encode_wav(&l, &r).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
    }

    #[test]
    fn normalize_peak_scales_to_target() {
        let mut left = vec![0.1f32, -0.2, 0.05];
        let mut right = vec![0.15f32, -0.1, 0.3];
        normalize_peak(&mut left, &mut right);
        let peak = left
            .iter()
            .chain(right.iter())
            .map(|sample| sample.abs())
            .fold(0.0f32, f32::max);
        assert!((peak - TARGET_PEAK).abs() < 1e-6);
    }

    #[test]
    fn normalize_peak_leaves_silence_unchanged() {
        let mut left = vec![0.0f32; 4];
        let mut right = vec![0.0f32; 4];
        normalize_peak(&mut left, &mut right);
        assert!(left.iter().all(|sample| *sample == 0.0));
        assert!(right.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn encode_wav_stereo_16bit_44100() {
        let l = vec![0.0f32; 100];
        let r = vec![0.0f32; 100];
        let bytes = encode_wav(&l, &r).unwrap();
        // WAV spec chunk: channels=2, sample_rate=44100, bits=16
        // bytes 22-23: channels (little-endian u16)
        assert_eq!(u16::from_le_bytes([bytes[22], bytes[23]]), 2);
        // bytes 24-27: sample rate (little-endian u32)
        assert_eq!(
            u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            44100
        );
        // bytes 34-35: bits per sample (little-endian u16)
        assert_eq!(u16::from_le_bytes([bytes[34], bytes[35]]), 16);
    }
}
