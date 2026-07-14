use super::*;
use midly::{MidiMessage, Smf, TrackEventKind};

#[cfg(feature = "wav")]
static SF2_BYTES: &[u8] = include_bytes!("../../fonts/GeneralUser_GS.sf2");

fn count_note_on_events(midi_bytes: &[u8]) -> usize {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter(|e| {
            matches!(
                e.kind,
                TrackEventKind::Midi {
                    message: MidiMessage::NoteOn { vel, .. },
                    ..
                } if vel.as_int() > 0
            )
        })
        .count()
}

fn navigation_source() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 4\n",
        "\n",
        "tocoda\n",
        "[Melody] 5 6 7 1'\n",
        "\n",
        "coda\n",
        "[Melody] 1' 7 6 5\n",
        "\n",
        "dcalcoda\n",
        "[Melody] 4 3 2 1\n",
    )
}

fn segno_navigation_source() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 4\n",
        "\n",
        "segno\n",
        "[Melody] 2 3 4 5\n",
        "\n",
        "tocoda\n",
        "[Melody] 5 6 7 1'\n",
        "\n",
        "coda\n",
        "[Melody] 1' 7 6 5\n",
        "\n",
        "dsalcoda\n",
        "[Melody] 4 3 2 1\n",
    )
}

fn plain_source() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 4\n",
        "\n",
        "[Melody] 5 6 7 1'\n",
        "\n",
        "[Melody] 1' 7 6 5\n",
        "\n",
        "[Melody] 4 3 2 1\n",
    )
}

fn dead_zone_source() -> &'static str {
    // Measure 3 lies between `dcalcoda` (measure 2) and `coda` (measure 4),
    // so it's unreachable in playback order (see `expand_navigation`'s
    // `idx = (0..=dc) ∪ (0..=to) ∪ (coda..=last)` construction).
    r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4

tocoda
[Melody] 5 6 7 1'

dcalcoda
[Melody] 2 3 4 5

[Melody] 6 7 1' 2'

coda
[Melody] 1' 7 6 5
"#
}

#[cfg(feature = "wav")]
#[test]
fn play_from_current_measure_after_navigation_includes_repeat() {
    let wav = write_wav_for_measure_range_from_source(
        navigation_source(),
        "test.jianpu",
        0..=3,
        None,
        SF2_BYTES,
        &[],
    )
    .expect("navigation range should generate WAV");

    let full_expanded_wav =
        write_wav_from_source_filtered(navigation_source(), "test.jianpu", None, SF2_BYTES, &[])
            .expect("full navigation score should generate WAV");

    // Both should cover the full expanded playback (8 measures x 4 notes =
    // 32 notes), not just the 4 literally-written measures (16 notes), since
    // "play from measure 0 to the last written measure" should follow the
    // D.C. al Coda repeat instead of stopping at the written end.
    assert_eq!(wav.len(), full_expanded_wav.len());
}

#[cfg(feature = "midi")]
#[test]
fn measure_start_times_for_range_reflects_navigation() {
    let times = measure_start_times_for_range_from_source(
        navigation_source(),
        "test.jianpu",
        0..=3,
        None,
        &[],
    )
    .expect("navigation range should compute measure start times");

    // 8 expanded measures => 9 boundaries, not 4 written measures => 5 boundaries.
    assert_eq!(times.len(), 9);
}

#[cfg(feature = "wav")]
#[test]
fn play_from_dead_zone_measure_falls_back_to_written_order() {
    // Measure 3 has no reachable playback position (see `dead_zone_source`);
    // this must fall back to written order instead of erroring or panicking.
    let result = write_wav_for_measure_range_from_source(
        dead_zone_source(),
        "test.jianpu",
        3..=4,
        None,
        SF2_BYTES,
        &[],
    );
    assert!(
        result.is_ok(),
        "dead-zone start measure must fall back instead of failing"
    );
}

#[cfg(feature = "wav")]
#[test]
fn no_markers_measure_range_playback_unchanged() {
    let with_navigation_helpers = write_wav_for_measure_range_from_source(
        plain_source(),
        "test.jianpu",
        0..=3,
        None,
        SF2_BYTES,
        &[],
    )
    .expect("plain score range should generate WAV");
    let full_plain_wav =
        write_wav_from_source_filtered(plain_source(), "test.jianpu", None, SF2_BYTES, &[])
            .expect("plain score should generate WAV");

    assert_eq!(with_navigation_helpers.len(), full_plain_wav.len());
}

#[test]
fn navigation_markers_replay_measures_in_midi_output() {
    let nav_midi = write_midi_from_source_filtered(navigation_source(), "test.jianpu", None, &[])
        .expect("navigation score should generate MIDI");
    let plain_midi = write_midi_from_source_filtered(plain_source(), "test.jianpu", None, &[])
        .expect("plain score should generate MIDI");

    let nav_notes = count_note_on_events(&nav_midi);
    let plain_notes = count_note_on_events(&plain_midi);

    // Written order: 4 measures x 4 notes = 16 notes.
    // Playback order: measures [0,1,2,3, 0,1, 2,3] = 8 measures x 4 notes = 32 notes.
    assert_eq!(
        plain_notes, 16,
        "plain (written-order) score should play 16 notes"
    );
    assert_eq!(
        nav_notes, 32,
        "D.C. al Coda navigation should replay measures 0-3 then 0-1 then 2-3, doubling the note count"
    );
}

#[test]
fn segno_dsalcoda_markers_replay_measures_in_midi_output() {
    let nav_midi =
        write_midi_from_source_filtered(segno_navigation_source(), "test.jianpu", None, &[])
            .expect("D.S. al Coda navigation score should generate MIDI");

    let nav_notes = count_note_on_events(&nav_midi);

    // Written order: 5 measures x 4 notes = 20 notes.
    // Playback order: measures [0,1,2,3,4, 1,2, 3,4] = 9 measures x 4 notes = 36 notes
    // (pass 1 through dsalcoda at measure 4, pass 2 restarts at segno,
    // measure 1, through tocoda at measure 2, then jumps to coda at measure
    // 3 through the end).
    assert_eq!(
        nav_notes, 36,
        "D.S. al Coda navigation should replay measures 0-4 then 1-2 then 3-4"
    );
}
