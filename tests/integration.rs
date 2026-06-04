use std::fs;
use std::process::Command;

#[test]
fn full_pipeline_produces_pdf() {
    let input = concat!(
        "[metadata]\n",
        "title = \"test score\"\n",
        "author = \"tester\"\n",
        "parts = notes: lyrics:\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "do re mi fa\n",
    );

    let input_path = "/tmp/test_score.jianpu";
    let output_path = "/tmp/test_score.pdf";

    fs::write(input_path, input).unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_jianpu"))
        .arg(input_path)
        .arg("--output")
        .arg(output_path)
        .status()
        .unwrap();

    assert!(status.success(), "jianpu command failed");

    let pdf_bytes = fs::read(output_path).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF"), "output is not a valid PDF");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}
