use super::migrate_lyrics_syntax;

#[test]
fn single_verse_becomes_a_lyrics_part() {
    let source = "# parts\nMelody [M] = notes\n\n# score\n[M] 1 2 3 4\nla la la la\n";
    assert_eq!(
        migrate_lyrics_syntax(source),
        "# parts\nMelody [M] = notes\nMelody lyrics [Mv1] = lyrics[M]\n\n# score\n[M] 1 2 3 4\n[Mv1] la la la la\n"
    );
}

#[test]
fn multiple_verses_are_numbered_in_order() {
    let source = "# parts\nM = notes\n\n# score\n[M] 1 2 3 4\na b c d\ne f g h\n";
    assert_eq!(
        migrate_lyrics_syntax(source),
        "# parts\nM = notes\nM verse 1 [Mv1] = lyrics[M]\nM verse 2 [Mv2] = lyrics[M]\n\n# score\n[M] 1 2 3 4\n[Mv1] a b c d\n[Mv2] e f g h\n"
    );
}

#[test]
fn verse_count_is_the_maximum_across_measures() {
    let source =
        "# parts\nM = notes\n\n# score\n[M] 1 2 3 4\na b c d\n\n[M] 1 2 3 4\ne f g h\ni j k l\n";
    let migrated = migrate_lyrics_syntax(source);
    assert!(migrated.contains("M verse 2 [Mv2] = lyrics[M]"));
    assert!(migrated.contains("[Mv1] e f g h\n[Mv2] i j k l"));
}

#[test]
fn bare_line_attaches_to_the_nearer_part() {
    let source =
        "# parts\nA = notes\nB = notes\n\n# score\n[A] 1 2 3 4\n[B] 5 6 7 0\nla la la la\n";
    let migrated = migrate_lyrics_syntax(source);
    assert!(migrated.contains("B lyrics [Bv1] = lyrics[B]"));
    assert!(!migrated.contains("lyrics[A]"));
    assert!(migrated.contains("[Bv1] la la la la"));
}

#[test]
fn directive_line_is_not_a_lyric_line() {
    let source = "# parts\nM = notes\n\n# score\nbpm=90\n[M] 1 2 3 4\nla la la la\n";
    let migrated = migrate_lyrics_syntax(source);
    assert!(migrated.contains("bpm=90\n[M] 1 2 3 4\n[Mv1] la la la la"));
}

#[test]
fn source_without_lyrics_is_unchanged() {
    let source = "# parts\nM = notes\n\n# score\n[M] 1 2 3 4\n";
    assert_eq!(migrate_lyrics_syntax(source), source);
}

#[test]
fn verse_abbreviation_avoids_existing_abbreviations() {
    let source = "# parts\nM = notes\nMv1 = notes\n\n# score\n[M] 1 2 3 4\nla la la la\n";
    assert!(migrate_lyrics_syntax(source).contains("[Mv1_] = lyrics[M]"));
}

#[test]
fn migrating_twice_changes_nothing() {
    let source = "# parts\nM = notes\n\n# score\n[M] 1 2 3 4\nla la la la\n";
    let once = migrate_lyrics_syntax(source);
    assert_eq!(migrate_lyrics_syntax(&once), once);
}

#[test]
fn a_comment_on_a_part_line_is_kept() {
    let source = "# parts\nM = notes // main\n\n# score\n[M] 1 2 3 4\nla la la la\n";
    let migrated = migrate_lyrics_syntax(source);
    assert!(migrated.contains("M = notes // main\nM lyrics [Mv1] = lyrics[M]"));
}
