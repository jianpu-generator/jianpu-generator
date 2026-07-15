use super::*;

// Group 1c — Navigation marker (segno/dsalcoda/tocoda/coda) set errors

#[test]
fn navigation_partial_ds_scheme_is_recoverable() {
    let source = minimal_fixture("segno\n[Melody] 1 2 3 4\n\ndsalcoda\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("partial segno/dsalcoda set must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "must all appear together"),
        "expected error about markers appearing together, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn navigation_tocoda_after_coda_is_recoverable() {
    let source = minimal_fixture(
        "segno\n[Melody] 1 2 3 4\n\ncoda\n[Melody] 1 2 3 4\n\ntocoda\n[Melody] 1 2 3 4\n\ndsalcoda\n[Melody] 1 2 3 4\n",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("tocoda after coda must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "tocoda must occur before coda"),
        "expected error about tocoda/coda order, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn navigation_tocoda_before_segno_is_recoverable() {
    let source = minimal_fixture(
        "tocoda\n[Melody] 1 2 3 4\n\nsegno\n[Melody] 1 2 3 4\n\ncoda\n[Melody] 1 2 3 4\n\ndsalcoda\n[Melody] 1 2 3 4\n",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("tocoda before segno must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "tocoda must occur at or after segno"),
        "expected error about tocoda/segno order, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn navigation_dcalcoda_with_segno_is_recoverable() {
    let source = minimal_fixture(
        "segno\n[Melody] 1 2 3 4\n\ndcalcoda\n[Melody] 1 2 3 4\n\ntocoda\n[Melody] 1 2 3 4\n\ncoda\n[Melody] 1 2 3 4\n",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("dcalcoda combined with segno must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "cannot appear together"),
        "expected error about dcalcoda/segno conflict, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group 1d — Navigation marker (dcalfine/dsalfine/fine) set errors

#[test]
fn navigation_partial_dcalfine_set_is_recoverable() {
    let source = minimal_fixture("dcalfine\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("partial dcalfine set must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "must all appear together"),
        "expected error about markers appearing together, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn navigation_dcalfine_with_tocoda_coda_is_recoverable() {
    let source = minimal_fixture(
        "tocoda\n[Melody] 1 2 3 4\n\ncoda\n[Melody] 1 2 3 4\n\nfine\n[Melody] 1 2 3 4\n\ndcalfine\n[Melody] 1 2 3 4\n",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("dcalfine combined with a stray tocoda/coda must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "cannot appear together"),
        "expected error about fine/coda conflict, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn navigation_fine_before_segno_is_recoverable() {
    let source = minimal_fixture(
        "fine\n[Melody] 1 2 3 4\n\nsegno\n[Melody] 1 2 3 4\n\ndsalfine\n[Melody] 1 2 3 4\n",
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("fine before segno must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "fine must occur at or after segno"),
        "expected error about fine/segno order, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
