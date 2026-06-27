use crate::compiler::{compile, types::MeasureBlock};
use crate::consolidator::consolidate;
use crate::grouper::group;
use crate::parser::parse;

fn consolidated_blocks(source: &str) -> Vec<MeasureBlock> {
    let document = parse(source, "test", &[]).unwrap();
    let score = group(document).unwrap();
    let result = compile(&score);
    consolidate(result).blocks
}

#[test]
fn follow_part_identical_to_source_is_omitted_per_measure() {
    // Measure 1: B follows A and is not explicitly given any data, so B is
    // identical to A in both notes and lyrics. B should be omitted.
    //
    // Measure 2: B is explicitly given notes (3 4 5 6), so B's notes differ
    // from A's. B's lyrics are still identical to A's (follow fills them from
    // A). B's notes row should appear, but B's lyrics should be omitted.
    let source = concat!(
        "# metadata\n",
        "title = \"hello\"\n",
        "author = \"\"\n",
        "\n",
        "\n",
        "# parts\n",
        "A = notes+lyrics\n",
        "B = follow[A]\n",
        "\n",
        "# score\n",
        "[A] 1 2 3 4\n",
        "[A] la la la la\n",
        "\n",
        "[A] 1 2 3 4\n",
        "[A]la la la la\n",
        "[B] 3 4 5 6\n",
    );
    let blocks = consolidated_blocks(source);

    // Measure 1: B is identical to A → only A's notes and lyrics rows remain
    assert_eq!(
        blocks[0].rows.len(),
        2,
        "measure 1: B (fully identical follow) should be omitted; got rows: {:?}",
        blocks[0]
            .rows
            .iter()
            .map(|row| &row.label)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        blocks[0].rows[0].label, "A B",
        "measure 1: notes row should merge B into A"
    );
    assert_eq!(
        blocks[0].rows[1].label, "A B",
        "measure 1: lyrics row should merge B into A"
    );

    // Measure 2: B has different notes → A notes, A lyrics, and B notes appear
    assert_eq!(
        blocks[1].rows.len(),
        3,
        "measure 2: A notes, A lyrics, and B notes should appear"
    );
    assert_eq!(
        blocks[1].rows[0].label, "A",
        "measure 2: first row should be A notes"
    );
    assert_eq!(
        blocks[1].rows[1].label, "A B",
        "measure 2: lyrics row should merge B into A"
    );
    assert_eq!(
        blocks[1].rows[2].label, "B",
        "measure 2: third row should be B notes"
    );
}
