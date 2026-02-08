//! POSIX.1-2017 compliance tests for test (and [ alias)
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/test.html

use crate::posix::helpers::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

// =============================================================================
// Exit status tests
// =============================================================================

/// POSIX: "0 - expression evaluated to true"
#[test]
fn posix_test_exit_true() {
    let result = run(&["test", "-n", "nonempty"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "1 - expression evaluated to false or missing"
#[test]
fn posix_test_exit_false() {
    let result = run(&["test", "-n", ""]);
    assert_eq!(result.0, 1);
}

/// POSIX: ">1 - An error occurred"
#[test]
fn posix_test_exit_error() {
    // Too many arguments without valid operator
    let result = run(&["test", "a", "b", "c", "d", "e"]);
    assert!(result.0 > 1);
}

// =============================================================================
// String tests
// =============================================================================

/// POSIX: "-n string - True if the length of string is non-zero"
#[test]
fn posix_test_string_n_nonempty() {
    let result = run(&["test", "-n", "hello"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-n string - False for empty string"
#[test]
fn posix_test_string_n_empty() {
    let result = run(&["test", "-n", ""]);
    assert_eq!(result.0, 1);
}

/// POSIX: "-z string - True if the length of string is zero"
#[test]
fn posix_test_string_z_empty() {
    let result = run(&["test", "-z", ""]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-z string - False for non-empty string"
#[test]
fn posix_test_string_z_nonempty() {
    let result = run(&["test", "-z", "hello"]);
    assert_eq!(result.0, 1);
}

/// POSIX: "s1 = s2 - True if strings are identical"
#[test]
fn posix_test_string_equal() {
    let result = run(&["test", "abc", "=", "abc"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "s1 = s2 - False if strings differ"
#[test]
fn posix_test_string_not_equal() {
    let result = run(&["test", "abc", "=", "xyz"]);
    assert_eq!(result.0, 1);
}

/// POSIX: "s1 != s2 - True if strings are not identical"
#[test]
fn posix_test_string_neq() {
    let result = run(&["test", "abc", "!=", "xyz"]);
    assert_eq!(result.0, 0);
}

// =============================================================================
// Integer comparison tests
// =============================================================================

/// POSIX: "n1 -eq n2 - True if integers are equal"
#[test]
fn posix_test_int_eq() {
    let result = run(&["test", "42", "-eq", "42"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "n1 -ne n2 - True if integers are not equal"
#[test]
fn posix_test_int_ne() {
    let result = run(&["test", "42", "-ne", "43"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "n1 -gt n2 - True if n1 > n2"
#[test]
fn posix_test_int_gt() {
    let result = run(&["test", "10", "-gt", "5"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "n1 -ge n2 - True if n1 >= n2"
#[test]
fn posix_test_int_ge() {
    let result = run(&["test", "10", "-ge", "10"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "n1 -lt n2 - True if n1 < n2"
#[test]
fn posix_test_int_lt() {
    let result = run(&["test", "5", "-lt", "10"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "n1 -le n2 - True if n1 <= n2"
#[test]
fn posix_test_int_le() {
    let result = run(&["test", "10", "-le", "10"]);
    assert_eq!(result.0, 0);
}

// =============================================================================
// File tests
// =============================================================================

/// POSIX: "-e pathname - True if pathname exists"
#[test]
fn posix_test_file_exists() {
    let dir = setup_test_env();
    let file = dir.path().join("exists");
    fs::write(&file, "content").unwrap();

    let result = run(&["test", "-e", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-e pathname - False if pathname does not exist"
#[test]
fn posix_test_file_not_exists() {
    let result = run(&["test", "-e", "/nonexistent/path/file"]);
    assert_eq!(result.0, 1);
}

/// POSIX: "-f pathname - True if pathname is a regular file"
#[test]
fn posix_test_file_regular() {
    let dir = setup_test_env();
    let file = dir.path().join("regular");
    fs::write(&file, "content").unwrap();

    let result = run(&["test", "-f", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-d pathname - True if pathname is a directory"
#[test]
fn posix_test_file_directory() {
    let dir = setup_test_env();
    let result = run(&["test", "-d", dir.path().to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-r pathname - True if pathname is readable"
#[test]
fn posix_test_file_readable() {
    let dir = setup_test_env();
    let file = dir.path().join("readable");
    fs::write(&file, "content").unwrap();

    let result = run(&["test", "-r", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-w pathname - True if pathname is writable"
#[test]
fn posix_test_file_writable() {
    let dir = setup_test_env();
    let file = dir.path().join("writable");
    fs::write(&file, "content").unwrap();

    let result = run(&["test", "-w", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-x pathname - True if pathname is executable"
#[test]
fn posix_test_file_executable() {
    let dir = setup_test_env();
    let file = dir.path().join("executable");
    fs::write(&file, "#!/bin/sh\necho hi").unwrap();
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&file, perms).unwrap();

    let result = run(&["test", "-x", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-s pathname - True if file exists and has size > 0"
#[test]
fn posix_test_file_size_nonzero() {
    let dir = setup_test_env();
    let file = dir.path().join("nonempty");
    fs::write(&file, "content").unwrap();

    let result = run(&["test", "-s", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: "-s pathname - False for empty file"
#[test]
fn posix_test_file_size_zero() {
    let dir = setup_test_env();
    let file = dir.path().join("empty");
    fs::write(&file, "").unwrap();

    let result = run(&["test", "-s", file.to_str().unwrap()]);
    assert_eq!(result.0, 1);
}

// =============================================================================
// Logical operators
// =============================================================================

/// POSIX: "! expression - True if expression is false"
#[test]
fn posix_test_not() {
    let result = run(&["test", "!", "-n", ""]);
    assert_eq!(result.0, 0);
}

/// POSIX: "( expression ) - Grouping"
#[test]
fn posix_test_grouping() {
    let result = run(&["test", "(", "-n", "hello", ")"]);
    assert_eq!(result.0, 0);
}

// =============================================================================
// [ alias tests
// =============================================================================

/// POSIX: "[ is equivalent to test"
#[test]
fn posix_bracket_alias() {
    let result = run(&["[", "-n", "hello", "]"]);
    assert_eq!(result.0, 0);
}

/// POSIX: "[ requires ] as last argument"
#[test]
fn posix_bracket_requires_closing() {
    let result = run(&["[", "-n", "hello"]);
    assert!(result.0 > 0);
}
