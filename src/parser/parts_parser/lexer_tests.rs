use super::lexer::{lex_line, PartsToken};
use crate::ast::parsed::PartKind;
use crate::error::{RecoverableErrorKind, Span, Spanned};

fn token_values(tokens: &[Spanned<PartsToken>]) -> Vec<PartsToken> {
    tokens.iter().map(|token| token.value.clone()).collect()
}

#[test]
fn lexes_lhs_with_brackets() {
    let line = "Alto 1 & Tenor [A1&T] = notes+lyrics";
    let tokens = lex_line(line, 0, Span::new(0, line.len())).expect("lex");
    assert_eq!(
        token_values(&tokens),
        vec![
            PartsToken::Name("Alto 1 & Tenor".to_string()),
            PartsToken::LBracket,
            PartsToken::Abbreviation("A1&T".to_string()),
            PartsToken::RBracket,
            PartsToken::Equals,
            PartsToken::Kind(PartKind::NotesWithLyrics),
        ]
    );
}

#[test]
fn lexes_rhs_kind_soundfont_volume_and_octave() {
    let line = "Bass = notes \"5: Guitar\" 75% -1";
    let tokens = lex_line(line, 0, Span::new(0, line.len())).expect("lex");
    assert_eq!(
        token_values(&tokens),
        vec![
            PartsToken::Name("Bass".to_string()),
            PartsToken::Equals,
            PartsToken::Kind(PartKind::Notes),
            PartsToken::Soundfont("5: Guitar".to_string()),
            PartsToken::Volume(75),
            PartsToken::OctaveOffset(-1),
        ]
    );
}

#[test]
fn follow_target_token_span_covers_abbreviation_only() {
    let prefix = "Alto [A] = ";
    let line = "Alto [A] = follow[UNKNOWN]";
    let trimmed_start = prefix.len();
    let tokens = lex_line(
        line,
        trimmed_start,
        Span::new(0, trimmed_start + line.len()),
    )
    .expect("lex");
    let follow_target = tokens
        .iter()
        .find(|token| matches!(token.value, PartsToken::FollowTarget(_)))
        .expect("follow target token");
    let expected_start = trimmed_start + prefix.len() + "follow[".len();
    assert_eq!(follow_target.span.start, expected_start);
    assert_eq!(follow_target.span.end, expected_start + "UNKNOWN".len());
}

#[test]
fn lexes_follow_with_soundfont_and_volume() {
    let line = "B = follow[A] \"1: Grand Piano\" 80%";
    let tokens = lex_line(line, 0, Span::new(0, line.len())).expect("lex");
    assert_eq!(
        token_values(&tokens),
        vec![
            PartsToken::Name("B".to_string()),
            PartsToken::Equals,
            PartsToken::Follow,
            PartsToken::FollowTarget("A".to_string()),
            PartsToken::Soundfont("1: Grand Piano".to_string()),
            PartsToken::Volume(80),
        ]
    );
}

#[test]
fn lexes_soundfont_with_equals_in_name() {
    let line = "Bass = notes \"1: Grand = Piano\"";
    let tokens = lex_line(line, 0, Span::new(0, line.len())).expect("lex");
    assert_eq!(
        token_values(&tokens),
        vec![
            PartsToken::Name("Bass".to_string()),
            PartsToken::Equals,
            PartsToken::Kind(PartKind::Notes),
            PartsToken::Soundfont("1: Grand = Piano".to_string()),
        ]
    );
}

#[test]
fn declaration_equals_is_outside_quotes() {
    let line = "Bass = notes \"1: Grand = Piano\"";
    let tokens = lex_line(line, 0, Span::new(0, line.len())).expect("lex");
    let equals = tokens
        .iter()
        .find(|token| matches!(token.value, PartsToken::Equals))
        .expect("equals token");
    assert_eq!(equals.span.start, "Bass ".len());
    assert_eq!(equals.span.end, "Bass =".len());
}

#[test]
fn soundfont_token_span_covers_quoted_region() {
    let prefix = "    ";
    let line = "Bass = notes \"1: Grand = Piano\"";
    let padded = format!("{prefix}{line}");
    let trimmed_start = prefix.len();
    let tokens = lex_line(line, trimmed_start, Span::new(0, padded.len())).expect("lex");
    let soundfont = tokens
        .iter()
        .find(|token| matches!(token.value, PartsToken::Soundfont(_)))
        .expect("soundfont token");
    let expected_start = trimmed_start + "Bass = notes ".len();
    assert_eq!(soundfont.span.start, expected_start);
    assert_eq!(
        soundfont.span.end,
        expected_start + "\"1: Grand = Piano\"".len()
    );
}

#[test]
fn rejects_missing_equals() {
    let line = "malformed-no-equals";
    let error = lex_line(line, 0, Span::new(0, line.len())).expect_err("error");
    assert!(matches!(
        error.kind,
        RecoverableErrorKind::PartsMalformedLine { .. }
    ));
}

#[test]
fn rejects_unclosed_quote() {
    let line = "Bass = notes \"5: Guitar";
    let error = lex_line(line, 0, Span::new(0, line.len())).expect_err("error");
    assert!(matches!(
        error.kind,
        RecoverableErrorKind::PartsInvalidColumns { .. }
    ));
}

#[test]
fn rejects_empty_follow_abbreviation() {
    let line = "B = follow[]";
    let error = lex_line(line, 0, Span::new(0, line.len())).expect_err("error");
    assert!(matches!(
        error.kind,
        RecoverableErrorKind::PartsInvalidColumns { .. }
    ));
}

#[test]
fn rejects_unknown_rhs_token() {
    let line = "X = bogus";
    let error = lex_line(line, 0, Span::new(0, line.len())).expect_err("error");
    assert!(matches!(
        error.kind,
        RecoverableErrorKind::PartsInvalidColumns { .. }
    ));
}
