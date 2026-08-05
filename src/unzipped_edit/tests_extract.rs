//! Tests for [`super::extract_unzipped_text`]: multi-part flattening into
//! `[Abbrev]`-headed blocks, and byte-range correctness of
//! `part_measure_ranges` against the emitted `text`.

use super::*;

const THREE_PART_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Soprano = notes
Alto = notes
Bass = notes

# score
[Soprano] 1 2 3 4
[Alto] 5 6 7 1
[Bass] 3 3 3 3

[Soprano] 5 6 7 1
[Alto] 1 2 3 4
[Bass] 6 6 6 6
";

#[test]
fn extract_emits_one_bracketed_block_per_declared_part_in_declaration_order() {
    let output = extract_unzipped_text(THREE_PART_SOURCE).unwrap();

    assert_eq!(output.text.matches("[Soprano]").count(), 1);
    assert_eq!(output.text.matches("[Alto]").count(), 1);
    assert_eq!(output.text.matches("[Bass]").count(), 1);

    let soprano_pos = output.text.find("[Soprano]").unwrap();
    let alto_pos = output.text.find("[Alto]").unwrap();
    let bass_pos = output.text.find("[Bass]").unwrap();
    assert!(soprano_pos < alto_pos, "Soprano block should come first");
    assert!(alto_pos < bass_pos, "Alto block should come before Bass");

    assert_eq!(output.part_measure_ranges.len(), 3);
}

#[test]
fn extract_omits_the_trailing_blank_line_separator_after_the_last_part() {
    let output = extract_unzipped_text(THREE_PART_SOURCE).unwrap();
    assert!(
        !output.text.ends_with("\n\n"),
        "no separator should follow the last part's block: {:?}",
        output.text
    );
}

#[test]
fn part_measure_ranges_byte_offsets_slice_out_the_correct_measure_text() {
    let output = extract_unzipped_text(THREE_PART_SOURCE).unwrap();

    // Declaration order: Soprano = 0, Alto = 1, Bass = 2.
    let soprano_ranges = &output.part_measure_ranges[0];
    assert_eq!(soprano_ranges.len(), 2);
    let (start, end) = soprano_ranges[0];
    assert_eq!(&output.text[start..end], "1 2 3 4");
    let (start, end) = soprano_ranges[1];
    assert_eq!(&output.text[start..end], "5 6 7 1");

    let alto_ranges = &output.part_measure_ranges[1];
    let (start, end) = alto_ranges[0];
    assert_eq!(&output.text[start..end], "5 6 7 1");
    let (start, end) = alto_ranges[1];
    assert_eq!(&output.text[start..end], "1 2 3 4");

    let bass_ranges = &output.part_measure_ranges[2];
    let (start, end) = bass_ranges[0];
    assert_eq!(&output.text[start..end], "3 3 3 3");
    let (start, end) = bass_ranges[1];
    assert_eq!(&output.text[start..end], "6 6 6 6");
}

#[test]
fn measure_line_breaks_within_a_part_block_become_single_spaces() {
    let output = extract_unzipped_text(THREE_PART_SOURCE).unwrap();
    let alto_pos = output.text.find("[Alto]").unwrap();
    let bass_pos = output.text.find("[Bass]").unwrap();
    let alto_block = &output.text[alto_pos..bass_pos];
    // The two Alto measures ("5 6 7 1" and "1 2 3 4") must be joined by
    // exactly one space, not a blank-line measure separator.
    assert!(
        alto_block.contains("5 6 7 1 1 2 3 4"),
        "expected single-space-joined measures, got: {alto_block:?}"
    );
}

#[test]
fn a_part_with_no_content_in_a_measure_still_gets_an_empty_range() {
    let source = "\
# metadata
title = \"Test\"

# parts
Melody = notes
Harmony = notes

# score
[Melody] 1 2 3 4

[Melody] 5 6 7 1
[Harmony] 1 1 1 1
";
    let output = extract_unzipped_text(source).unwrap();
    // Melody is declaration index 0, Harmony is index 1.
    let harmony_ranges = &output.part_measure_ranges[1];
    assert_eq!(harmony_ranges.len(), 2);
    // Harmony wasn't mentioned in the first measure; desugaring implicit-fills
    // it with rests, so its range is non-empty rest content, not literally "".
    let (start, end) = harmony_ranges[0];
    assert_eq!(&output.text[start..end], "0 0 0 0");
}

