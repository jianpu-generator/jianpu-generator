use cloud_outbox::merge::merge_three_way;
use cloud_outbox::types::MergeOutcome;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn expect(actual: &MergeOutcome, expected: &MergeOutcome) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected:?}, got {actual:?}").into())
    }
}

#[test]
fn merge_non_overlapping_edits_are_clean() -> TestResult {
    let base = "a\nb\nc\nd\ne\nf\ng\n";
    let mine = "A\nb\nc\nd\ne\nf\ng\n";
    let theirs = "a\nb\nc\nd\ne\nf\nG\n";
    expect(
        &merge_three_way(base, mine, theirs, |_| true),
        &MergeOutcome::Clean {
            text: "A\nb\nc\nd\ne\nf\nG\n".into(),
        },
    )
}

#[test]
fn merge_overlapping_edits_conflict_with_markers() -> TestResult {
    match merge_three_way("a\n", "mine\n", "theirs\n", |_| true) {
        MergeOutcome::Conflicted {
            text,
            has_markers: true,
        } if text.contains("<<<<<<<") && text.contains(">>>>>>>") => Ok(()),
        other => Err(format!("unexpected outcome {other:?}").into()),
    }
}

#[test]
fn merge_identical_edits_are_clean() -> TestResult {
    expect(
        &merge_three_way("a\n", "x\n", "x\n", |_| true),
        &MergeOutcome::Clean { text: "x\n".into() },
    )
}

#[test]
fn merge_empty_base_with_same_text_is_clean() -> TestResult {
    expect(
        &merge_three_way("", "x\n", "x\n", |_| true),
        &MergeOutcome::Clean { text: "x\n".into() },
    )
}

#[test]
fn merge_empty_base_with_different_text_conflicts() -> TestResult {
    match merge_three_way("", "x\n", "y\n", |_| true) {
        MergeOutcome::Conflicted {
            has_markers: true, ..
        } => Ok(()),
        other => Err(format!("unexpected outcome {other:?}").into()),
    }
}

#[test]
fn merge_validator_rejection_returns_clean_text_without_markers() -> TestResult {
    expect(
        &merge_three_way("a\n", "x\n", "x\n", |_| false),
        &MergeOutcome::Conflicted {
            text: "x\n".into(),
            has_markers: false,
        },
    )
}
