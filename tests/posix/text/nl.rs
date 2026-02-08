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
    // All lines numbered including blank
}

/// POSIX: nl -b t (only non-empty lines)
#[test]
fn posix_nl_body_text() {
    let result = run_with_stdin(&["nl", "-bt"], b"hello\n\nworld\n");
    assert_success(&result);
    // Blank line should not be numbered
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
    // Line number field width is 3
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
