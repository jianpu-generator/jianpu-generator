use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

fn doc(score_content: &str) -> String {
    format!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nM = notes\nC = lyrics\n\n# score\n{score_content}"
    )
}

fn lyric_lines(row: &MeasureRow) -> Vec<(u32, &str, usize)> {
    row.elements
        .iter()
        .filter_map(|e| {
            if let ElementContent::LyricLine { text, verse } = &e.content {
                Some((e.column, text.as_str(), *verse))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn lyrics_only_part_emits_one_line_per_verse_at_column_zero() {
    let source = doc("time=4/4 key=C4 bpm=120\n[M] 1 2 3 4\n[C] hello world\n");
    let score = score_from(&source);
    let result = compile(&score);
    let block = &result.blocks[0];
    let caption_row = block
        .rows
        .iter()
        .find(|r| lyric_lines(r).len() == 1)
        .expect("caption row with one lyric line");
    assert_eq!(lyric_lines(caption_row), vec![(0, "hello world", 0)]);
}

#[test]
fn lyrics_only_part_rejoins_tokenized_syllables_with_spaces() {
    // `tokenize_lyrics` splits each CJK character into its own syllable; a
    // standalone `lyrics` part rejoins them with spaces into one line rather
    // than tying one syllable per note (there are no notes to tie to).
    let source = doc("time=4/4 key=C4 bpm=120\n[M] 1 2 3 4\n[C] 春天\n");
    let score = score_from(&source);
    let result = compile(&score);
    let block = &result.blocks[0];
    let caption_row = block
        .rows
        .iter()
        .find(|r| !lyric_lines(r).is_empty())
        .expect("caption row");
    assert_eq!(lyric_lines(caption_row)[0].1, "春 天");
}

#[test]
fn lyrics_only_part_supports_multiple_verses() {
    let source = doc("time=4/4 key=C4 bpm=120\n[M] 1 2 3 4\n[C] verse one\n[C] verse two\n");
    let score = score_from(&source);
    let result = compile(&score);
    let block = &result.blocks[0];
    let caption_row = block
        .rows
        .iter()
        .find(|r| lyric_lines(r).len() == 2)
        .expect("caption row with two verses");
    let lines = lyric_lines(caption_row);
    assert_eq!(lines[0], (0, "verse one", 0));
    assert_eq!(lines[1], (0, "verse two", 1));
}

#[test]
fn lyrics_only_part_coexists_with_notes_part_in_same_measure() {
    let source = doc("time=4/4 key=C4 bpm=120\n[M] 1 2 3 4\n[C] caption text\n");
    let score = score_from(&source);
    let result = compile(&score);
    let block = &result.blocks[0];
    assert_eq!(block.rows.len(), 2, "notes row + lyrics-only row");
    let notes_row = &block.rows[0];
    assert!(notes_row
        .elements
        .iter()
        .any(|e| matches!(e.content, ElementContent::NoteHead { .. })));
    assert!(lyric_lines(notes_row).is_empty());
}
