use super::*;
use midly::{MidiMessage, Smf, TrackEventKind};

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
