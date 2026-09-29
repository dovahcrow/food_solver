//! The stamped build provenance must always be readable.

use food_core::{BuildInfo, BUILD_INFO};

/// True when `text` is `YYYY-MM-DDTHH:MM:SSZ` with digits in every slot.
fn is_iso8601_utc(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 20 {
        return false;
    }
    let digits = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18];
    digits.iter().all(|index| bytes[*index].is_ascii_digit())
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
}

#[test]
fn build_date_is_iso8601_utc() {
    assert!(
        is_iso8601_utc(BUILD_INFO.build_date),
        "unexpected build date {:?}",
        BUILD_INFO.build_date
    );
}

#[test]
fn git_sha_is_a_revision_or_explicitly_unknown() {
    let sha = BUILD_INFO.git_sha;
    assert!(!sha.is_empty(), "git sha must never be empty");
    let is_hex = sha.chars().all(|c| c.is_ascii_hexdigit());
    assert!(
        sha == "unknown" || is_hex,
        "git sha {sha:?} is neither a hex revision nor `unknown`"
    );
}

#[test]
fn display_names_both_fields() {
    let text = BUILD_INFO.to_string();
    assert!(text.contains(BUILD_INFO.build_date), "{text}");
    assert!(text.contains(BUILD_INFO.git_sha), "{text}");
}

#[test]
fn is_copy_and_comparable() {
    // A struct literal is Copy, so callers can take it by value.
    let copied: BuildInfo = BUILD_INFO;
    assert_eq!(copied, BUILD_INFO);
}
