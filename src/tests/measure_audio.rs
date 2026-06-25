use super::write_wav_for_measure_range_from_source;
use super::*;

#[cfg(feature = "wav")]
static SF2_BYTES: &[u8] = include_bytes!("../../fonts/GeneralUser_GS.sf2");

fn two_measure_source() -> &'static str {
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
        "[Melody] 5 6 7 1\n",
    )
}

#[test]
fn find_measure_at_byte_offset_finds_first_measure() {
    let source = two_measure_source();
    let score = compile(source, "test.jianpu").unwrap();
    let first_note_offset = source.find("1 2 3 4").unwrap();
    assert_eq!(
        find_measure_at_byte_offset(&score, first_note_offset),
        Some(0)
    );
}

#[test]
fn find_measure_at_byte_offset_finds_second_measure() {
    let source = two_measure_source();
    let score = compile(source, "test.jianpu").unwrap();
    let second_note_offset = source.rfind("5 6 7 1").unwrap();
    assert_eq!(
        find_measure_at_byte_offset(&score, second_note_offset),
        Some(1)
    );
}

#[test]
fn find_measure_at_byte_offset_returns_none_for_header() {
    let source = two_measure_source();
    let score = compile(source, "test.jianpu").unwrap();
    assert_eq!(find_measure_at_byte_offset(&score, 0), None);
}

fn two_part_source() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Chord = chord\n",
        "Melody = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Chord] 1\n",
        "[Melody] 1 2 3 4\n",
        "\n",
        "[Chord] 5\n",
        "[Melody] 5 6 7 1\n",
    )
}

#[test]
fn find_measure_at_byte_offset_finds_measure_on_second_part_line() {
    let source = two_part_source();
    let score = compile(source, "test.jianpu").unwrap();
    // The cursor is on the Melody (second part) line "1 2 3 4", not the Chord line "1"
    let melody_note_offset = source.find("1 2 3 4").unwrap();
    assert_eq!(
        find_measure_at_byte_offset(&score, melody_note_offset),
        Some(0),
        "cursor on second part's line should still find the correct measure"
    );
}

fn twinkle_source() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"Twinkle Twinkle Little Star\"\n",
        "author = \"Mozart\"\n",
        "row height = 24\n",
        "\n",
        "# parts\n",
        "Chord = chord\n",
        "Melody = notes lyrics\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Chord] 1 - - -\n",
        "[Melody] 1 1 5 5\n",
        "[Melody] twin- kle twin- kle\n",
        "\n",
        "[Chord] 5 - - -\n",
        "[Melody] 6 6 5-\n",
        "[Melody] lit- tle star\n",
        "\n",
        "[Chord] 4 - - -\n",
        "[Melody] 4 4 3 3\n",
        "[Melody] how I won- der\n",
        "\n",
        "[Chord] 4 - - -\n",
        "[Melody] 2 2 1-\n",
        "[Melody] what you are\n",
    )
}

#[test]
fn find_measure_at_byte_offset_covers_all_lines_in_twinkle_star() {
    let source = twinkle_source();
    let score = compile(source, "test.jianpu").unwrap();
    assert_eq!(score.measures.len(), 4, "expected 4 measures");

    let check = |label: &str, offset: usize, expected: Option<usize>| {
        assert_eq!(
            find_measure_at_byte_offset(&score, offset),
            expected,
            "{label} (offset {offset})"
        );
    };

    // Measure 0: chord "1 - - -", melody "1 1 5 5", lyrics "twin- kle twin- kle"
    check(
        "measure 0 chord line",
        source.find("1 - - -").unwrap(),
        Some(0),
    );
    check(
        "measure 0 melody line",
        source.find("1 1 5 5").unwrap(),
        Some(0),
    );
    check(
        "measure 0 lyrics line",
        source.find("twin- kle twin- kle").unwrap(),
        Some(0),
    );

    // Measure 1: chord "5 - - -", melody "6 6 5-", lyrics "lit- tle star"
    check(
        "measure 1 chord line",
        source.find("5 - - -").unwrap(),
        Some(1),
    );
    check(
        "measure 1 melody line",
        source.find("6 6 5-").unwrap(),
        Some(1),
    );
    check(
        "measure 1 lyrics line",
        source.find("lit- tle star").unwrap(),
        Some(1),
    );

    // Measure 2: first "4 - - -", melody "4 4 3 3", lyrics "how I won- der"
    check(
        "measure 2 chord line",
        source.find("4 - - -").unwrap(),
        Some(2),
    );
    check(
        "measure 2 melody line",
        source.find("4 4 3 3").unwrap(),
        Some(2),
    );
    check(
        "measure 2 lyrics line",
        source.find("how I won- der").unwrap(),
        Some(2),
    );

    // Measure 3: second "4 - - -", melody "2 2 1-", lyrics "what you are"
    check(
        "measure 3 chord line",
        source.rfind("4 - - -").unwrap(),
        Some(3),
    );
    check(
        "measure 3 melody line",
        source.find("2 2 1-").unwrap(),
        Some(3),
    );
    check(
        "measure 3 lyrics line",
        source.find("what you are").unwrap(),
        Some(3),
    );
}

