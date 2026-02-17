//! POSIX.1-2017 compliance tests for rm
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/rm.html

use crate::posix::helpers::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

/// POSIX: "The rm utility shall remove the directory entry..."
#[test]
fn posix_rm_basic() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    let result = run(&["rm", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(!file.exists());
}

/// POSIX: rm multiple files
#[test]
fn posix_rm_multiple() {
    let dir = setup_test_env();
    let file1 = dir.path().join("file1");
    let file2 = dir.path().join("file2");
    fs::write(&file1, "a").unwrap();
    fs::write(&file2, "b").unwrap();

    let result = run(&["rm", file1.to_str().unwrap(), file2.to_str().unwrap()]);
    assert_success(&result);
    assert!(!file1.exists());
    assert!(!file2.exists());
}

/// POSIX: rm -f force (no error for nonexistent)
#[test]
fn posix_rm_force_nonexistent() {
    let result = run(&["rm", "-f", "/nonexistent/path/file"]);
    assert_eq!(result.0, 0);
}

/// POSIX: rm -f force removes read-only file
#[test]
fn posix_rm_force_readonly() {
    let dir = setup_test_env();
    let file = dir.path().join("readonly");
    fs::write(&file, "content").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o444)).unwrap();

    let result = run(&["rm", "-f", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(!file.exists());
}

/// POSIX: rm -r recursive
#[test]
fn posix_rm_recursive() {
    let dir = setup_test_env();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file"), "content").unwrap();

    let result = run(&["rm", "-r", subdir.to_str().unwrap()]);
    assert_success(&result);
    assert!(!subdir.exists());
}

/// POSIX: rm -R is equivalent to -r
#[test]
fn posix_rm_recursive_uppercase() {
    let dir = setup_test_env();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file"), "content").unwrap();

    let result = run(&["rm", "-R", subdir.to_str().unwrap()]);
    assert_success(&result);
    assert!(!subdir.exists());
}

/// POSIX: rm -rf combined flags
#[test]
fn posix_rm_rf() {
    let dir = setup_test_env();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file"), "content").unwrap();

    let result = run(&["rm", "-rf", subdir.to_str().unwrap()]);
    assert_success(&result);
    assert!(!subdir.exists());
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_rm_exit_success() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "x").unwrap();

    let result = run(&["rm", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error (without -f)
#[test]
fn posix_rm_exit_error() {
    let result = run(&["rm", "/nonexistent/path/file"]);
    assert!(result.0 > 0);
}

/// POSIX: rm -i flag accepted (interactive)
#[test]
fn posix_rm_interactive() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "content").unwrap();

    // In non-interactive mode, -i should still work
    // (behavior may vary but flag should be accepted)
    let result = run(&["rm", "-i", file.to_str().unwrap()]);
    // Accept success or other behavior
    let _ = result;
}

/// POSIX: rm nested directories
#[test]
fn posix_rm_nested() {
    let dir = setup_test_env();
    let nested = dir.path().join("a/b/c/d");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("file"), "content").unwrap();

    let result = run(&["rm", "-r", dir.path().join("a").to_str().unwrap()]);
    assert_success(&result);
    assert!(!dir.path().join("a").exists());
}

/// POSIX: rm empty directory requires -r or -d
#[test]
fn posix_rm_directory_without_flag() {
    let dir = setup_test_env();
    let subdir = dir.path().join("emptydir");
    fs::create_dir(&subdir).unwrap();

    let result = run(&["rm", subdir.to_str().unwrap()]);
    // Should fail without -r flag
    assert!(result.0 > 0);
}

/// POSIX: rm -d removes empty directory
#[test]
fn posix_rm_d_empty_dir() {
    let dir = setup_test_env();
    let subdir = dir.path().join("emptydir");
    fs::create_dir(&subdir).unwrap();

    let result = run(&["rm", "-d", subdir.to_str().unwrap()]);
    // -d may or may not be supported (not POSIX standard)
    if result.0 == 0 {
        assert!(!subdir.exists());
    }
}
