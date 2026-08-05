use super::*;

const SIMPLE_SOURCE: &str = concat!(
    "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
    "# parts\nMelody = notes\n\n",
    "# score\n[Melody] 1 2 3 4\n\n[Melody] 5 6 7 1\n",
);

const TWO_PART_SOURCE: &str = concat!(
    "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
    "# parts\nSoprano = notes\nAlto = notes\n\n",
    "# score\n[Soprano] 1 2 3 4\n[Alto] 5 6 7 1\n\n[Soprano] 5 6 7 1\n[Alto] 1 2 3 4\n",
);

#[test]
fn extract_unzipped_text_response_returns_ok_with_correct_part_measure_ranges_shape() {
    let response = extract_unzipped_text_response(TWO_PART_SOURCE);
    match response {
        UnzippedEditResponse::Ok {
            text,
            part_measure_ranges,
            lyrics_verse_ranges,
        } => {
            assert!(text.contains("[Soprano]"));
            assert!(text.contains("[Alto]"));
            assert_eq!(part_measure_ranges.len(), 2);
            assert!(
                lyrics_verse_ranges.is_empty(),
                "no Lyrics-role parts declared"
            );
            assert_eq!(part_measure_ranges[0].abbreviation, "Soprano");
            assert_eq!(part_measure_ranges[1].abbreviation, "Alto");
            // Two measures per part.
            assert_eq!(part_measure_ranges[0].ranges.len(), 2);
            assert_eq!(part_measure_ranges[1].ranges.len(), 2);
            // Ranges must slice out the expected measure text.
            let range = &part_measure_ranges[0].ranges[0];
            assert_eq!(&text[range.start..range.end], "1 2 3 4");
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn merge_unzipped_text_response_returns_ok_for_valid_edited_text() {
    let unzipped_text = "[Melody]\n1 2 3 4 5 6 7 1";
    let response = merge_unzipped_text_response(SIMPLE_SOURCE, unzipped_text);
    match response {
        UnzippedEditResponse::Ok {
            text,
            part_measure_ranges,
            lyrics_verse_ranges,
        } => {
            assert!(text.contains("[Melody]"));
            // merge's response never carries ranges (callers re-extract).
            assert!(part_measure_ranges.is_empty());
            assert!(lyrics_verse_ranges.is_empty());
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn merge_unzipped_text_response_returns_unknown_part_for_an_undeclared_header() {
    let unzipped_text = "[NotAPart]\n1 2 3 4";
    let response = merge_unzipped_text_response(SIMPLE_SOURCE, unzipped_text);
    assert!(matches!(response, UnzippedEditResponse::UnknownPart));
}

#[test]
fn merge_unzipped_text_response_returns_err_for_a_malformed_header() {
    let unzipped_text = "Melody\n1 2 3 4";
    let response = merge_unzipped_text_response(SIMPLE_SOURCE, unzipped_text);
    assert!(matches!(response, UnzippedEditResponse::Err));
}

const NOTES_WITH_LYRICS_SOURCE: &str = concat!(
    "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
    "# parts\nMelody = notes+lyrics\n\n",
    "# score\n[Melody] 1 2 3 4\n[Melody] la la la la\n[Melody] da da da da\n\n[Melody] 5 6 7 1\n[Melody] na na na na\n",
);

#[test]
fn extract_unzipped_text_response_flattens_multiple_verses_into_tagged_blocks() {
    let response = extract_unzipped_text_response(NOTES_WITH_LYRICS_SOURCE);
    match response {
        UnzippedEditResponse::Ok {
            text,
            part_measure_ranges,
            lyrics_verse_ranges,
        } => {
            assert!(text.contains("[Melody]"));
            assert!(text.contains("[Melody:lyrics:1]"));
            assert!(text.contains("[Melody:lyrics:2]"));
            assert_eq!(part_measure_ranges.len(), 1);
            assert_eq!(lyrics_verse_ranges.len(), 2);
            assert_eq!(lyrics_verse_ranges[0].abbreviation, "Melody");
            assert_eq!(lyrics_verse_ranges[0].verse_number, 1);
            assert_eq!(lyrics_verse_ranges[1].verse_number, 2);
            let verse1_first = &lyrics_verse_ranges[0].ranges[0];
            assert_eq!(&text[verse1_first.start..verse1_first.end], "la la la la");
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn merge_unzipped_text_response_returns_err_for_a_lyrics_tag_on_a_non_lyrics_part() {
    let unzipped_text = "[Soprano:lyrics:1]\nla la";
    let response = merge_unzipped_text_response(TWO_PART_SOURCE, unzipped_text);
    assert!(matches!(response, UnzippedEditResponse::Err));
}
