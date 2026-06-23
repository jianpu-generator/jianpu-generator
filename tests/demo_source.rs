#![allow(clippy::panic, clippy::disallowed_macros)]

#[test]
fn reference_jianpu_parses_and_renders() {
    let source = include_str!("../reference.jianpu");
    let svgs = jianpu_generator::render_svgs_from_source(source, "reference.jianpu")
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
