//! POSIX.1-2017 compliance tests for fold
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/fold.html

use crate::posix::helpers::*;

/// POSIX: "The fold utility shall fold long lines in input files"
#[test]
fn posix_fold_default() {
    let long_line = "x".repeat(100);
    let result = run_with_stdin(&["fold"], long_line.as_bytes());
    assert_success(&result);
    // Default is 80 columns
    let lines: Vec<&str> = result.1.lines().collect();
    assert!(lines.len() >= 2);
}

/// POSIX: fold -w width
#[test]
fn posix_fold_width() {
    let result = run_with_stdin(&["fold", "-w", "10"], b"hello world this is a test");
    assert_success(&result);
    // Lines should be max 10 chars
    for line in result.1.lines() {
        assert!(line.len() <= 10);
    }
}

/// POSIX: fold -s break at spaces
#[test]
fn posix_fold_spaces() {
    let result = run_with_stdin(&["fold", "-s", "-w", "10"], b"hello world test");
    assert_success(&result);
    // Should break at word boundaries when possible
    for line in result.1.lines() {
        assert!(line.len() <= 10);
    }
}

/// POSIX: fold -b count bytes
#[test]
fn posix_fold_bytes() {
    let result = run_with_stdin(&["fold", "-b", "-w", "5"], b"hello world");
    assert_success(&result);
    // Should fold at 5 bytes
    let first_line = result.1.lines().next().unwrap();
    assert_eq!(first_line.len(), 5);
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_fold_exit_success() {
    let result = run_with_stdin(&["fold"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: fold short lines unchanged
#[test]
fn posix_fold_short_unchanged() {
    let result = run_with_stdin(&["fold", "-w", "80"], b"short line");
    assert_success(&result);
    assert_eq!(result.1, "short line\n");
}
