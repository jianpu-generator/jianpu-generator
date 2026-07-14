#![allow(clippy::disallowed_macros)]
use std::fs;
use std::process::Command;

fn jianpu_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jianpu"))
}

#[test]
fn check_succeeds_on_valid_file() {
    let input_path = "/tmp/test_check_valid.jianpu";
    fs::write(
        input_path,
        r#"# parts
Melody = notes

# score
[Melody] 1 2 3 4
"#,
    )
    .unwrap();

    let output = jianpu_cmd().args(["check", input_path]).output().unwrap();

    fs::remove_file(input_path).ok();
    assert!(
        output.status.success(),
        "expected success, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_fails_on_recoverable_error() {
    let input_path = "/tmp/test_check_recoverable_error.jianpu";
    fs::write(
        input_path,
        r#"# parts
Melody = notes

# score
[Unknown] 1 2 3 4
"#,
    )
    .unwrap();

    let output = jianpu_cmd().args(["check", input_path]).output().unwrap();

    fs::remove_file(input_path).ok();
    assert!(
        !output.status.success(),
        "expected failure for unrecognised abbreviation"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown"));
}

#[test]
fn check_fails_on_irrecoverable_error() {
    let input_path = "/tmp/test_check_irrecoverable_error.jianpu";
    fs::write(input_path, "# score\n[Melody] 1 2 3 4\n").unwrap();

    let output = jianpu_cmd().args(["check", input_path]).output().unwrap();

    fs::remove_file(input_path).ok();
    assert!(
        !output.status.success(),
        "expected failure for missing # parts section"
    );
}
