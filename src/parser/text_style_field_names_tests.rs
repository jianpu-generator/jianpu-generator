//! Cross-boundary guard for item 4 of `TODO-cross-boundary-invariants.md`:
//! `web/src/utils/textStyleFields.ts` hand-mirrors the `TextStyle` kind/field
//! names and `font_family` values this parser matches on as bare string
//! literals. Nothing on either side enforces the two stay in sync, so these
//! tests read the TS source directly and fail the moment a name is added,
//! removed, or typo'd differently on one side.

use std::collections::BTreeSet;

const TEXT_STYLE_FIELDS_TS: &str = include_str!("../../web/src/utils/textStyleFields.ts");

/// Pulls the quoted string literals out of a TS `export const <name> = [ ... ] as const`
/// array. Deliberately naive (no real TS parsing) — good enough for this file's flat,
/// single-quoted string arrays, and fails loudly (via `expect`) rather than silently
/// returning nothing if the array's shape ever changes enough to break this scan.
fn extract_ts_string_array(source: &str, const_name: &str) -> BTreeSet<String> {
    let marker = format!("export const {const_name} = [");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("`{const_name}` not found in textStyleFields.ts"))
        + marker.len();
    let end = source[start..]
        .find(']')
        .unwrap_or_else(|| panic!("no closing `]` found for `{const_name}`"))
        + start;
    source[start..end]
        .split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            entry
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
                .map(str::to_owned)
        })
        .collect()
}

#[test]
fn text_style_kind_names_match_ts() {
    let rust_names: BTreeSet<String> = super::super::TEXT_STYLE_KIND_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    let ts_names = extract_ts_string_array(TEXT_STYLE_FIELDS_TS, "textStyleKinds");
    assert_eq!(
        rust_names, ts_names,
        "metadata_parser.rs's TEXT_STYLE_KIND_NAMES and textStyleFields.ts's \
         textStyleKinds have diverged — see item 4 of TODO-cross-boundary-invariants.md"
    );
}

#[test]
fn text_style_field_names_match_ts() {
    let rust_names: BTreeSet<String> = super::TEXT_STYLE_FIELD_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    let ts_names: BTreeSet<String> =
        extract_ts_string_array(TEXT_STYLE_FIELDS_TS, "textStyleNumericComponents")
            .into_iter()
            .chain(extract_ts_string_array(
                TEXT_STYLE_FIELDS_TS,
                "textStyleBooleanComponents",
            ))
            .chain(extract_ts_string_array(
                TEXT_STYLE_FIELDS_TS,
                "textStyleEnumComponents",
            ))
            .collect();
    assert_eq!(
        rust_names, ts_names,
        "text_style_parser.rs's TEXT_STYLE_FIELD_NAMES and textStyleFields.ts's \
         numeric/boolean/enum component lists have diverged — see item 4 of \
         TODO-cross-boundary-invariants.md"
    );
}

#[test]
fn font_family_choice_names_match_ts() {
    let rust_names: BTreeSet<String> = super::super::FONT_FAMILY_CHOICE_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    let ts_names = extract_ts_string_array(TEXT_STYLE_FIELDS_TS, "fontFamilyValues");
    assert_eq!(
        rust_names, ts_names,
        "metadata_parser.rs's FONT_FAMILY_CHOICE_NAMES and textStyleFields.ts's \
         fontFamilyValues have diverged — see item 4 of TODO-cross-boundary-invariants.md"
    );
}
