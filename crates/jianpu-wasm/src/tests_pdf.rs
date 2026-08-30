use crate::responses::{generate_pdf_response, generate_split_pdfs_response};
use crate::types::{GeneratePdfResponse, GenerateSplitPdfsResponse};

fn test_pdf_fonts() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    (
        jianpu_generator::fonts::SERIF_FONT_BYTES.to_vec(),
        jianpu_generator::fonts::SANS_SERIF_FONT_BYTES.to_vec(),
        jianpu_generator::fonts::MONOSPACE_FONT_BYTES.to_vec(),
    )
}

#[test]
fn reference_jianpu_generates_pdf() {
    let (sc, tc, mono) = test_pdf_fonts();
    for path in super::demo_file_paths() {
        let source = super::read_demo_file(&path);
        let resp = generate_pdf_response(&source, None, None, sc.clone(), tc.clone(), mono.clone());
        match resp {
            GeneratePdfResponse::Ok { pdf } => {
                assert!(pdf.len() > 4);
                assert_eq!(&pdf[0..4], b"%PDF");
            }
            GeneratePdfResponse::Err { diagnostics } => {
                panic!(
                    "{path:?} failed in wasm pdf path: {}",
                    diagnostics[0].message
                );
            }
        }
    }
}

#[test]
fn reference_jianpu_generates_split_pdf_zip() {
    use std::io::Read;
    use zip::ZipArchive;

    let (sc, tc, mono) = test_pdf_fonts();
    for path in super::demo_file_paths() {
        let source = super::read_demo_file(&path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("demo");
        let resp =
            generate_split_pdfs_response(&source, stem, sc.clone(), tc.clone(), mono.clone());
        match resp {
            GenerateSplitPdfsResponse::Ok { zip } => {
                assert!(zip.len() > 4);
                assert_eq!(&zip[0..2], b"PK");
                let cursor = std::io::Cursor::new(zip);
                let mut archive = ZipArchive::new(cursor).unwrap();
                assert!(!archive.is_empty());
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let name = file.name().to_string();
                    assert!(
                        name.starts_with(&format!("{stem} - ")) && name.ends_with(".pdf"),
                        "unexpected zip entry: {name}"
                    );
                    let mut buf = [0u8; 4];
                    file.read_exact(&mut buf).unwrap();
                    assert_eq!(&buf, b"%PDF");
                }
            }
            GenerateSplitPdfsResponse::Err { diagnostics } => {
                panic!(
                    "{path:?} failed in wasm split pdf path: {}",
                    diagnostics[0].message
                );
            }
        }
    }
}
