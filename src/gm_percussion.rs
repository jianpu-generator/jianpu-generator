#[derive(serde::Deserialize)]
struct GmPercussionEntry {
    key: u8,
    name: String,
}

/// GM percussion key names (program numbers 35-81), per the General MIDI
/// percussion map. Loaded from the JSON data file shared with the
/// TypeScript side (`web/src/utils/gmPercussion.ts`) so the two never drift.
static GM_PERCUSSION_NAMES: std::sync::LazyLock<Vec<GmPercussionEntry>> =
    std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../web/src/data/gmPercussion.json")).unwrap_or_default()
    });

pub(crate) fn percussion_program_to_label(program: u8) -> String {
    GM_PERCUSSION_NAMES
        .iter()
        .find(|entry| entry.key == program)
        .map_or_else(
            || format!("{program}: Unknown"),
            |entry| format!("{}: {}", entry.key, entry.name),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gm_percussion_json_parses_and_resolves_known_key() {
        assert!(!GM_PERCUSSION_NAMES.is_empty());
        assert_eq!(percussion_program_to_label(38), "38: Acoustic Snare");
    }
}
