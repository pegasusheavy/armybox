//! POSIX.1-2017 compliance tests for mkfifo
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mkfifo.html

use crate::posix::helpers::*;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};

/// POSIX: "The mkfifo utility shall create the FIFO special files"
#[test]
fn posix_mkfifo_basic() {
    let dir = setup_test_env();
    let fifo = dir.path().join("testfifo");

    let result = run(&["mkfifo", fifo.to_str().unwrap()]);
    assert_success(&result);
    assert!(fs::metadata(&fifo).unwrap().file_type().is_fifo());
}

/// POSIX: mkfifo -m sets mode
#[test]
fn posix_mkfifo_mode() {
    let dir = setup_test_env();
    let fifo = dir.path().join("testfifo");

    let result = run(&["mkfifo", "-m", "644", fifo.to_str().unwrap()]);
    assert_success(&result);

    let perms = fs::metadata(&fifo).unwrap().permissions();
    assert_eq!(perms.mode() & 0o777, 0o644);
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_mkfifo_exit_success() {
    let dir = setup_test_env();
    let fifo = dir.path().join("testfifo");

    let result = run(&["mkfifo", fifo.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error (existing file)
#[test]
fn posix_mkfifo_exit_exists() {
    let dir = setup_test_env();
    let fifo = dir.path().join("testfifo");
    fs::write(&fifo, "x").unwrap();

    let result = run(&["mkfifo", fifo.to_str().unwrap()]);
    assert!(result.0 > 0);
}
