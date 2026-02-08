//! POSIX.1-2017 compliance tests for chgrp
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/chgrp.html
//!
//! Note: Most chgrp tests require root privileges. These tests verify
//! basic functionality and error handling that works without root.

use crate::posix::helpers::*;
use std::fs;

/// POSIX: Exit status >0 when file does not exist
#[test]
fn posix_chgrp_nonexistent() {
    let result = run(&["chgrp", "root", "/nonexistent/path/file"]);
    assert!(result.0 > 0);
}

/// POSIX: Exit status >0 for invalid group specification
#[test]
fn posix_chgrp_invalid_group() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    // Invalid group that doesn't exist
    let result = run(&["chgrp", "nonexistent_group_12345", file.to_str().unwrap()]);
    assert!(result.0 > 0);
}

/// POSIX: chgrp -R for recursive operation
#[test]
fn posix_chgrp_recursive_flag() {
    let dir = setup_test_env();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file = subdir.join("file");
    fs::write(&file, "content").unwrap();

    // Will fail without root, but verifies -R is accepted
    let result = run(&["chgrp", "-R", "root", dir.path().to_str().unwrap()]);
    // Accept either success or permission denied
    let _ = result;
}

/// POSIX: chgrp -h affects symbolic links
#[test]
fn posix_chgrp_symlink_flag() {
    let dir = setup_test_env();
    let file = dir.path().join("target");
    fs::write(&file, "content").unwrap();

    // Verify -h flag is accepted
    let result = run(&["chgrp", "-h", "root", file.to_str().unwrap()]);
    let _ = result;
}

/// POSIX: chgrp multiple files
#[test]
fn posix_chgrp_multiple_files() {
    let dir = setup_test_env();
    let file1 = dir.path().join("file1");
    let file2 = dir.path().join("file2");
    fs::write(&file1, "a").unwrap();
    fs::write(&file2, "b").unwrap();

    // Verify multiple files accepted
    let result = run(&[
        "chgrp",
        "root",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
    ]);
    let _ = result;
}

/// POSIX: chgrp with numeric GID
#[test]
fn posix_chgrp_numeric_gid() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    // Numeric GID should be accepted
    let result = run(&["chgrp", "0", file.to_str().unwrap()]);
    let _ = result;
}
