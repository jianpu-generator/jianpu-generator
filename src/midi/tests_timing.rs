use super::one_measure_score;
use crate::midi::measure_start_times_seconds;

#[test]
fn measure_start_times_single_note_measure_has_two_boundaries() {
    let score = one_measure_score();
    let times = measure_start_times_seconds(&score).unwrap();
    assert_eq!(
        times.len(),
        2,
        "one measure should produce 2 boundaries (start and end)"
    );
    assert_eq!(times[0], 0.0, "first boundary must be at t=0");
    assert!(
        times[1] > 0.0,
        "measure with a note must have nonzero duration"
    );
}

#[test]
fn measure_start_times_doubling_bpm_halves_duration() {
    let mut fast_score = one_measure_score();
    fast_score.measures[0].bpm = Some(240);
    let slow_times = measure_start_times_seconds(&one_measure_score()).unwrap();
    let fast_times = measure_start_times_seconds(&fast_score).unwrap();
    assert!(
        (fast_times[1] - slow_times[1] / 2.0).abs() < 1e-9,
        "doubling BPM should halve measure duration: slow={}, fast={}",
        slow_times[1],
        fast_times[1]
    );
}
