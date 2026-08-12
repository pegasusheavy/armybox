//! POSIX.1-2017 compliance tests for nl
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/nl.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The nl utility shall read lines from input and write them to standard output with line numbers"
#[test]
fn posix_nl_basic() {
    let result = run_with_stdin(&["nl"], b"hello\nworld\n");
    assert_success(&result);
    assert!(result.1.contains("1") && result.1.contains("hello"));
    assert!(result.1.contains("2") && result.1.contains("world"));
}

/// POSIX: nl -b body numbering style
#[test]
fn posix_nl_body_all() {
    let result = run_with_stdin(&["nl", "-ba"], b"hello\n\nworld\n");
    assert_success(&result);
    // All lines numbered, including the blank line (counter advances on it).
    assert_eq!(result.1, "     1\thello\n     2\t\n     3\tworld\n");
}

/// POSIX: nl -b t (only non-empty lines)
#[test]
fn posix_nl_body_text() {
    let result = run_with_stdin(&["nl", "-bt"], b"hello\n\nworld\n");
    assert_success(&result);
    // Blank line is not numbered; the counter does not advance for it, so
    // "world" is line 2, and the blank line is emitted with no number.
    assert_eq!(result.1, "     1\thello\n\n     2\tworld\n");
}

/// nl -v START -i INCR: custom start and increment.
#[test]
fn posix_nl_start_increment() {
    let result = run_with_stdin(&["nl", "-v100", "-i5"], b"a\nb\nc\n");
    assert_success(&result);
    // Numbering starts at 100 and increments by 5: 100, 105, 110.
    assert_eq!(result.1, "   100\ta\n   105\tb\n   110\tc\n");
}

/// POSIX: nl -n format
#[test]
fn posix_nl_format_rz() {
    let result = run_with_stdin(&["nl", "-n", "rz"], b"hello\n");
    assert_success(&result);
    // Right justified with leading zeros
    assert!(result.1.contains("000001"));
}

/// POSIX: nl -w width
#[test]
fn posix_nl_width() {
    let result = run_with_stdin(&["nl", "-w", "3"], b"hello\n");
    assert_success(&result);
    // Line number field width is 3, right-justified, then a TAB separator.
    assert_eq!(result.1, "  1\thello\n");
}

/// nl rejects an unparsable -w width with a usage error (exit 2).
#[test]
fn posix_nl_invalid_width() {
    let result = run_with_stdin(&["nl", "-w", "abc"], b"hello\n");
    assert_eq!(result.0, 2);
}

/// POSIX: nl -s separator
#[test]
fn posix_nl_separator() {
    let result = run_with_stdin(&["nl", "-s", ": "], b"hello\n");
    assert_success(&result);
    assert!(result.1.contains(": hello"));
}

/// POSIX: nl from file
#[test]
fn posix_nl_from_file() {
    let dir = setup_test_env();
    let file = dir.path().join("input");
    fs::write(&file, "hello\nworld\n").unwrap();

    let result = run(&["nl", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("hello"));
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_nl_exit_success() {
    let result = run_with_stdin(&["nl"], b"test");
    assert_eq!(result.0, 0);
}
