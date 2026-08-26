//! Verifies the *final* displayed row label — resolved by
//! `layout_systems::resolve_label` once a system's full `RowId` membership is
//! known, downstream of `consolidator::consolidate_rows`'s per-measure-only
//! merge decisions (see `consolidator/tests.rs`, which only checks those
//! decisions, not the label text they eventually produce).

use crate::compiler::compile;
use crate::consolidator::consolidate;
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::{GridContent, Header};
use crate::grouper::group;
use crate::parser::parse;
use crate::render_config::RenderConfig;

fn header() -> Header {
    Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: 45.0,
        subtitle_font_size: 24.0,
        author_font_size: 18.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
    }
}

/// Every `RowLabel` text in the laid-out score, in row order (top to bottom,
/// `[parts]` declaration order — see `union_row_order`).
fn row_labels(source: &str) -> Vec<String> {
    let document = parse(source, "test", &[]).unwrap();
    let score = group(document).unwrap();
    let config = RenderConfig::from_metadata(&score.metadata);
    let compile_result = consolidate(compile(&score));
    let output = layout(&compile_result, &config, &header(), 2000.0, 4000.0, None);
    output
        .pages
        .iter()
        .flat_map(|page| page.rows.iter())
        .flat_map(|row| row.elements.iter())
        .filter_map(|el| match &el.content {
            GridContent::RowLabel(text) => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn group_broadcast_label_after_union() {
    // Both members broadcast identical content in this single-measure score,
    // so S1/S2 merge into one row for the whole (one-measure) system — the
    // group abbreviation label from `consolidator::tests::
    // group_broadcast_merge_records_both_members_as_one_row` is resolved
    // here, once the system's full row membership is known.
    let source = concat!(
        "# parts\n",
        "Soprano 1 [S1] = notes\n",
        "Soprano 2 [S2] = notes\n",
        "\n",
        "# groups\n",
        "Soprano [s] = S1 S2\n",
        "\n",
        "# score\n",
        "[s] 1 2 3 4\n",
    );
    assert_eq!(row_labels(source), vec!["s".to_string()]);
}

#[test]
fn coincidental_match_with_an_unrelated_part_does_not_widen_the_group_label() {
    // Regression test: S1/S2 (the "Soprano" group) and Tenor happen to share
    // identical notes in measure 0 only; Tenor has different notes in
    // measure 1. Tenor therefore needs its own persistent row for the whole
    // system (it can't stay folded into the Soprano row, since that row
    // doesn't hold Tenor's content in measure 1) — so the one-measure
    // coincidence in measure 0 must not taint the Soprano row's label into
    // "s T": only S1/S2 ever share a row for the *whole* system, so the
    // label must stay "s".
    let source = concat!(
        "# parts\n",
        "Soprano 1 [S1] = notes\n",
        "Soprano 2 [S2] = notes\n",
        "Tenor [T] = notes\n",
        "\n",
        "# groups\n",
        "Soprano [s] = S1 S2\n",
        "\n",
        "# score\n",
        "[s] 1 2 3 4\n",
        "[T] 1 2 3 4\n",
        "\n",
        "[s] 5 6 7 1\n",
        "[T] 2 3 4 5\n",
    );
    assert_eq!(row_labels(source), vec!["s".to_string(), "T".to_string()]);
}
