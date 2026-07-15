use super::*;

fn realistic_coda_navigation_source() -> &'static str {
    // Mirrors reference.jianpu's real marker order: segno < tocoda < dsalcoda
    // < coda, i.e. the coda section (and thus the score's literal last
    // written measure) comes *after* the dsalcoda mark, not before it. This
    // is the common real-world layout: the coda section is appended after
    // the main tune, so its last measure is also the score's last measure.
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
        "dsalcoda\n",
        "[Melody] 4 3 2 1\n",
        "\n",
        "coda\n",
        "[Melody] 1' 7 6 5\n",
    )
}

#[cfg(feature = "wav")]
#[test]
fn play_current_measure_on_final_coda_measure_plays_one_measure() {
    // The `coda` measure here is both a navigation marker measure *and* the
    // score's literal last written measure (matching reference.jianpu's real
    // layout). Selecting exactly that measure as "current measure"
    // (extend_to_last_occurrence: false) must play only that one measure.
    let wav = write_wav_for_measure_range_from_source(
        realistic_coda_navigation_source(),
        "test.jianpu",
        &MeasureRangeSelection {
            range: 4..=4,
            extend_to_last_occurrence: false,
        },
        None,
        SF2_BYTES,
        &[],
    )
    .expect("single-measure range on the final coda measure should generate WAV");

    // plain_source has 4 measures (indices 0-3), so its last measure (3) is
    // the single-measure baseline for a written measure's audio length.
    let single_measure_wav = write_wav_for_measure_range_from_source(
        plain_source(),
        "test.jianpu",
        &MeasureRangeSelection {
            range: 3..=3,
            extend_to_last_occurrence: false,
        },
        None,
        SF2_BYTES,
        &[],
    )
    .expect("single-measure range on plain score should generate WAV");

    assert_eq!(
        wav.len(),
        single_measure_wav.len(),
        "selecting the final coda measure alone must not overrun into an earlier repeat pass"
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
