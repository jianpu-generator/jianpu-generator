#![allow(clippy::disallowed_macros)]
use jianpu_generator::error::Diagnostic;
use jianpu_generator::render_svgs_from_source;

fn minimal_fixture(score_section: &str) -> String {
    format!(
        r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes

# score
{score_section}
"#
    )
}

fn has_error_containing(output: &jianpu_generator::RenderOutput, keyword: &str) -> bool {
    output
        .diagnostics
        .iter()
        .any(|d| matches!(d, Diagnostic::Error(_)) && d.message().contains(keyword))
}

// Tie/slur group at measure start (motivating case for removing directive parens)

#[test]
fn measure_starting_with_paren_group_is_not_a_directive() {
    let source = minimal_fixture("(1 2) 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("measure starting with tie group must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        output
            .diagnostics
            .iter()
            .all(|d| !matches!(d, Diagnostic::Error(_))),
        "expected no errors, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group 1a — Directive row errors (whole-row skip)

#[test]
fn directive_unclosed_quote_is_recoverable() {
    let source = minimal_fixture("label=\"unterminated\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("unclosed quote must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unclosed quote"),
        "expected error about unclosed quote, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group 1b — Per-token directive errors

#[test]
fn directive_invalid_bpm_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=abc\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("invalid bpm must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid bpm"),
        "expected error about invalid bpm, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_label_not_quoted_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120 label=unquoted\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("unquoted label must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "quoted string"),
        "expected error about quoted string, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_empty_label_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120 label=\"\"\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("empty label must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "must not be empty"),
        "expected error about empty label, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_unknown_token_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120 unknown_token\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("unknown directive token must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unknown directive"),
        "expected error about unknown directive, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_key_missing_note_name_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=4 bpm=120\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("key= with digit only must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid note name")
            || has_error_containing(&output, "expected note name"),
        "expected error about missing note name, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_key_invalid_note_letter_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=Z4 bpm=120\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("key= with invalid note letter must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid note name"),
        "expected error about invalid note name, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_time_not_fraction_is_recoverable() {
    let source = minimal_fixture("time=abc key=C4 bpm=120\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("time= not in N/D form must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid time"),
        "expected error about invalid time signature, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_time_numerator_too_large_is_recoverable() {
    let source = minimal_fixture("time=999/4 key=C4 bpm=120\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("time numerator too large must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid time numerator"),
        "expected error about invalid time numerator, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_time_zero_denominator_is_recoverable() {
    let source = minimal_fixture("time=4/0 key=C4 bpm=120\n1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("time denominator zero must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "zero"),
        "expected error about zero denominator, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group 2 — Inline timed-lexer errors (in notes line)

#[test]
fn inline_bpm_invalid_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\nbpm=abc 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("inline invalid bpm must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "bpm="),
        "expected error about bpm=, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn inline_time_zero_denominator_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n4/0 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("inline time 4/0 must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "zero"),
        "expected error about zero denominator, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group — Dotted eighth without sixteenth tail

#[test]
fn dotted_eighth_note_without_sixteenth_tail_is_recoverable() {
    // `1_.` is a dotted eighth note; it must be followed by a sixteenth.
    // When no sixteenth tail follows, render must continue and surface a diagnostic.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n1_. 2_ 3_ 4_ 5_ 6_ 7_ 0=\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("dotted eighth without sixteenth tail must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "dotted eighth"),
        "expected error about dotted eighth, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn dotted_eighth_rest_without_sixteenth_tail_is_recoverable() {
    // `0_.` is a dotted eighth rest; same rule applies.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n0_. 1_ 2_ 3_ 4_ 5_ 6_ 0=\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("dotted eighth rest without sixteenth tail must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "dotted eighth"),
        "expected error about dotted eighth, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// NoteExpectedPitchDigit

#[test]
fn note_invalid_pitch_char_is_recoverable() {
    // 'x' is not a valid pitch digit (0-7); the lexer rejects it as LexUnexpectedChar,
    // which is recoverable — the measure is skipped and the render continues.
    let source = minimal_fixture("1 x 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("invalid pitch char must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unexpected character"),
        "expected error about unexpected character, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn dotted_eighth_with_sixteenth_tail_is_valid() {
    // `1_.` followed by `2=` (sixteenth) is a valid pattern — no error expected.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n1_. 2= 3_ 4_ 5_ 6_ 7_ 1_\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("dotted eighth with sixteenth tail must not abort");
    assert!(!output.svgs.is_empty());
    assert!(
        !has_error_containing(&output, "dotted eighth"),
        "expected no dotted-eighth error for valid pattern, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// DurationCannotDotQuarterBeat

#[test]
fn dotted_quarter_beat_is_recoverable() {
    // `1=.` applies a dot to a quarter-beat note, which is invalid.
    // The render must continue; the dot is ignored and duration stays at 1 beat.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n1=. 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("dotted quarter-beat must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "cannot dot a quarter-beat"),
        "expected error about dotted quarter-beat, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// GroupUnexpectedCloseParen

#[test]
fn group_unexpected_close_paren_is_recoverable() {
    // `1 2) 3 4` has a stray `)` with no matching `(`.
    // The render must continue; the `)` is ignored and an error is reported on the measure.
    let source = minimal_fixture("1 2) 3 4\n");
    let output =
        render_svgs_from_source(&source, "test.jianpu").expect("stray ) must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unexpected"),
        "expected error about unexpected ), got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// UnclosedGroupAtEnd

#[test]
fn unclosed_paren_group_at_eof_is_recoverable() {
    // `(1 2 3 4` opens a group but never closes it before EOF.
    // The render must continue; the group is treated as open and an error is reported.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n(1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("unclosed group at EOF must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unclosed '(' group"),
        "expected error about unclosed group, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// DurationMixedOctaveMarkers

#[test]
fn mixed_octave_markers_are_recoverable() {
    // `1',` has both ' (octave-up) and , (octave-down) — mixed octave markers.
    // The render must continue; the note is emitted with octave shift zeroed out.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n1', 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu")
        .expect("mixed octave markers must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "mixed octave"),
        "expected error about mixed octave markers, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
