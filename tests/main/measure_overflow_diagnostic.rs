#![allow(clippy::disallowed_macros)]
use jianpu_generator::error::{Diagnostic, WarningKind};
use jianpu_generator::render_svgs_from_source;

/// A `.jianpu` source packing far more measures into one system than the
/// page can fit — `max_measures_per_system` is cranked up so `pack_into_systems`
/// (which is width-blind by design, see `ARCHITECTURE.md`) never splits the
/// run into multiple systems, forcing the system's summed column rods past
/// the page's usable music width.
fn overflowing_fixture() -> String {
    let measures: String = std::iter::repeat_n("[Melody] 1 2 3 4\n", 40)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"# metadata
title = "t"
author = "a"
max_measures_per_system = 40

# parts
Melody = notes

# score
time=4/4 key=C4 bpm=120
{measures}"#
    )
}

fn has_measure_overflow_warning(output: &jianpu_generator::RenderOutput) -> bool {
    output.diagnostics.iter().any(|d| {
        matches!(
            d,
            Diagnostic::Warning(w) if w.kind == WarningKind::MeasureOverflow
        )
    })
}

#[test]
fn tightly_packed_system_produces_measure_overflow_warning() {
    let source = overflowing_fixture();
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .expect("an overflowing system must not abort the render");
    assert!(
        !output.svgs.is_empty(),
        "render should still produce output"
    );
    assert!(
        has_measure_overflow_warning(&output),
        "expected a WarningKind::MeasureOverflow diagnostic, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn ordinary_score_produces_no_measure_overflow_warning() {
    let source = r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4
"#;
    let output = render_svgs_from_source(source, "test.jianpu", &[])
        .expect("a normal score must render without error");
    assert!(
        !has_measure_overflow_warning(&output),
        "an ordinary, non-overflowing score should not produce a \
         MeasureOverflow warning, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
