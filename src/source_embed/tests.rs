use super::*;

#[test]
fn extracts_base64_encoded_source() {
    let encoded = base64::engine::general_purpose::STANDARD.encode("[Melody] 1 2 3");
    let svg = format!(r#"<svg><metadata id="jianpu-source">{encoded}</metadata></svg>"#);
    assert_eq!(
        extract_embedded_source(&svg).as_deref(),
        Some("[Melody] 1 2 3")
    );
}

#[test]
fn returns_none_when_tag_absent() {
    assert_eq!(extract_embedded_source("<svg></svg>"), None);
}

#[test]
fn returns_none_when_contents_are_not_valid_base64() {
    let svg = r#"<svg><metadata id="jianpu-source">not-valid-base64!!!</metadata></svg>"#;
    assert_eq!(extract_embedded_source(svg), None);
}
