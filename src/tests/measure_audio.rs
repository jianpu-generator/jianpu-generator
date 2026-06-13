use super::*;

fn two_measure_source() -> &'static str {
    concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Melody = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "\n",
        "5 6 7 1\n",
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

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_from_source_returns_riff_wav() {
    let source = two_measure_source();
    let wav = write_wav_for_measure_from_source(source, "test.jianpu", 0, None).unwrap();
    assert!(wav.len() > 4);
    assert_eq!(&wav[0..4], b"RIFF");
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_from_source_second_measure_uses_context_key() {
    // Two-measure source where second measure has no explicit key directive.
    // write_wav_for_measure_from_source(source, file, 1, None) must not error.
    let source = two_measure_source();
    let result = write_wav_for_measure_from_source(source, "test.jianpu", 1, None);
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_from_source_out_of_range_returns_err() {
    let source = two_measure_source();
    let result = write_wav_for_measure_from_source(source, "test.jianpu", 99, None);
    assert!(result.is_err());
}
