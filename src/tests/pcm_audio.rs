use super::{
    render_pcm_streaming_for_measure_range_from_source, write_wav_for_measure_range_from_source,
    MeasureRangeStreamingRequest,
};

#[cfg(feature = "wav")]
static SF2_BYTES: &[u8] = include_bytes!("../../fonts/GeneralUser_GS.sf2");

fn two_measure_source() -> &'static str {
    r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4

[Melody] 5 6 7 1
"#
}

fn tied_across_barline_source() -> &'static str {
    r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4~

[Melody] 4 6 7 1
"#
}

struct StreamedChunk {
    measure_offset: usize,
    samples: Vec<f32>,
    is_final: bool,
}

fn stream_measure_range(
    source: &str,
    start_index: usize,
    end_index: usize,
) -> Vec<StreamedChunk> {
    let mut chunks: Vec<StreamedChunk> = Vec::new();
    render_pcm_streaming_for_measure_range_from_source(
        &MeasureRangeStreamingRequest {
            source,
            filename: "test.jianpu",
            measure_range: start_index..=end_index,
            enabled_tracks: None,
            sf2_bytes: SF2_BYTES,
            instruments: &[],
        },
        &mut |measure_offset, samples, is_final| {
            chunks.push(StreamedChunk {
                measure_offset,
                samples: samples.to_vec(),
                is_final,
            });
        },
    )
    .unwrap();
    chunks
}

#[test]
fn streams_one_chunk_per_measure_with_is_final_only_on_last() {
    let source = two_measure_source();
    let chunks = stream_measure_range(source, 0, 1);

    assert_eq!(chunks.len(), 2, "expected one chunk per measure");
    assert_eq!(chunks[0].measure_offset, 0);
    assert!(!chunks[0].is_final);
    assert_eq!(chunks[1].measure_offset, 1);
    assert!(chunks[1].is_final);
}

#[test]
fn streamed_chunks_are_nonempty_interleaved_stereo() {
    let source = two_measure_source();
    let chunks = stream_measure_range(source, 0, 1);

    for chunk in &chunks {
        assert!(!chunk.samples.is_empty());
        assert_eq!(
            chunk.samples.len() % 2,
            0,
            "interleaved [l0, r0, l1, r1, ...] must have an even sample count"
        );
    }
}

#[test]
fn streamed_total_sample_count_matches_whole_range_render() {
    // The whole-range WAV render includes the same 1-second reverb tail as
    // the streamed path's final chunk, so total sample counts should match
    // exactly: the tail is emitted once, on the final chunk only.
    let source = two_measure_source();
    let chunks = stream_measure_range(source, 0, 1);
    let streamed_sample_count: usize = chunks.iter().map(|c| c.samples.len()).sum();

    let wav = write_wav_for_measure_range_from_source(
        source,
        "test.jianpu",
        0..=1,
        None,
        SF2_BYTES,
        &[],
    )
    .unwrap();
    let mut reader = hound::WavReader::new(std::io::Cursor::new(&wav)).unwrap();
    let decoded_sample_count = reader.samples::<i16>().count();

    assert_eq!(
        streamed_sample_count, decoded_sample_count,
        "streamed chunks (with tail on final chunk only) must match a single continuous render exactly"
    );
}

#[test]
fn no_silent_gap_at_barline_for_note_tied_across_measures() {
    // Regression test for the single-synth-no-reset fix: an earlier
    // experimental per-measure render re-initialized the synth on every
    // call, which would silence a note tied across the barline. With one
    // synth for the whole range, the boundary between chunk 0 and chunk 1
    // must not contain a run of near-silence.
    let source = tied_across_barline_source();
    let chunks = stream_measure_range(source, 0, 1);
    assert_eq!(chunks.len(), 2);

    let window = 200usize; // ~2.3ms of stereo frames at 44100 Hz, well under note duration
    let boundary_tail: Vec<f32> = chunks[0]
        .samples
        .iter()
        .rev()
        .take(window)
        .copied()
        .collect();
    let boundary_head: Vec<f32> = chunks[1].samples.iter().take(window).copied().collect();

    let max_abs = boundary_tail
        .iter()
        .chain(boundary_head.iter())
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_abs > 0.001,
        "expected audible signal spanning the barline for a tied note, found near-silence (max abs {max_abs})"
    );
}

#[test]
fn out_of_range_indices_clamp_to_last_measure() {
    let source = two_measure_source();
    let chunks = stream_measure_range(source, 0, 99);
    assert!(!chunks.is_empty());
    assert!(chunks.last().unwrap().is_final);
}
