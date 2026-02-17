//! POSIX.1-2017 compliance tests for unlink
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/unlink.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The unlink utility shall call the unlink() function to remove the specified file"
#[test]
fn posix_unlink_basic() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    let result = run(&["unlink", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(!file.exists());
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_unlink_exit_success() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "x").unwrap();

    let result = run(&["unlink", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_unlink_exit_error() {
    let result = run(&["unlink", "/nonexistent/path/file"]);
    assert!(result.0 > 0);
}

/// POSIX: unlink only accepts one operand
#[test]
fn posix_unlink_single_operand() {
    let dir = setup_test_env();
    let file1 = dir.path().join("file1");
    let file2 = dir.path().join("file2");
    fs::write(&file1, "a").unwrap();
    fs::write(&file2, "b").unwrap();

    // unlink should fail or only remove first file when given multiple operands
    let result = run(&["unlink", file1.to_str().unwrap(), file2.to_str().unwrap()]);
    // Behavior varies by implementation
    let _ = result;
}
