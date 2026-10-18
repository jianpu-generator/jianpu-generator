use super::lyric_rest::lyric_rest;
use super::rest_shape::{rest_dash_offsets, rest_underline_count};

#[test]
fn underline_count_follows_the_notes_line_rule() {
    assert_eq!(rest_underline_count(1, 1), 2);
    assert_eq!(rest_underline_count(2, 1), 1);
    assert_eq!(rest_underline_count(4, 1), 0);
    assert_eq!(rest_underline_count(6, 3), 1);
}

#[test]
fn dash_offsets_land_on_beat_boundaries() {
    assert_eq!(rest_dash_offsets(8, false, false, 1), vec![4]);
    assert_eq!(rest_dash_offsets(12, false, false, 1), vec![4, 8]);
    assert_eq!(rest_dash_offsets(12, true, false, 1), vec![6]);
    assert!(rest_dash_offsets(4, false, false, 1).is_empty());
}

#[test]
fn lyric_rest_scales_by_the_resolution_multiplier() {
    let rest = lyric_rest("0_", 3).expect("a rest");
    assert_eq!(rest.underline_count, 1);
    assert!(lyric_rest("陪", 3).is_none());
    assert_eq!(lyric_rest("0-", 2).expect("a rest").dash_offsets, vec![8]);
}