const NOTES_WITH_LYRICS_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Melody = notes+lyrics

# score
[Melody] 1 2 3 4
[Melody] la la la la
[Melody] da da da da

[Melody] 5 6 7 1
";

fn find_verse(
    output: &UnzippedExtractOutput,
    part_index: usize,
    verse_number: usize,
) -> &LyricsVerseRanges {
    output.lyrics_verse_ranges[part_index]
        .iter()
        .find(|verse| verse.verse_number == verse_number)
        .unwrap_or_else(|| panic!("verse {verse_number} not found for part {part_index}"))
}

#[test]
fn a_notes_with_lyrics_part_emits_one_tagged_block_per_verse_beyond_the_primary_notes_block() {
    let output = extract_unzipped_text(NOTES_WITH_LYRICS_SOURCE).unwrap();

    assert_eq!(output.text.matches("[Melody]").count(), 1);
    assert_eq!(output.text.matches("[Melody:lyrics:1]").count(), 1);
    assert_eq!(output.text.matches("[Melody:lyrics:2]").count(), 1);
    assert_eq!(output.lyrics_verse_ranges[0].len(), 2);
}

#[test]
fn notes_with_lyrics_verse_ranges_slice_out_the_correct_measure_text() {
    let output = extract_unzipped_text(NOTES_WITH_LYRICS_SOURCE).unwrap();

    let (start, end) = output.part_measure_ranges[0][0];
    assert_eq!(&output.text[start..end], "1 2 3 4");

    let verse1 = find_verse(&output, 0, 1);
    let (start, end) = verse1.measure_ranges[0];
    assert_eq!(&output.text[start..end], "la la la la");

    let verse2 = find_verse(&output, 0, 2);
    let (start, end) = verse2.measure_ranges[0];
    assert_eq!(&output.text[start..end], "da da da da");
}

#[test]
fn a_lower_verse_missing_from_a_measure_is_implicit_filled_but_a_higher_missing_verse_is_empty() {
    // Measure 0 writes 2 verses; measure 1 writes none at all (just notes).
    // `NotesWithLyrics` always guarantees at least one Lyrics slot, so verse 1
    // is implicit-filled with `_` for measure 1 — but verse 2 has no slot to
    // occupy there at all, so its range is empty, not `_`.
    let output = extract_unzipped_text(NOTES_WITH_LYRICS_SOURCE).unwrap();

    let verse1 = find_verse(&output, 0, 1);
    assert_eq!(verse1.measure_ranges.len(), 2);
    let (start, end) = verse1.measure_ranges[1];
    assert_eq!(&output.text[start..end], "_");

    let verse2 = find_verse(&output, 0, 2);
    assert_eq!(verse2.measure_ranges.len(), 2);
    let (start, end) = verse2.measure_ranges[1];
    assert_eq!(&output.text[start..end], "");
}

const STANDALONE_LYRICS_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Words = lyrics

# score
[Words] la la
[Words] da da

[Words] ya ya
";

#[test]
fn a_standalone_lyrics_part_folds_verse_one_into_the_primary_block_and_tags_the_rest() {
    let output = extract_unzipped_text(STANDALONE_LYRICS_SOURCE).unwrap();

    // Verse 1 is the primary `[Words]` block, not tagged.
    assert_eq!(output.text.matches("[Words:lyrics:1]").count(), 0);
    assert_eq!(output.text.matches("[Words:lyrics:2]").count(), 1);
    assert_eq!(output.lyrics_verse_ranges[0].len(), 1);
    assert_eq!(output.lyrics_verse_ranges[0][0].verse_number, 2);

    let (start, end) = output.part_measure_ranges[0][0];
    assert_eq!(&output.text[start..end], "la la");

    let verse2 = find_verse(&output, 0, 2);
    let (start, end) = verse2.measure_ranges[0];
    assert_eq!(&output.text[start..end], "da da");
    // Measure 1 only wrote verse 1 ("ya ya"); verse 2 has no slot there.
    let (start, end) = verse2.measure_ranges[1];
    assert_eq!(&output.text[start..end], "");
}
