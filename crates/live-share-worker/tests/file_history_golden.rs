//! Stored reverse patches outlive a release: each fixture under
//! `tests/fixtures/file_history/` holds a patch written by the diff library
//! version that first shipped, and must keep applying after an upgrade.
#![allow(clippy::disallowed_macros)]
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use live_share_worker::file_history::apply_patch;

#[test]
fn every_committed_patch_still_rebuilds_its_older_text() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/file_history");
    let cases: Vec<_> = std::fs::read_dir(&root)
        .expect("fixtures directory")
        .map(|entry| entry.expect("fixture entry").path())
        .collect();
    assert!(!cases.is_empty());
    for case in cases {
        let read = |name: &str| std::fs::read_to_string(case.join(name)).expect(name);
        assert_eq!(
            apply_patch(&read("reverse.patch"), &read("newer.txt")),
            Ok(read("older.txt")),
            "{}",
            case.display()
        );
    }
}
