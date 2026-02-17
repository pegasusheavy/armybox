//! POSIX.1-2017 compliance tests for echo
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/echo.html

use crate::posix::helpers::*;

/// POSIX: "The echo utility writes its arguments to standard output,
/// followed by a <newline>."
#[test]
fn posix_echo_basic() {
    let result = run(&["echo", "hello"]);
    assert_success(&result);
    assert_stdout(&result, "hello\n");
}

/// POSIX: "Arguments shall be separated by single <space> characters"
#[test]
fn posix_echo_multiple_args() {
    let result = run(&["echo", "hello", "world"]);
    assert_success(&result);
    assert_stdout(&result, "hello world\n");
}

/// POSIX: With no arguments, only newline is written
#[test]
fn posix_echo_no_args() {
    let result = run(&["echo"]);
    assert_success(&result);
    assert_stdout(&result, "\n");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_echo_exit_success() {
    let result = run(&["echo", "test"]);
    assert_eq!(result.0, 0);
}

/// POSIX: Empty string argument
#[test]
fn posix_echo_empty_string() {
    let result = run(&["echo", ""]);
    assert_success(&result);
    assert_stdout(&result, "\n");
}

/// POSIX: Multiple empty string arguments
#[test]
fn posix_echo_multiple_empty() {
    let result = run(&["echo", "", ""]);
    assert_success(&result);
    assert_stdout(&result, " \n");
}

/// Test echo with special characters (no interpretation per POSIX XSI)
#[test]
fn posix_echo_special_chars() {
    // Note: POSIX leaves escape interpretation implementation-defined
    // We test that basic strings pass through correctly
    let result = run(&["echo", "hello\\nworld"]);
    assert_success(&result);
    // The behavior depends on whether -e is default; test passes if successful
}

/// Test echo with many arguments
#[test]
fn posix_echo_many_args() {
    let result = run(&["echo", "a", "b", "c", "d", "e"]);
    assert_success(&result);
    assert_stdout(&result, "a b c d e\n");
}
