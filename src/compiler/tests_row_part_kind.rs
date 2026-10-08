use crate::ast::parsed::PartKind;
use crate::compiler::compile;
use crate::grouper::group;
use crate::parser::parse;

#[test]
fn compiled_rows_carry_their_parts_kind() {
    let source = concat!(
        "# metadata\n",
        "hide_resting_parts = no\n",
        "\n",
        "# parts\n",
        "S = notes\n",
        "V = lyrics[S]\n",
        "C = chords\n",
        "D = percussion\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4\n",
        "[V] la la la la\n",
        "[C] 1 - - -\n",
        "[D] 1 1 1 1\n",
    );
    let score = group(parse(source, "test", &[]).unwrap()).unwrap();
    let kinds: Vec<(String, PartKind)> = compile(&score).blocks[0]
        .rows
        .iter()
        .map(|row| (row.id.0.clone(), row.kind))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("S".to_string(), PartKind::Notes),
            (
                "V".to_string(),
                PartKind::Lyrics {
                    target_part_index: 0
                }
            ),
            ("C".to_string(), PartKind::Chords),
            ("D".to_string(), PartKind::Percussion),
        ]
    );
}
