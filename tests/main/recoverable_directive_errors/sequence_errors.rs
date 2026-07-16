use super::*;

// Group 1e — `# sequence` section errors

fn fixture_with_sequence(score_section: &str, sequence: &str) -> String {
    format!(
        "# metadata\ntitle = \"t\"\nauthor = \"a\"\n\n# parts\nMelody = notes\n\n# sequence\n{sequence}\n\n# score\n{score_section}\n"
    )
}

#[test]
fn sequence_combined_with_inline_marker_is_recoverable() {
    let source = fixture_with_sequence(
        concat!(
            "time=4/4 key=C4 bpm=120 label=\"A\" dcalcoda\n[Melody] 1 2 3 4\n\n",
            "tocoda\n[Melody] 5 6 7 1\n\n",
            "coda\n[Melody] 1 1 1 1\n",
        ),
        "A",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("sequence combined with inline marker must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "cannot be combined with a `# sequence` section"),
        "expected error about sequence/marker conflict, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn sequence_referencing_undefined_label_is_recoverable() {
    let source = fixture_with_sequence(
        "time=4/4 key=C4 bpm=120 label=\"A\"\n[Melody] 1 2 3 4\n",
        "A, Z",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("sequence referencing undefined label must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "undefined label"),
        "expected error about undefined label, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn sequence_with_duplicate_label_definitions_is_recoverable() {
    let source = fixture_with_sequence(
        concat!(
            "time=4/4 key=C4 bpm=120 label=\"A\"\n[Melody] 1 2 3 4\n\n",
            "label=\"A\"\n[Melody] 5 6 7 1\n",
        ),
        "A",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("duplicate label definitions must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "defined more than once"),
        "expected error about duplicate label definition, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
