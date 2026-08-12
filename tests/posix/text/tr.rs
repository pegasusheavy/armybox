//! POSIX.1-2017 compliance tests for tr
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/tr.html

use crate::posix::helpers::*;

/// POSIX: "The tr utility shall copy standard input to standard output with substitution or deletion"
#[test]
fn posix_tr_basic() {
    let result = run_with_stdin(&["tr", "abc", "xyz"], b"aabbcc");
    assert_success(&result);
    assert_eq!(result.1, "xxyyzz");
}

/// POSIX: tr -d delete characters
#[test]
fn posix_tr_delete() {
    let result = run_with_stdin(&["tr", "-d", "aeiou"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hll wrld");
}

/// POSIX: tr -s squeeze repeated characters
#[test]
fn posix_tr_squeeze() {
    let result = run_with_stdin(&["tr", "-s", " "], b"hello    world");
    assert_success(&result);
    assert_eq!(result.1, "hello world");
}

/// POSIX: tr -c complement of set1
#[test]
fn posix_tr_complement() {
    let result = run_with_stdin(&["tr", "-cd", "a-z"], b"Hello123World");
    assert_success(&result);
    assert_eq!(result.1, "elloorld");
}

/// POSIX: tr character class [:lower:]
#[test]
fn posix_tr_class_lower() {
    let result = run_with_stdin(&["tr", "[:lower:]", "[:upper:]"], b"hello");
    assert_success(&result);
    assert_eq!(result.1, "HELLO");
}

/// POSIX: tr character class [:upper:]
#[test]
fn posix_tr_class_upper() {
    let result = run_with_stdin(&["tr", "[:upper:]", "[:lower:]"], b"HELLO");
    assert_success(&result);
    assert_eq!(result.1, "hello");
}

/// POSIX: tr character class [:digit:]
#[test]
fn posix_tr_class_digit() {
    let result = run_with_stdin(&["tr", "-d", "[:digit:]"], b"abc123def456");
    assert_success(&result);
    assert_eq!(result.1, "abcdef");
}

/// POSIX: tr character class [:space:]
#[test]
fn posix_tr_class_space() {
    let result = run_with_stdin(&["tr", "-s", "[:space:]"], b"hello   \t\n  world");
    assert_success(&result);
    // -s only squeezes runs of the *same* character; a run mixing distinct
    // whitespace members (space, tab, newline) collapses each distinct
    // member's run but does not merge different members together.
    assert_eq!(result.1, "hello \t\n world");
}

/// POSIX: tr range a-z
#[test]
fn posix_tr_range() {
    let result = run_with_stdin(&["tr", "a-z", "A-Z"], b"hello");
    assert_success(&result);
    assert_eq!(result.1, "HELLO");
}

/// POSIX: tr -ds delete and squeeze
#[test]
fn posix_tr_delete_squeeze() {
    let result = run_with_stdin(&["tr", "-ds", "aeiou", " "], b"hello  world");
    assert_success(&result);
    // Deletes vowels and squeezes spaces
    assert_eq!(result.1, "hll wrld");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_tr_exit_success() {
    let result = run_with_stdin(&["tr", "a", "b"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: tr truncation when set2 shorter than set1
#[test]
fn posix_tr_truncate() {
    let result = run_with_stdin(&["tr", "abc", "x"], b"aabbcc");
    assert_success(&result);
    // set2 is extended with last character
    assert_eq!(result.1, "xxxxxx");
}

/// POSIX: tr -t truncate set1 to set2 length
#[test]
fn posix_tr_truncate_flag() {
    let result = run_with_stdin(&["tr", "-t", "abc", "x"], b"aabbcc");
    assert_success(&result);
    // Only 'a' is translated, b and c unchanged
    assert_eq!(result.1, "xxbbcc");
}

/// POSIX: tr escape sequences
#[test]
fn posix_tr_escapes() {
    let result = run_with_stdin(&["tr", "\\n", "X"], b"hello\nworld");
    assert_success(&result);
    // Newline translated to X
    assert_eq!(result.1, "helloXworld");
}
