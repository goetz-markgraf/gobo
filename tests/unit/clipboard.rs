// Unit tests for clipboard.rs pure functions (spec 009, constitution IV).
// No arboard calls — all inputs are synthetic. Tests the size-limit boundary,
// the is_text_only predicate, and the MAX_CLIPBOARD_BYTES constant.

use gobo::editor::clipboard::{fits_size_limit, is_text_only, MAX_CLIPBOARD_BYTES};

// ---- fits_size_limit ----------------------------------------------------------

#[test]
fn fits_size_limit_zero_is_ok() {
    assert!(fits_size_limit(0));
}

#[test]
fn fits_size_limit_exactly_1mb_is_ok() {
    assert!(fits_size_limit(MAX_CLIPBOARD_BYTES));
    assert!(fits_size_limit(1 << 20));
}

#[test]
fn fits_size_limit_one_over_rejects() {
    assert!(!fits_size_limit(MAX_CLIPBOARD_BYTES + 1));
    assert!(!fits_size_limit(1 << 20 + 1));
}

#[test]
fn fits_size_limit_large_value_rejects() {
    assert!(!fits_size_limit(usize::MAX));
    assert!(!fits_size_limit(10 * (1 << 20)));
}

// ---- is_text_only ------------------------------------------------------------

#[test]
fn is_text_only_ascii_bytes_true() {
    assert!(is_text_only(b"hello world"));
    assert!(is_text_only(b""));
}

#[test]
fn is_text_only_valid_utf8_multibyte_true() {
    // "Hällö Wörld" contains multi-byte UTF-8 sequences
    assert!(is_text_only("Hällö Wörld".as_bytes()));
    // Emoji: 4-byte UTF-8
    assert!(is_text_only("🎉".as_bytes()));
}

#[test]
fn is_text_only_invalid_utf8_false() {
    // 0xFF / 0xFE are not valid UTF-8 lead bytes
    assert!(!is_text_only(b"\xff\xfe"));
    // Truncated 2-byte sequence (0xC3 without continuation)
    assert!(!is_text_only(b"\xc3"));
    // Valid 2-byte prefix followed by invalid continuation
    assert!(!is_text_only(b"\xc3\x00"));
}

// ---- MAX_CLIPBOARD_BYTES constant --------------------------------------------

#[test]
fn max_clipboard_bytes_is_exactly_1mib() {
    assert_eq!(MAX_CLIPBOARD_BYTES, 1_048_576);
}
