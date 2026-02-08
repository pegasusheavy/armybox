//! POSIX.1-2017 compliance tests for mkdir
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mkdir.html

use crate::posix::helpers::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

/// POSIX: "The mkdir utility shall create the directories..."
#[test]
fn posix_mkdir_basic() {
    let dir = setup_test_env();
    let newdir = dir.path().join("newdir");

    let result = run(&["mkdir", newdir.to_str().unwrap()]);
    assert_success(&result);
    assert!(newdir.is_dir());
}

/// POSIX: mkdir -p creates intermediate directories
#[test]
fn posix_mkdir_parents() {
    let dir = setup_test_env();
    let nested = dir.path().join("a/b/c/d");

    let result = run(&["mkdir", "-p", nested.to_str().unwrap()]);
    assert_success(&result);
    assert!(nested.is_dir());
}

/// POSIX: mkdir -p does not fail if directory exists
#[test]
fn posix_mkdir_parents_exists() {
    let dir = setup_test_env();
    let existing = dir.path().join("existing");
    fs::create_dir(&existing).unwrap();

    let result = run(&["mkdir", "-p", existing.to_str().unwrap()]);
    assert_success(&result);
}

/// POSIX: mkdir -m sets mode
#[test]
fn posix_mkdir_mode() {
    let dir = setup_test_env();
    let newdir = dir.path().join("modedir");

    let result = run(&["mkdir", "-m", "755", newdir.to_str().unwrap()]);
    assert_success(&result);

    let perms = fs::metadata(&newdir).unwrap().permissions();
    assert_eq!(perms.mode() & 0o777, 0o755);
}

/// POSIX: mkdir multiple directories
#[test]
fn posix_mkdir_multiple() {
    let dir = setup_test_env();
    let dir1 = dir.path().join("dir1");
    let dir2 = dir.path().join("dir2");

    let result = run(&["mkdir", dir1.to_str().unwrap(), dir2.to_str().unwrap()]);
    assert_success(&result);
    assert!(dir1.is_dir());
    assert!(dir2.is_dir());
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_mkdir_exit_success() {
    let dir = setup_test_env();
    let newdir = dir.path().join("newdir");

    let result = run(&["mkdir", newdir.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 when directory exists (without -p)
#[test]
fn posix_mkdir_exit_exists() {
    let dir = setup_test_env();
    let existing = dir.path().join("existing");
    fs::create_dir(&existing).unwrap();

    let result = run(&["mkdir", existing.to_str().unwrap()]);
    assert!(result.0 > 0);
}

/// POSIX: Exit status >0 when parent does not exist (without -p)
#[test]
fn posix_mkdir_exit_no_parent() {
    let dir = setup_test_env();
    let nested = dir.path().join("nonexistent/child");

    let result = run(&["mkdir", nested.to_str().unwrap()]);
    assert!(result.0 > 0);
}
