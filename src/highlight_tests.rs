use super::*;

fn highlighted(source: &str) -> Vec<(HighlightKind, &str)> {
    highlight_tokens(source)
        .into_iter()
        .map(|token| {
            let text = source
                .get(token.span.start..token.span.end)
                .unwrap_or_default();
            (token.kind, text)
        })
        .collect()
}

#[test]
fn highlights_keywords_of_every_section() {
    let source = r#"# metadata
title = "Song"
row_height = 30

# parts
Melody [M] = notes
Chords [C] = chords 80%
Echo [E] = follow[M]

# score
bpm=92 key=C4 time=4/4 label="Verse" break
[M] 1 2 3 4
[C] 1 - - -

# sequence
Verse
"#;
    use HighlightKind::{DirectiveKey, MetadataKey, PartKind, SectionHeader};
    assert_eq!(
        highlighted(source),
        [
            (SectionHeader, "# metadata"),
            (MetadataKey, "title"),
            (MetadataKey, "row_height"),
            (SectionHeader, "# parts"),
            (PartKind, "notes"),
            (PartKind, "chords"),
            (PartKind, "follow"),
            (SectionHeader, "# score"),
            (DirectiveKey, "bpm"),
            (DirectiveKey, "key"),
            (DirectiveKey, "time"),
            (DirectiveKey, "label"),
            (DirectiveKey, "break"),
            (SectionHeader, "# sequence"),
        ]
    );
}

#[test]
fn skips_unknown_keys_directives_and_sections() {
    let source = r#"# metadata
colour = red
# drafts
bpm=1
# parts
Melody [M] = notes
# score
tempo=92 bpm=90
[M] 1
"#;
    use HighlightKind::{DirectiveKey, PartKind, SectionHeader};
    assert_eq!(
        highlighted(source),
        [
            (SectionHeader, "# metadata"),
            (SectionHeader, "# parts"),
            (PartKind, "notes"),
            (SectionHeader, "# score"),
            (DirectiveKey, "bpm"),
        ]
    );
}

#[test]
fn ignores_keywords_inside_comments_and_note_lines() {
    let source = "# parts // notes\nMelody [M] = notes // chords\n# score\n[M] 1 // bpm=3\n";
    use HighlightKind::{PartKind, SectionHeader};
    assert_eq!(
        highlighted(source),
        [
            (SectionHeader, "# parts"),
            (PartKind, "notes"),
            (SectionHeader, "# score"),
        ]
    );
}

#[test]
fn offsets_are_utf8_byte_offsets() {
    let source = "# metadata\ntitle = \"歌\"\nrow_height = 30\n";
    let tokens = highlight_tokens(source);
    let row_height = tokens.last().map(|token| token.span);
    assert_eq!(
        row_height,
        source
            .find("row_height")
            .map(|start| Span::new(start, start + 10))
    );
}

#[derive(Debug, PartialEq, Eq)]
struct HeaderText<'source> {
    kind: SectionKind,
    text: &'source str,
}

fn headers(source: &str) -> Vec<HeaderText<'_>> {
    section_headers(source)
        .into_iter()
        .map(|header| HeaderText {
            kind: header.kind,
            text: source
                .get(header.span.start..header.span.end)
                .unwrap_or_default(),
        })
        .collect()
}

fn header(kind: SectionKind, text: &str) -> HeaderText<'_> {
    HeaderText { kind, text }
}

#[test]
fn section_headers_names_each_known_section() {
    let source = "# metadata\n# parts\n# drafts\n# score\n# sequence\n";
    assert_eq!(
        headers(source),
        [
            header(SectionKind::Metadata, "# metadata"),
            header(SectionKind::Parts, "# parts"),
            header(SectionKind::Score, "# score"),
            header(SectionKind::Sequence, "# sequence"),
        ]
    );
}

#[test]
fn section_header_followed_by_a_comment_is_recognized() {
    let source = "# parts // note\nMelody [M] = notes\n";
    assert_eq!(headers(source), [header(SectionKind::Parts, "# parts")]);
}

#[test]
fn indented_header_like_line_is_not_a_section_header() {
    let source = "# score\n[M] 1\n  # parts\n";
    assert_eq!(headers(source), [header(SectionKind::Score, "# score")]);
}
