//! Three-way text merge.
use crate::types::MergeOutcome;

/// Merges `mine` and `theirs` relative to `base`. A conflict-free merge is
/// only `Clean` when `accepts` approves the merged text.
pub fn merge_three_way(
    base: &str,
    mine: &str,
    theirs: &str,
    accepts: impl Fn(&str) -> bool,
) -> MergeOutcome {
    match diffy::merge(base, mine, theirs) {
        Ok(text) if accepts(&text) => MergeOutcome::Clean { text },
        Ok(text) => MergeOutcome::Conflicted {
            text,
            has_markers: false,
        },
        Err(text) => MergeOutcome::Conflicted {
            text,
            has_markers: true,
        },
    }
}
