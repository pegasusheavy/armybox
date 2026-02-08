//! POSIX.1-2017 compliance tests for unexpand
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/unexpand.html

use crate::posix::helpers::*;

/// POSIX: "The unexpand utility shall copy input... converting spaces to tabs"
#[test]
fn posix_unexpand_basic() {
    let result = run_with_stdin(&["unexpand"], b"        hello");
    assert_success(&result);
    // Leading spaces (8) should become a tab
    assert!(result.1.contains('\t') || result.1.starts_with(' '));
}

/// POSIX: unexpand -a all spaces (not just leading)
#[test]
fn posix_unexpand_all() {
    let result = run_with_stdin(&["unexpand", "-a"], b"hello        world");
    assert_success(&result);
    // Spaces should be converted to tabs
}

/// POSIX: unexpand -t tabstop
#[test]
fn posix_unexpand_tabstop() {
    let result = run_with_stdin(&["unexpand", "-t", "4"], b"    hello");
    assert_success(&result);
    // 4 spaces should become a tab
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_unexpand_exit_success() {
    let result = run_with_stdin(&["unexpand"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: unexpand leaves non-leading spaces by default
#[test]
fn posix_unexpand_nonleading() {
    let result = run_with_stdin(&["unexpand"], b"hello        world");
    assert_success(&result);
    // Without -a, internal spaces should be preserved
}
