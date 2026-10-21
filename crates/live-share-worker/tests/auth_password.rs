//! Unit tests for `auth::password`: the length rule, the stored hash's text
//! form, and the comparison. The PBKDF2 derivation itself runs on WebCrypto
//! (`crate::crypto`), so it's covered by the e2e sign-in scenarios instead.
#![allow(clippy::disallowed_macros)]

use live_share_worker::auth::password::{
    constant_time_eq, password_weakness, PasswordHash, MIN_PASSWORD_LENGTH,
};

#[test]
fn a_password_needs_at_least_twelve_characters_and_nothing_else() {
    assert_eq!(MIN_PASSWORD_LENGTH, 12);
    assert_eq!(
        password_weakness("elevenchars"),
        Some("Use at least 12 characters.".to_string())
    );
    assert_eq!(password_weakness("twelve chars"), None);
    assert_eq!(password_weakness("aaaaaaaaaaaa"), None);
}

#[test]
fn counts_characters_not_bytes() {
    assert_eq!(
        password_weakness("简谱简谱简谱简谱简谱简"),
        Some("Use at least 12 characters.".to_string())
    );
    assert_eq!(password_weakness("简谱简谱简谱简谱简谱简谱"), None);
}

fn sample_hash() -> PasswordHash {
    PasswordHash {
        iterations: 100_000,
        salt: vec![1, 2, 3, 4],
        hash: vec![250, 251, 252],
    }
}

#[test]
fn encodes_as_scheme_iterations_salt_and_hash() {
    assert_eq!(sample_hash().encode(), "pbkdf2-sha256$100000$AQIDBA==$+vv8");
}

#[test]
fn parses_what_it_encodes() {
    assert_eq!(
        PasswordHash::parse(&sample_hash().encode()),
        Some(sample_hash())
    );
}

#[test]
fn refuses_to_parse_anything_else() {
    let malformed = [
        "",
        "bcrypt$100000$AQIDBA==$+vv8",
        "pbkdf2-sha256$0$AQIDBA==$+vv8",
        "pbkdf2-sha256$many$AQIDBA==$+vv8",
        "pbkdf2-sha256$100000$not base64$+vv8",
        "pbkdf2-sha256$100000$AQIDBA==",
        "pbkdf2-sha256$100000$AQIDBA==$+vv8$extra",
    ];

    for encoded in malformed {
        assert_eq!(PasswordHash::parse(encoded), None, "{encoded:?}");
    }
}

#[test]
fn matches_only_the_same_derived_bytes() {
    let hash = sample_hash();

    assert!(hash.matches(&[250, 251, 252]));
    assert!(!hash.matches(&[250, 251, 253]));
    assert!(!hash.matches(&[250, 251]));
}

#[test]
fn constant_time_eq_compares_whole_slices() {
    assert!(constant_time_eq(&[], &[]));
    assert!(constant_time_eq(&[7, 8], &[7, 8]));
    assert!(!constant_time_eq(&[7, 8], &[8, 8]));
    assert!(!constant_time_eq(&[7, 8], &[7, 8, 9]));
}
