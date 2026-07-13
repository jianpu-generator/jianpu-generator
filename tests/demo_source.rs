#![allow(clippy::panic, clippy::disallowed_macros)]

#[test]
fn reference_jianpu_parses_and_renders() {
    let source = include_str!("../reference.jianpu");
    let svgs = jianpu_generator::render_svgs_from_source(source, "reference.jianpu", &[])
        .unwrap_or_else(|e| {
            panic!("reference.jianpu failed to parse/render: {e}");
        })
        .svgs;
    assert!(
        !svgs.is_empty(),
        "reference.jianpu should produce at least one SVG page"
    );
    assert!(
        svgs.iter()
            .all(|svg| svg.starts_with("<svg") && svg.ends_with("</svg>")),
        "reference.jianpu SVG output should be well-formed"
    );
}

#[test]
fn reference_jianpu_has_no_diagnostics() {
    let source = include_str!("../reference.jianpu");
    let output = jianpu_generator::render_svgs_from_source(source, "reference.jianpu", &[])
        .unwrap_or_else(|e| panic!("reference.jianpu failed to parse/render: {e}"));
    assert!(
        output.diagnostics.is_empty(),
        "reference.jianpu should have no errors or warnings, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn reference_jianpu_renders_expected_content() {
    let source = include_str!("../reference.jianpu");
    let output = jianpu_generator::render_svgs_from_source(source, "reference.jianpu", &[])
        .unwrap_or_else(|e| panic!("reference.jianpu failed to parse/render: {e}"));
    let svg = output.svgs.join("");
    assert!(
        svg.contains('春'),
        "reference.jianpu should render CJK lyrics"
    );
    assert!(
        svg.contains("1m"),
        "reference.jianpu should render minor chord symbols"
    );
}
