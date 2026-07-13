use super::*;
use std::io::Read;
use zip::ZipArchive;

fn test_pdf_fonts() -> pdf::PdfFonts {
    pdf::PdfFonts {
        sans_serif_sc: include_bytes!("../../fonts/SourceHanSansSC-Regular.otf").to_vec(),
        sans_serif_tc: include_bytes!("../../fonts/SourceHanSansTC-Regular.otf").to_vec(),
        monospace: include_bytes!("../../fonts/NotoSansMono-Regular.ttf").to_vec(),
    }
}

fn multi_track_input() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"test score\"\n",
        "author = \"tester\"\n",
        "\n",
        "# parts\n",
        "Soprano 1 [S1] = notes+lyrics\n",
        "Soprano 2 [S2] = notes+lyrics\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[S1] 1 2 3 4\n",
        "[S1] do re mi fa\n",
        "[S2] 5 6 7 1\n",
        "[S2] sol la ti do\n",
    )
}

#[test]
fn write_split_pdfs_from_source_produces_one_pdf_per_track() {
    let entries = write_split_pdfs_from_source(
        multi_track_input(),
        "test.jianpu",
        "test_split",
        &[],
        &test_pdf_fonts(),
    )
    .unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].track_name, "S1");
    assert_eq!(entries[0].filename, "test_split - Soprano 1.pdf");
    assert_eq!(entries[1].track_name, "S2");
    assert_eq!(entries[1].filename, "test_split - Soprano 2.pdf");
    assert_eq!(&entries[0].pdf[0..4], b"%PDF");
    assert_eq!(&entries[1].pdf[0..4], b"%PDF");
}

#[test]
fn write_split_pdfs_from_source_single_part_uses_split_naming() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes+lyrics\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 4\n",
        "[Melody] a b c d\n",
    );
    let entries =
        write_split_pdfs_from_source(input, "test.jianpu", "song", &[], &test_pdf_fonts()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].filename, "song - Melody.pdf");
    assert_eq!(&entries[0].pdf[0..4], b"%PDF");
}

#[test]
fn write_split_pdfs_from_source_no_sections_returns_empty() {
    // Source with no section headers: section-structure errors are recoverable,
    // so no Err is returned; the missing # parts section means no tracks exist.
    let entries =
        write_split_pdfs_from_source("not valid", "test.jianpu", "song", &[], &test_pdf_fonts())
            .unwrap();
    assert!(entries.is_empty());
}

#[test]
fn zip_split_pdfs_contains_named_entries() {
    let entries = write_split_pdfs_from_source(
        multi_track_input(),
        "test.jianpu",
        "test_split",
        &[],
        &test_pdf_fonts(),
    )
    .unwrap();
    let zip_bytes = zip_split_pdfs(&entries).unwrap();
    assert_eq!(&zip_bytes[0..2], b"PK");

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    assert_eq!(archive.len(), 2);
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "test_split - Soprano 1.pdf".to_string(),
            "test_split - Soprano 2.pdf".to_string()
        ]
    );

    let mut first = archive.by_name("test_split - Soprano 1.pdf").unwrap();
    let mut buf = Vec::new();
    first.read_to_end(&mut buf).unwrap();
    assert_eq!(&buf[0..4], b"%PDF");
}
