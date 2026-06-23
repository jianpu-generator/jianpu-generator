use super::*;

const SOURCE: &str = include_str!("../../postcard.jianpu");

const HEADER: &str = "[metadata]
title = \"Jianpu Postcard\"
author = \"—\"
subtitle = \"test\"

[parts]
Chords (C) = chord
Soprano (S) = notes lyrics
Alto (A) = notes lyrics

[score]
";

#[test]
fn bisect_postcard_sections() {
    let sections: &[(&str, &str)] = &[
        ("scale_degrees", "label=\"Scale degrees\"\nbpm=120 key=C4 time=4/4\n0 - - -\n1 2 3 0\ndo re mi _\n"),
        ("octave", "label=\"Octave\"\n0 - - -\n1' 2' 1, 2,\n_\n"),
        ("duration_eighth", "label=\"Duration eighth\"\n0 - - -\n1_ 1_ 1= 1= 1= 1= 1 -\n_\n"),
        ("duration_dotted", "label=\"Duration dotted\"\n0 - - -\n1. 2_ 1. 2_\n_\n"),
        ("duration_extension", "label=\"Duration extension\"\n5 - - -\n1 - - -\n_\n"),
        ("slur_group", "label=\"Slur group\"\n0 - - -\n(1 2) (3 4)\n_\n"),
        ("nested_slur", "label=\"Nested slur\"\n0 - - -\n((1_ 2_) 3) 4\n_\n"),
        ("cross_measure_slur", "label=\"Cross-measure slur bar 1\"\n0 - - -\n1 2 3 (4\n_\n\n0 - - -\n5) 6 7 0\n_\n"),
        ("chord_major_minor", "label=\"Chord major minor\"\n1 1m 1 1m\n1 - - -\n_\n"),
        ("chord_dim_aug", "label=\"Chord dim aug\"\n1o 1+ 1o 1+\n1 - - -\n_\n"),
        ("chord_dom7", "label=\"Chord dom7\"\n17 4m7 57 1\n1 - - -\n_\n"),
        ("chord_maj7_min7", "label=\"Chord maj7 min7\"\n1M7 4m7 1m7 1\n1 - - -\n_\n"),
        ("chord_slash_bass", "label=\"Chord slash bass\"\n1/5 - 5/7 -\n1 - 5 -\n_\n"),
        ("chord_rest_ext", "label=\"Chord rest ext\"\n0 - 1 -\n1 - 5 -\n_\n"),
        ("directive_bpm_key_time", "label=\"Directive bpm key time\"\nbpm=96 key=G4 time=3/4\n0 - -\n1 2 3\n_\n"),
        ("inline_time_change", "label=\"Inline time change\"\nbpm=120 key=C4\n0 - -\n4/4 1 - - -\n_\n"),
        ("inline_key_change", "label=\"Inline key change\"\n0 - - -\n1=D4 1 2 3 4\n_\n"),
        ("inline_bpm", "label=\"Inline BPM\"\nkey=C4\n0 - - -\nbpm=72 1 - - -\n_\n"),
        ("lyrics_cjk", "label=\"Lyrics CJK\"\nbpm=120 time=4/4\n0 - - -\n1 2 3 4\n\u{6625} \u{5929} \u{6765} \u{4e86}\n"),
        ("lyrics_latin", "label=\"Lyrics Latin\"\n0 - - -\n1 2 3 4\ndo re mi fa\n"),
        ("lyrics_syllable_break", "label=\"Lyrics syllable break\"\n0 - - -\n1_ 2_ 3 4\nhel- lo world !\n"),
        ("lyrics_held", "label=\"Lyrics held\"\n0 - - -\n1 - 2 -\nspring - here -\n"),
        ("lyrics_no_lyrics", "label=\"Lyrics no-lyrics\"\n0 - - -\n1 2 3 4\n_\n"),
        ("ditto", "label=\"Ditto\"\n1 2 3 4\n1 2 3 4\ndo re mi fa\n\"\n\"\n"),
    ];

    // Test each section in isolation first
    for (name, section) in sections {
        let source = format!("{}{}", HEADER, section);
        let result = render_svgs_from_source(&source, "bisect.jianpu");
        assert!(
            result.is_ok(),
            "section '{}' in isolation caused error: {:?}",
            name,
            result.err()
        );
    }

    // Test smallest failing combo
    let scale = sections[0].1;
    let dur_eighth = sections[2].1;

    // Try without Alto part
    let two_part_header = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nChords (C) = chord\nSoprano (S) = notes lyrics\n\n[score]\n";
    let scale_2part = "label=\"Scale degrees\"\nbpm=120 key=C4 time=4/4\n0 - - -\n1 2 3 0\ndo re mi _\n";
    let dur_2part = "label=\"Duration eighth\"\n0 - - -\n1_ 1_ 1= 1= 1= 1= 1 -\n_\n";

    let combo2part = format!("{}{}\n{}\n", two_part_header, scale_2part, dur_2part);
    let result2part = render_svgs_from_source(&combo2part, "bisect.jianpu");
    assert!(result2part.is_ok(), "2-part scale+dur_eighth failed: {:?}", result2part.err());

    let combo3part = format!("{}{}\n{}\n", HEADER, scale, dur_eighth);
    let result3part = render_svgs_from_source(&combo3part, "bisect.jianpu");
    assert!(result3part.is_ok(), "3-part scale+dur_eighth failed: {:?}", result3part.err());

    // Test sections cumulatively
    let mut accumulated = String::from(HEADER);
    for (name, section) in sections {
        accumulated.push_str(section);
        accumulated.push('\n');
        let result = render_svgs_from_source(&accumulated, "bisect.jianpu");
        assert!(
            result.is_ok(),
            "section '{}' in cumulative context caused error: {:?}",
            name,
            result.err()
        );
    }
}

#[test]
fn postcard_renders_without_errors() {
    let output = render_svgs_from_source(SOURCE, "postcard.jianpu")
        .expect("postcard.jianpu must not produce irrecoverable errors");
    assert!(
        !output.svgs.is_empty(),
        "postcard.jianpu must produce at least one SVG page"
    );
    assert!(
        output.diagnostics.is_empty(),
        "postcard.jianpu must produce no diagnostics, got: {:?}",
        output.diagnostics
    );
}
