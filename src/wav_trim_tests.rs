use super::*;

fn ramp(n: usize, value: f32) -> Vec<f32> {
    vec![value; n]
}

#[test]
fn trim_and_fade_keeps_only_the_windowed_samples_plus_release_tail() {
    let total = SAMPLE_RATE as usize; // 1 s of audio
    let mut l = ramp(total, 0.5);
    let mut r = ramp(total, 0.5);
    let trim = TrimWindow {
        start_s: 0.2,
        end_s: 0.4,
        next_note_start_s: None,
    };
    trim_and_fade(&mut l, &mut r, trim);

    let expected_start = (0.2 * SAMPLE_RATE as f64).round() as usize;
    let expected_end =
        ((0.4 * SAMPLE_RATE as f64).round() as usize + TRIM_RELEASE_TAIL_SAMPLES).min(total);
    assert_eq!(l.len(), expected_end - expected_start);
    assert_eq!(r.len(), expected_end - expected_start);
}

#[test]
fn trim_and_fade_clamps_to_the_buffer_length() {
    let total = SAMPLE_RATE as usize / 10; // 100 ms of audio
    let mut l = ramp(total, 0.5);
    let mut r = ramp(total, 0.5);
    // Window well past the end of the buffer plus the release tail margin.
    let trim = TrimWindow {
        start_s: 0.0,
        end_s: 5.0,
        next_note_start_s: None,
    };
    trim_and_fade(&mut l, &mut r, trim);
    assert_eq!(l.len(), total);
    assert_eq!(r.len(), total);
}

#[test]
fn trim_and_fade_is_a_no_op_for_an_inverted_window() {
    let mut l = ramp(1000, 0.5);
    let mut r = ramp(1000, 0.5);
    let trim = TrimWindow {
        start_s: 0.5,
        end_s: 0.1,
        next_note_start_s: None,
    };
    trim_and_fade(&mut l, &mut r, trim);
    assert_eq!(l.len(), 1000);
    assert_eq!(r.len(), 1000);
}

#[test]
fn trim_and_fade_caps_the_release_tail_at_the_next_note_so_it_is_not_audible() {
    // 1 s of audio; selection nominally ends at 0.2 s, but another
    // (unselected) note starts at 0.22 s — well inside the 300 ms release
    // tail. Without the cap, that note's attack would bleed into the clip
    // and be heard as one extra note.
    let total = SAMPLE_RATE as usize;
    let mut l = ramp(total, 0.5);
    let mut r = ramp(total, 0.5);
    let trim = TrimWindow {
        start_s: 0.1,
        end_s: 0.2,
        next_note_start_s: Some(0.22),
    };
    trim_and_fade(&mut l, &mut r, trim);

    let expected_start = (0.1 * SAMPLE_RATE as f64).round() as usize;
    let expected_end = (0.22 * SAMPLE_RATE as f64).round() as usize;
    assert_eq!(l.len(), expected_end - expected_start);
    assert_eq!(r.len(), expected_end - expected_start);
}

#[test]
fn fade_edges_ramps_first_and_last_samples_toward_silence() {
    let mut l = ramp(TRIM_FADE_SAMPLES * 4, 1.0);
    let mut r = ramp(TRIM_FADE_SAMPLES * 4, 1.0);
    fade_edges(&mut l, &mut r);

    assert_eq!(
        l.first().copied(),
        Some(0.0),
        "first sample fades to silence"
    );
    assert_eq!(l.last().copied(), Some(0.0), "last sample fades to silence");
    // Comfortably past the fade window on both ends: untouched.
    let mid = l.len() / 2;
    assert_eq!(l.get(mid).copied(), Some(1.0));
    assert_eq!(r.get(mid).copied(), Some(1.0));
}

#[test]
fn fade_edges_handles_a_clip_shorter_than_the_fade_window() {
    let mut l = ramp(3, 1.0);
    let mut r = ramp(3, 1.0);
    // Must not panic on a clip too short for a full fade in each direction.
    fade_edges(&mut l, &mut r);
    assert_eq!(l.len(), 3);
    assert_eq!(r.len(), 3);
}
