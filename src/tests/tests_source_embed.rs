use super::*;
use crate::source_embed::extract_embedded_source;

#[test]
fn round_trip_recovers_original_source() {
    let source =
        "# metadata\ntitle = \"Testing\"\n\n# parts\nMelody = notes\n\n# score\n[Melody] 1 2 3 4\n";
    let output = render_svgs_from_source(source, "test.jianpu", &[]).unwrap();
    for svg in &output.svgs {
        assert_eq!(extract_embedded_source(svg).as_deref(), Some(source));
    }
}

#[test]
fn score_only_render_has_no_embedded_source() {
    let source =
        "# metadata\ntitle = \"Testing\"\n\n# parts\nMelody = notes\n\n# score\n[Melody] 1 2 3 4\n";
    let score = compile(source, "test.jianpu", &[]).unwrap();
    let svgs = render_svgs(&score).unwrap();
    for svg in &svgs {
        assert!(!svg.contains(r#"<metadata id="jianpu-source">"#));
        assert_eq!(extract_embedded_source(svg), None);
    }
}

#[test]
fn extract_returns_none_when_metadata_missing() {
    assert_eq!(extract_embedded_source("<svg></svg>"), None);
}

#[cfg(feature = "pdf")]
mod pdf_embedding {
    use super::*;
    use crate::source_embed::extract_embedded_source_from_pdf;

    fn test_pdf_fonts() -> pdf::PdfFonts {
        pdf::PdfFonts {
            sans_serif_sc: include_bytes!("../../fonts/SourceHanSansSC-Regular.otf").to_vec(),
            sans_serif_tc: include_bytes!("../../fonts/SourceHanSansTC-Regular.otf").to_vec(),
            monospace: include_bytes!("../../fonts/NotoSansMono-Regular.ttf").to_vec(),
        }
    }

    #[test]
    fn round_trip_recovers_original_source_from_pdf() {
        let source =
            "# metadata\ntitle = \"Testing\"\n\n# parts\nMelody = notes\n\n# score\n[Melody] 1 2 3 4\n";
        let pdf_bytes =
            write_pdf_from_source_filtered(source, "test.jianpu", None, &test_pdf_fonts(), &[])
                .unwrap();
        assert_eq!(
            extract_embedded_source_from_pdf(&pdf_bytes).as_deref(),
            Some(source)
        );
    }

    #[test]
    fn score_only_pdf_has_no_embedded_source() {
        let source =
            "# metadata\ntitle = \"Testing\"\n\n# parts\nMelody = notes\n\n# score\n[Melody] 1 2 3 4\n";
        let score = compile(source, "test.jianpu", &[]).unwrap();
        let svgs = render_svgs(&score).unwrap();
        let pdf_bytes = pdf::write_pdf(&svgs, &test_pdf_fonts(), None).unwrap();
        assert_eq!(extract_embedded_source_from_pdf(&pdf_bytes), None);
    }
}
