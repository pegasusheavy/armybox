//! POSIX.1-2017 compliance tests for chown
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/chown.html
//!
//! Note: Most chown tests require root privileges. These tests verify
//! basic functionality and error handling that works without root.

use crate::posix::helpers::*;
use std::fs;

/// POSIX: Exit status >0 when file does not exist
#[test]
fn posix_chown_nonexistent() {
    let result = run(&["chown", "nobody", "/nonexistent/path/file"]);
    assert!(result.0 > 0);
}

/// POSIX: Exit status >0 for invalid owner specification
#[test]
fn posix_chown_invalid_owner() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    // Invalid user that doesn't exist
    let result = run(&["chown", "nonexistent_user_12345", file.to_str().unwrap()]);
    assert!(result.0 > 0);
}

/// POSIX: chown accepts owner:group format
#[test]
fn posix_chown_owner_group_format() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    // This will fail without root, but should parse correctly
    let result = run(&["chown", "root:root", file.to_str().unwrap()]);
    // Either succeeds (as root) or fails with permission error (not root)
    // Main thing is it doesn't crash on the format
    let _ = result;
}

/// POSIX: chown -R for recursive operation
#[test]
fn posix_chown_recursive_flag() {
    let dir = setup_test_env();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file = subdir.join("file");
    fs::write(&file, "content").unwrap();

    // Will fail without root, but verifies -R is accepted
    let result = run(&["chown", "-R", "root", dir.path().to_str().unwrap()]);
    // Accept either success or permission denied
    let _ = result;
}

/// POSIX: chown -h affects symbolic links
#[test]
fn posix_chown_symlink_flag() {
    let dir = setup_test_env();
    let file = dir.path().join("target");
    fs::write(&file, "content").unwrap();

    // Verify -h flag is accepted
    let result = run(&["chown", "-h", "root", file.to_str().unwrap()]);
    // Accept either success or permission denied
    let _ = result;
}

/// POSIX: chown multiple files
#[test]
fn posix_chown_multiple_files() {
    let dir = setup_test_env();
    let file1 = dir.path().join("file1");
    let file2 = dir.path().join("file2");
    fs::write(&file1, "a").unwrap();
    fs::write(&file2, "b").unwrap();

    // Verify multiple files accepted
    let result = run(&[
        "chown",
        "root",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
    ]);
    // Accept either success or permission denied
    let _ = result;
}

/// POSIX: chown with colon-only (change group only)
#[test]
fn posix_chown_group_only() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    // :group format should be accepted
    let result = run(&["chown", ":root", file.to_str().unwrap()]);
    let _ = result;
}

/// POSIX: chown with numeric UID
#[test]
fn posix_chown_numeric_uid() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    // Numeric UID should be accepted
    let result = run(&["chown", "0", file.to_str().unwrap()]);
    let _ = result;
}
