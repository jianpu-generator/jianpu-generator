//! Unit tests for `parse_github_user_ids`, the parser for the test-only
//! `POST /e2e/reset` route's allowlist var.
#![allow(clippy::disallowed_macros)]

use live_share_worker::e2e_reset::parse_github_user_ids;

#[test]
fn parses_a_json_array_of_github_user_ids_as_provider_user_ids() {
    assert_eq!(
        parse_github_user_ids("[987654321,987654322]").unwrap(),
        vec!["987654321".to_string(), "987654322".to_string()],
    );
}

#[test]
fn rejects_a_value_that_is_not_a_json_array_of_numbers() {
    assert!(parse_github_user_ids("987654321,987654322").is_err());
    assert!(parse_github_user_ids(r#"["e2e-test-user"]"#).is_err());
}
