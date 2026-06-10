use super::*;

// ── Output-ditto tests ────────────────────────────────────────────────────

#[test]
fn explicit_ditto_part_is_marked_as_ditto_in_score() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    assert!(
        matches!(score.measures[0].parts[1], PartRow::Ditto(_)),
        "Alto part written as `\"` ditto should be PartRow::Ditto"
    );
}

#[test]
fn implicit_ditto_part_is_marked_as_ditto_in_score() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        // Alto line omitted — implicit ditto
    );
    let score = compile(input, "test.jianpu").unwrap();
    assert!(
        matches!(score.measures[0].parts[1], PartRow::Ditto(_)),
        "Alto part from implicit trailing omission should be PartRow::Ditto"
    );
}

#[test]
fn non_ditto_part_is_timed_in_score() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    assert!(
        matches!(score.measures[0].parts[0], PartRow::Timed(_)),
        "Soprano with explicit notes should be PartRow::Timed"
    );
    assert!(
        matches!(score.measures[0].parts[1], PartRow::Timed(_)),
        "Alto with explicit notes should be PartRow::Timed"
    );
}

#[test]
fn ditto_parts_produce_smaller_svg_than_non_ditto() {
    let with_ditto = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "\"\n",
    );
    let without_ditto = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let svgs_ditto = render_svgs_from_source(with_ditto, "test.jianpu").unwrap();
    let svgs_no_ditto = render_svgs_from_source(without_ditto, "test.jianpu").unwrap();
    assert!(
        svgs_ditto[0].len() < svgs_no_ditto[0].len(),
        "SVG with ditto Alto should be smaller than SVG with explicit Alto notes"
    );
}

#[test]
fn measures_with_different_ditto_patterns_go_on_separate_rows() {
    // Measure 1: both Soprano and Alto active.
    // Measure 2: Soprano active, Alto ditto'd.
    // Both fit in 28 columns but must be on separate rows because
    // their active-part sets differ.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 60\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
        "\n",
        "1 2 3 4\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert_eq!(
        total_row_groups, 2,
        "measures with different ditto patterns should be forced onto separate rows"
    );
}

#[test]
fn measures_with_same_ditto_pattern_can_share_a_row() {
    // Both measures have Alto ditto'd — they should share a single row.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 60\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "\"\n",
        "\n",
        "5 6 7 1\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert_eq!(
        total_row_groups, 1,
        "two measures with the same ditto pattern should share a single row"
    );
}

#[test]
fn same_ditto_count_but_different_parts_still_forces_line_break() {
    // Measure 1: Alto ditto'd (S + T active).
    // Measure 2: Tenor ditto'd (S + A active).
    // Both rows would have identical heights — the break decision must
    // compare WHICH parts are active, not how many.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 60\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "Tenor = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "\"\n",
        "5 6 7 1\n",
        "\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert_eq!(
        total_row_groups, 2,
        "same ditto count but different ditto'd parts must not share a row"
    );
}

#[test]
fn alternating_ditto_patterns_force_break_at_every_change() {
    // Patterns: [S,A] → [S] → [S,A]. Each change forces a break,
    // including returning to a previously seen pattern.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 60\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
        "\n",
        "1 2 3 4\n",
        "\"\n",
        "\n",
        "5 6 7 1\n",
        "1 2 3 4\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert_eq!(
        total_row_groups, 3,
        "every ditto-pattern change should start a new row"
    );
}

#[test]
fn width_wrapping_still_applies_within_same_ditto_pattern() {
    // Many measures sharing one ditto pattern must still wrap when the
    // row runs out of columns — pattern-matching must not disable
    // ordinary width-based wrapping.
    let mut input = String::from(concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 28\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
    ));
    for _ in 0..6 {
        input.push_str("1 2 3 4\n\"\n\n");
    }
    let score = compile(&input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert!(
        total_row_groups > 1,
        "six 16-beat measures cannot fit one 28-column row; width wrapping must still occur"
    );
}

#[test]
fn partially_ditto_part_counts_as_active_for_line_breaking() {
    // Alto's notes line is ditto but its lyrics line is explicit —
    // the part still renders, so its pattern matches a fully-active
    // measure and the two can share a row.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 60\n",
        "\n",
        "[parts]\n",
        "Soprano = notes lyrics\n",
        "Alto = notes lyrics\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "5 6 7 1\n",
        "la la la la\n",
        "\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "\"\n",
        "ah ah ah ah\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    assert!(
        matches!(score.measures[1].parts[1], PartRow::Timed(_)),
        "part with explicit lyrics over ditto notes must stay Timed"
    );
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert_eq!(
        total_row_groups, 1,
        "partially-ditto part is active, so both measures share one row"
    );
}

#[test]
fn ditto_row_group_is_shorter_than_fully_active_row_group() {
    // A row where Alto is ditto'd should be shorter than one where both are active.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
        "\n",
        "1 2 3 4\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let heights: Vec<u32> = pages
        .iter()
        .flat_map(|p| p.row_groups.iter())
        .map(|rg| rg.height_in_rows)
        .collect();
    assert_eq!(heights.len(), 2);
    assert!(
        heights[0] > heights[1],
        "row with both parts active (height={}) should be taller than row with Alto ditto'd (height={})",
        heights[0],
        heights[1]
    );
}
