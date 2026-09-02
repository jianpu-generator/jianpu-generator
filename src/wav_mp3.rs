use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};

use super::SAMPLE_RATE;

/// 128kbps CBR — fixed, no user-facing bitrate knob (see [`super::write_mp3`]).
pub(super) const MP3_BITRATE_KBPS: u32 = 128;

pub(super) fn encode_mp3(l: &[f32], r: &[f32]) -> Result<Vec<u8>, IrrecoverableError> {
    use rusty_mp3::{Mp3Encoder, Mp3EncoderConfig};

    let mut encoder = Mp3Encoder::new(Mp3EncoderConfig {
        bitrate_kbps: MP3_BITRATE_KBPS,
        vbr_quality: None,
    });

    let interleaved: Vec<f32> = l
        .iter()
        .zip(r.iter())
        .flat_map(|(ls, rs)| [ls.clamp(-1.0, 1.0), rs.clamp(-1.0, 1.0)])
        .collect();
    encoder
        .push_pcm_f32(&interleaved, 2, SAMPLE_RATE)
        .map_err(|e| {
            IrrecoverableError::new(IrrecoverableErrorKind::Mp3EncodeFailed {
                span: Span::new(0, 0),
                source: e.to_string(),
            })
        })?;
    encoder.finish();

    let mut buf = Vec::new();
    loop {
        match encoder.next_packet() {
            Ok(packet) => buf.extend_from_slice(&packet),
            Err(rusty_mp3::Error::Eof) => break,
            Err(e) => {
                return Err(IrrecoverableError::new(
                    IrrecoverableErrorKind::Mp3EncodeFailed {
                        span: Span::new(0, 0),
                        source: e.to_string(),
                    },
                ))
            }
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_mp3_starts_with_frame_sync() {
        let l = vec![0.0f32; 44100];
        let r = vec![0.0f32; 44100];
        let bytes = encode_mp3(&l, &r).unwrap();
        assert_eq!(bytes[0], 0xFF);
        assert_eq!(bytes[1] & 0xE0, 0xE0);
    }

    #[test]
    fn encode_mp3_round_trips_through_its_own_decoder() {
        // A sine wave, not silence, so the encoder doesn't degenerate to
        // trivially-zero quantized coefficients.
        let samples = 44100; // 1 second @ 44.1kHz
        let l: Vec<f32> = (0..samples)
            .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / SAMPLE_RATE as f32).sin() * 0.5)
            .collect();
        let r = l.clone();
        let bytes = encode_mp3(&l, &r).unwrap();

        let mut decoder = rusty_mp3::Mp3Decoder::new();
        decoder.push(&bytes);
        decoder.flush();
        let mut decoded_samples = 0usize;
        loop {
            match decoder.next_frame() {
                Ok(frame) => decoded_samples += frame.samples.len() / frame.channels as usize,
                Err(rusty_mp3::Error::Eof) => break,
                Err(e) => panic!("unexpected decode error: {e}"),
            }
        }
        // MP3 frames are fixed-size (1152 samples/channel for MPEG-1), so the
        // decoded length is only ever rounded up to the next frame boundary —
        // check it's in the right ballpark rather than exact.
        assert!(
            decoded_samples >= samples && decoded_samples < samples + 1152 * 2,
            "decoded {decoded_samples} samples, expected close to {samples}"
        );
    }
}
