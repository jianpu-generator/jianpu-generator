use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

#[test]
fn blank_lyric_part_before_a_written_one_draws_no_row() {
    let source = concat!(
        "# parts\n",
        "S = notes\n",
        "V1 = lyrics[S]\n",
        "V2 = lyrics[S]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4\n",
        "[V2] la la la la\n",
    );
    let score = group(parse(source, "test", &[]).unwrap()).unwrap();

    let rows = &compile(&score).blocks[0].rows;

    let ids: Vec<&str> = rows.iter().map(|row| row.id.0.as_str()).collect();
    assert_eq!(ids, vec!["S", "V2"]);
    assert!(matches!(rows[1].kind, RowKind::Lyrics { .. }));
}
