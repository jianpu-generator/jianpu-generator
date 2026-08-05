//! Tests for [`super::format_unzipped_text`]: each measure lands on its own
//! line, the merge/re-extract round-trip is byte-consistent with a plain
//! `merge_unzipped_text` + `extract_unzipped_text` pair, and errors propagate
//! exactly like [`super::merge_unzipped_text`]'s.

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

const LYRICS_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Soprano = notes+lyrics

# score
[Soprano] 1 2 3 4
[Soprano] la la la la
[Soprano] da da da da

[Soprano] 5 6 7 1
[Soprano] la la la la
[Soprano] da da da da
";

#[test]
fn each_measure_lands_on_its_own_line_within_a_block() {
    let unzipped = extract_unzipped_text(THREE_PART_SOURCE).unwrap().text;
    let formatted = format_unzipped_text(THREE_PART_SOURCE, &unzipped).unwrap();

    assert!(formatted.text.contains("[Soprano]\n1 2 3 4\n5 6 7 1"));
    assert!(formatted.text.contains("[Alto]\n5 6 7 1\n1 2 3 4"));
    assert!(formatted.text.contains("[Bass]\n3 3 3 3\n6 6 6 6"));
}

#[test]
fn formatting_is_a_no_op_on_the_merged_back_source() {
    let unzipped = extract_unzipped_text(THREE_PART_SOURCE).unwrap().text;
    let formatted = format_unzipped_text(THREE_PART_SOURCE, &unzipped).unwrap();

    let merged_from_unformatted = merge_unzipped_text(THREE_PART_SOURCE, &unzipped).unwrap();
    let merged_from_formatted = merge_unzipped_text(THREE_PART_SOURCE, &formatted.text).unwrap();
    assert_eq!(merged_from_unformatted, merged_from_formatted);
}

#[test]
fn measure_ranges_slice_out_the_same_text_as_a_space_joined_extraction() {
    let unzipped = extract_unzipped_text(THREE_PART_SOURCE).unwrap().text;
    let formatted = format_unzipped_text(THREE_PART_SOURCE, &unzipped).unwrap();

    let soprano_ranges = &formatted.part_measure_ranges[0];
    assert_eq!(soprano_ranges.len(), 2);
    let (start, end) = soprano_ranges[0];
    assert_eq!(&formatted.text[start..end], "1 2 3 4");
    let (start, end) = soprano_ranges[1];
    assert_eq!(&formatted.text[start..end], "5 6 7 1");
}

#[test]
fn lyrics_verse_blocks_also_get_one_measure_per_line() {
    let unzipped = extract_unzipped_text(LYRICS_SOURCE).unwrap().text;
    let formatted = format_unzipped_text(LYRICS_SOURCE, &unzipped).unwrap();

    assert!(formatted
        .text
        .contains("[Soprano:lyrics:2]\nda da da da\nda da da da"));
}

#[test]
fn unknown_part_header_errors_like_merge_unzipped_text() {
    let result = format_unzipped_text(THREE_PART_SOURCE, "[NotAPart]\n1 2 3 4");
    assert_eq!(result.unwrap_err(), UnzippedEditError::UnknownPart);
}
