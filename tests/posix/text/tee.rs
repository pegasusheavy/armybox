//! POSIX.1-2017 compliance tests for tee
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/tee.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The tee utility shall copy standard input to standard output and to zero or more files"
#[test]
fn posix_tee_basic() {
    let dir = setup_test_env();
    let file = dir.path().join("output");

    let result = run_with_stdin(&["tee", file.to_str().unwrap()], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello world");
    assert_eq!(fs::read_to_string(&file).unwrap(), "hello world");
}

/// POSIX: tee -a append
#[test]
fn posix_tee_append() {
    let dir = setup_test_env();
    let file = dir.path().join("output");
    fs::write(&file, "existing\n").unwrap();

    let result = run_with_stdin(&["tee", "-a", file.to_str().unwrap()], b"new");
    assert_success(&result);
    assert_eq!(fs::read_to_string(&file).unwrap(), "existing\nnew");
}

/// POSIX: tee multiple files
#[test]
fn posix_tee_multiple() {
    let dir = setup_test_env();
    let file1 = dir.path().join("output1");
    let file2 = dir.path().join("output2");

    let result = run_with_stdin(
        &["tee", file1.to_str().unwrap(), file2.to_str().unwrap()],
        b"content",
    );
    assert_success(&result);
    assert_eq!(fs::read_to_string(&file1).unwrap(), "content");
    assert_eq!(fs::read_to_string(&file2).unwrap(), "content");
}

/// POSIX: tee stdout still works
#[test]
fn posix_tee_stdout() {
    let result = run_with_stdin(&["tee"], b"just stdout");
    assert_success(&result);
    assert_eq!(result.1, "just stdout");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_tee_exit_success() {
    let result = run_with_stdin(&["tee"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: tee -i ignore interrupts
#[test]
fn posix_tee_ignore_interrupt() {
    let dir = setup_test_env();
    let file = dir.path().join("output");

    // -i flag should be accepted
    let result = run_with_stdin(&["tee", "-i", file.to_str().unwrap()], b"content");
    assert_success(&result);
}
