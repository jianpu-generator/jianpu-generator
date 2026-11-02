//! Pure, D1-free logic for a cloud file's version history (the
//! `file_versions` table, see `live-share-worker/migrations/0008_file_versions.sql`).
//!
//! `files.content` always holds the full current text (the head). Every
//! accepted content save that changes the text also stores a **reverse
//! patch**: a unified diff that turns the new text back into the previous
//! one. An old version is rebuilt by applying patches backward from the head.
//!
//! Each row names the revision it rebuilds (`revision`) and the revision its
//! patch applies to (`newer_revision`), so the chain is explicit: a save that
//! leaves the text unchanged moves the newest row's `newer_revision` forward
//! instead of storing an empty patch, and a row whose save failed to record
//! leaves a gap that `reachable_versions` stops at. A gap hides older history
//! rather than rebuilding wrong text.

use std::collections::HashMap;

/// How many versions of a file are kept; older ones are pruned on each save.
pub const MAX_VERSIONS_PER_FILE: i64 = 100;

/// Mirrors a `file_versions` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRow {
    /// The revision this row rebuilds.
    pub revision: i64,
    /// The revision `reverse_patch` applies to.
    pub newer_revision: i64,
    /// Unified diff from the text at `newer_revision` to the text at `revision`.
    pub reverse_patch: String,
    /// When the text at `revision` was saved (epoch millis).
    pub saved_at: i64,
}

/// A patch that failed to parse or apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchError {
    pub message: String,
}

/// The unified diff that turns `newer` back into `older`.
pub fn reverse_patch(newer: &str, older: &str) -> String {
    diffy::create_patch(newer, older).to_string()
}

/// Applies a unified diff made by `reverse_patch` to `text`.
pub fn apply_patch(patch: &str, text: &str) -> Result<String, PatchError> {
    let parsed = diffy::Patch::from_str(patch).map_err(|error| PatchError {
        message: error.to_string(),
    })?;
    diffy::apply(text, &parsed).map_err(|error| PatchError {
        message: error.to_string(),
    })
}

/// The rows reachable from the head by following `newer_revision` links,
/// newest first. Stops at the first gap.
pub fn reachable_versions(head_revision: i64, rows: &[VersionRow]) -> Vec<&VersionRow> {
    let by_newer: HashMap<i64, &VersionRow> =
        rows.iter().map(|row| (row.newer_revision, row)).collect();
    std::iter::successors(by_newer.get(&head_revision).copied(), |row| {
        by_newer.get(&row.revision).copied()
    })
    .collect()
}

/// The text of `revision`, rebuilt from the head. `None` when `revision` is
/// not reachable from the head.
pub fn rebuild(
    head_content: &str,
    head_revision: i64,
    rows: &[VersionRow],
    revision: i64,
) -> Option<Result<String, PatchError>> {
    let chain = reachable_versions(head_revision, rows);
    let steps = chain.iter().position(|row| row.revision == revision)? + 1;
    Some(
        chain
            .iter()
            .take(steps)
            .try_fold(head_content.to_string(), |text, row| {
                apply_patch(&row.reverse_patch, &text)
            }),
    )
}
