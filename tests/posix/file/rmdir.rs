//! POSIX.1-2017 compliance tests for rmdir
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/rmdir.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The rmdir utility shall remove the directory entry..."
#[test]
fn posix_rmdir_basic() {
    let dir = setup_test_env();
    let target = dir.path().join("emptydir");
    fs::create_dir(&target).unwrap();

    let result = run(&["rmdir", target.to_str().unwrap()]);
    assert_success(&result);
    assert!(!target.exists());
}

/// POSIX: rmdir -p removes parent directories
#[test]
fn posix_rmdir_parents() {
    let dir = setup_test_env();
    let nested = dir.path().join("a/b/c");
    fs::create_dir_all(&nested).unwrap();

    let result = run(&["rmdir", "-p", nested.to_str().unwrap()]);
    assert_success(&result);
    assert!(!dir.path().join("a").exists());
}

/// POSIX: rmdir fails on non-empty directory
#[test]
fn posix_rmdir_nonempty() {
    let dir = setup_test_env();
    let target = dir.path().join("nonempty");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("file"), "content").unwrap();

    let result = run(&["rmdir", target.to_str().unwrap()]);
    assert!(result.0 > 0);
    assert!(target.exists());
}

/// POSIX: rmdir multiple directories
#[test]
fn posix_rmdir_multiple() {
    let dir = setup_test_env();
    let dir1 = dir.path().join("dir1");
    let dir2 = dir.path().join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    let result = run(&["rmdir", dir1.to_str().unwrap(), dir2.to_str().unwrap()]);
    assert_success(&result);
    assert!(!dir1.exists());
    assert!(!dir2.exists());
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_rmdir_exit_success() {
    let dir = setup_test_env();
    let target = dir.path().join("emptydir");
    fs::create_dir(&target).unwrap();

    let result = run(&["rmdir", target.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_rmdir_exit_error() {
    let result = run(&["rmdir", "/nonexistent/path"]);
    assert!(result.0 > 0);
}
