//! A lyric part that has no syllables anywhere in a system must not take up
//! a (blank) row there. Reproduces 把根留住, where `L1`/`L2` are only used in
//! sections A/B and `L3` only in C/D, yet every system showed all three rows.

use crate::compiler::compile;
use crate::consolidator::consolidate;
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::{GridContent, Header};
use crate::grouper::group;
use crate::parser::parse;
use crate::render_config::RenderConfig;

/// For every laid-out row that contains a lyric row label, the number of
/// `LyricSyllable`s in it.
fn lyric_row_syllable_counts(source: &str) -> Vec<usize> {
    let document = parse(source, "test", &[]).unwrap();
    let score = group(document).unwrap();
    let config = RenderConfig::from_metadata(&score.metadata);
    let compile_result = consolidate(compile(&score));
    let header = Header {
        parts_list_columns: 3,
        ..Default::default()
    };
    let output = layout(&compile_result, &config, &header, 2000.0, 4000.0, None);
    output
        .pages
        .iter()
        .flat_map(|page| page.rows.iter())
        .filter(|row| {
            row.elements
                .iter()
                .any(|el| matches!(&el.content, GridContent::RowLabel(t) if t.starts_with('L')))
        })
        .map(|row| {
            row.elements
                .iter()
                .filter(|el| matches!(el.content, GridContent::LyricSyllable { .. }))
                .count()
        })
        .collect()
}

#[test]
fn lyric_part_without_syllables_in_a_system_has_no_row() {
    let source = concat!(
        "# parts\n",
        "Bass [B] = notes\n",
        "Lyrics 1 [L1] = lyrics[B]\n",
        "Lyrics 2 [L2] = lyrics[B]\n",
        "Lyrics 3 [L3] = lyrics[B]\n",
        "\n",
        "# score\n",
        "[B] 1 2 3\n",
        "[L1]a b c\n",
    );
    assert_eq!(lyric_row_syllable_counts(source), vec![3]);
}

#[test]
fn lyric_parts_used_in_different_measures_get_no_blank_rows() {
    // Measure 1 only sings verse L1, measure 2 only verse L3, in one system.
    let source = concat!(
        "# parts\n",
        "Bass [B] = notes\n",
        "Lyrics 1 [L1] = lyrics[B]\n",
        "Lyrics 2 [L2] = lyrics[B]\n",
        "Lyrics 3 [L3] = lyrics[B]\n",
        "\n",
        "# score\n",
        "[B] 1 2 3\n",
        "[L1]a b c\n",
        "\n",
        "[B] 1 2 3\n",
        "[L3]d e f\n",
    );
    assert_eq!(lyric_row_syllable_counts(source), vec![3, 3]);
}

#[test]
fn lyric_less_measure_with_merge_and_hide_settings_gets_no_blank_rows() {
    let source = concat!(
        "# metadata\n",
        "merge_duplicate_measures_across_parts = yes\n",
        "hide_resting_parts = yes\n",
        "\n",
        "# parts\n",
        "Soprano [S] = notes\n",
        "Bass [B] = notes\n",
        "Lyrics 1 [L1] = lyrics[B]\n",
        "Lyrics 3 [L3] = lyrics[B]\n",
        "\n",
        "# score\n",
        "[S] 1 2 3\n",
        "[B] 1 2 3\n",
        "\n",
        "[S] 1 2 3\n",
        "[B] 1 2 3\n",
        "[L1]a b c\n",
        "\n",
        "[S] 1 2 3\n",
        "[B] 4 5 6\n",
        "[L3]d e f\n",
    );
    assert_eq!(lyric_row_syllable_counts(source), vec![3, 3]);
}

#[test]
fn merged_empty_lyric_parts_next_to_a_sung_one_get_no_blank_row() {
    // Only L3 is sung in this system, so L1 and L2 are both empty. The
    // consolidator merges identical rows, which used to leave one blank
    // "L1 L2" row behind.
    let source = concat!(
        "# metadata\n",
        "merge_duplicate_measures_across_parts = yes\n",
        "\n",
        "# parts\n",
        "Bass [B] = notes\n",
        "Lyrics 1 [L1] = lyrics[B]\n",
        "Lyrics 2 [L2] = lyrics[B]\n",
        "Lyrics 3 [L3] = lyrics[B]\n",
        "\n",
        "# score\n",
        "[B] 1 2 3\n",
        "[L3]a b c\n",
    );
    assert_eq!(lyric_row_syllable_counts(source), vec![3]);
}
