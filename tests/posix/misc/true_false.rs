//! POSIX.1-2017 compliance tests for true and false
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/true.html
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/false.html

use crate::posix::helpers::*;

// =============================================================================
// true utility tests
// =============================================================================

/// POSIX: "The true utility shall return with exit code zero."
#[test]
fn posix_true_exit_zero() {
    let result = run(&["true"]);
    assert_eq!(result.0, 0, "true must exit with code 0");
}

/// POSIX: true shall produce no output
#[test]
fn posix_true_no_output() {
    let result = run(&["true"]);
    assert_stdout_empty(&result);
    assert_stderr_empty(&result);
}

// =============================================================================
// false utility tests
// =============================================================================

/// POSIX: "The false utility shall return with a non-zero exit code."
#[test]
fn posix_false_exit_nonzero() {
    let result = run(&["false"]);
    assert!(result.0 != 0, "false must exit with non-zero code");
}

/// POSIX: false shall produce no output
#[test]
fn posix_false_no_output() {
    let result = run(&["false"]);
    assert_stdout_empty(&result);
    assert_stderr_empty(&result);
}