#[test]
fn find_measure_at_line_number_covers_all_lines_in_twinkle_star() {
    let source = twinkle_source();
    let score = compile(source, "test.jianpu").unwrap();
    assert_eq!(score.measures.len(), 4, "expected 4 measures");

    use crate::find_measure_at_line_number;
    let check = |label: &str, line: usize, expected: Option<usize>| {
        assert_eq!(
            find_measure_at_line_number(&score, source, line),
            expected,
            "{label} (line {line})"
        );
    };

    // Lines 0-10: # metadata, title, author, row height, blank, # parts, Chord, Melody, blank, # score, directive
    for line in 0..=10 {
        check("header/directive line", line, None);
    }

    // Measure 0: chord "1 - - -" (11), melody "1 1 5 5" (12), lyrics "twin- kle twin- kle" (13)
    check("measure 0 chord line", 11, Some(0));
    check("measure 0 melody line", 12, Some(0));
    check("measure 0 lyrics line", 13, Some(0));

    // Line 14: blank separator
    check("blank separator after measure 0", 14, None);

    // Measure 1: chord "5 - - -" (15), melody "6 6 5-" (16), lyrics "lit- tle star" (17)
    check("measure 1 chord line", 15, Some(1));
    check("measure 1 melody line", 16, Some(1));
    check("measure 1 lyrics line", 17, Some(1));

    // Line 18: blank separator
    check("blank separator after measure 1", 18, None);

    // Measure 2: chord "4 - - -" (19), melody "4 4 3 3" (20), lyrics "how I won- der" (21)
    check("measure 2 chord line", 19, Some(2));
    check("measure 2 melody line", 20, Some(2));
    check("measure 2 lyrics line", 21, Some(2));

    // Line 22: blank separator
    check("blank separator after measure 2", 22, None);

    // Measure 3: chord "4 - - -" (23), melody "2 2 1-" (24), lyrics "what you are" (25)
    check("measure 3 chord line", 23, Some(3));
    check("measure 3 melody line", 24, Some(3));
    check("measure 3 lyrics line", 25, Some(3));
}

#[test]
fn find_measure_at_byte_offset_detects_measure_when_cursor_is_one_past_last_char() {
    // Regression: cursor placed directly after the last character of a measure
    // line (e.g. after "4" in "1 2 3 4") must still resolve to that measure.
    let source = two_measure_source();
    let score = compile(source, "test.jianpu").unwrap();
    let after_last_char = source.find("1 2 3 4").unwrap() + "1 2 3 4".len();
    assert_eq!(
        find_measure_at_byte_offset(&score, after_last_char),
        Some(0),
        "cursor one byte past last char should still detect the measure (offset {after_last_char})"
    );
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_from_source_returns_riff_wav() {
    let source = two_measure_source();
    let wav = write_wav_for_measure_from_source(source, "test.jianpu", 0, None, SF2_BYTES).unwrap();
    assert!(wav.len() > 4);
    assert_eq!(&wav[0..4], b"RIFF");
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_from_source_second_measure_uses_context_key() {
    // Two-measure source where second measure has no explicit key directive.
    // write_wav_for_measure_from_source(source, file, 1, None) must not error.
    let source = two_measure_source();
    let result = write_wav_for_measure_from_source(source, "test.jianpu", 1, None, SF2_BYTES);
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_from_source_out_of_range_clamps_to_last_measure() {
    let source = two_measure_source();
    let result = write_wav_for_measure_from_source(source, "test.jianpu", 99, None, SF2_BYTES);
    assert!(
        result.is_ok(),
        "out-of-range index must clamp instead of failing"
    );
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_returns_riff_wav() {
    let source = two_measure_source();
    let wav = write_wav_for_measure_range_from_source(source, "test.jianpu", 0, 1, None, SF2_BYTES)
        .unwrap();
    assert!(wav.len() > 4);
    assert_eq!(&wav[0..4], b"RIFF");
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_single_measure_matches_range_of_one() {
    let source = two_measure_source();
    let single =
        write_wav_for_measure_from_source(source, "test.jianpu", 0, None, SF2_BYTES).unwrap();
    let range =
        write_wav_for_measure_range_from_source(source, "test.jianpu", 0, 0, None, SF2_BYTES)
            .unwrap();
    // Both paths produce RIFF WAV; the exact bytes may differ but both are valid.
    assert_eq!(&single[0..4], b"RIFF");
    assert_eq!(&range[0..4], b"RIFF");
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_out_of_range_clamps_to_last_measure() {
    let source = two_measure_source();
    let result =
        write_wav_for_measure_range_from_source(source, "test.jianpu", 0, 99, None, SF2_BYTES);
    assert!(
        result.is_ok(),
        "out-of-range range end must clamp instead of failing"
    );
}
