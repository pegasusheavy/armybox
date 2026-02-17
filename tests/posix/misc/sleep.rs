//! POSIX.1-2017 compliance tests for sleep
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/sleep.html

use crate::posix::helpers::*;
use std::time::Instant;

/// POSIX: "The sleep utility shall suspend execution for at least the integral number of seconds"
#[test]
fn posix_sleep_basic() {
    let start = Instant::now();
    let result = run(&["sleep", "1"]);
    let elapsed = start.elapsed();

    assert_success(&result);
    assert!(elapsed.as_millis() >= 900); // Allow some tolerance
}

/// POSIX: sleep with fractional seconds (extension)
#[test]
fn posix_sleep_fractional() {
    let start = Instant::now();
    let result = run(&["sleep", "0.1"]);
    let elapsed = start.elapsed();

    // May or may not support fractional seconds
    if result.0 == 0 {
        assert!(elapsed.as_millis() >= 50);
    }
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_sleep_exit_success() {
    let result = run(&["sleep", "0"]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_sleep_exit_error() {
    let result = run(&["sleep", "-1"]);
    assert!(result.0 > 0);
}
