//! POSIX.1-2017 compliance tests for mv
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mv.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The mv utility shall move files..."
#[test]
fn posix_mv_basic() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let dst = dir.path().join("dest");
    fs::write(&src, "content").unwrap();

    let result = run(&["mv", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_success(&result);
    assert!(!src.exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "content");
}

/// POSIX: mv file to directory
#[test]
fn posix_mv_to_directory() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let target_dir = dir.path().join("target");
    fs::write(&src, "content").unwrap();
    fs::create_dir(&target_dir).unwrap();

    let result = run(&["mv", src.to_str().unwrap(), target_dir.to_str().unwrap()]);
    assert_success(&result);
    assert!(!src.exists());
    assert!(target_dir.join("source").exists());
}

/// POSIX: mv -f force overwrite
#[test]
fn posix_mv_force() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let dst = dir.path().join("dest");
    fs::write(&src, "new").unwrap();
    fs::write(&dst, "old").unwrap();

    let result = run(&["mv", "-f", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(fs::read_to_string(&dst).unwrap(), "new");
}

/// POSIX: mv -i flag accepted (interactive)
#[test]
fn posix_mv_interactive() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let dst = dir.path().join("dest");
    fs::write(&src, "content").unwrap();

    let result = run(&["mv", "-i", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_success(&result);
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_mv_exit_success() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let dst = dir.path().join("dest");
    fs::write(&src, "x").unwrap();

    let result = run(&["mv", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_mv_exit_error() {
    let result = run(&["mv", "/nonexistent/source", "/tmp/dest"]);
    assert!(result.0 > 0);
}

/// POSIX: mv directory
#[test]
fn posix_mv_directory() {
    let dir = setup_test_env();
    let src_dir = dir.path().join("srcdir");
    let dst_dir = dir.path().join("dstdir");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("file"), "content").unwrap();

    let result = run(&["mv", src_dir.to_str().unwrap(), dst_dir.to_str().unwrap()]);
    assert_success(&result);
    assert!(!src_dir.exists());
    assert!(dst_dir.join("file").exists());
}

/// POSIX: mv multiple sources to directory
#[test]
fn posix_mv_multiple() {
    let dir = setup_test_env();
    let src1 = dir.path().join("file1");
    let src2 = dir.path().join("file2");
    let target = dir.path().join("target");
    fs::write(&src1, "content1").unwrap();
    fs::write(&src2, "content2").unwrap();
    fs::create_dir(&target).unwrap();

    let result = run(&[
        "mv",
        src1.to_str().unwrap(),
        src2.to_str().unwrap(),
        target.to_str().unwrap(),
    ]);
    assert_success(&result);
    assert!(!src1.exists());
    assert!(!src2.exists());
    assert!(target.join("file1").exists());
    assert!(target.join("file2").exists());
}

/// POSIX: mv overwrites destination by default
#[test]
fn posix_mv_overwrite() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let dst = dir.path().join("dest");
    fs::write(&src, "new").unwrap();
    fs::write(&dst, "old").unwrap();

    let result = run(&["mv", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(fs::read_to_string(&dst).unwrap(), "new");
}

/// POSIX: mv preserves binary content
#[test]
fn posix_mv_binary() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let dst = dir.path().join("dest");
    let binary: Vec<u8> = (0..=255).collect();
    fs::write(&src, &binary).unwrap();

    let result = run(&["mv", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(fs::read(&dst).unwrap(), binary);
}

/// POSIX: mv rename in same directory
#[test]
fn posix_mv_rename() {
    let dir = setup_test_env();
    let src = dir.path().join("oldname");
    let dst = dir.path().join("newname");
    fs::write(&src, "content").unwrap();

    let result = run(&["mv", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_success(&result);
    assert!(!src.exists());
    assert!(dst.exists());
}

/// POSIX: mv empty file
#[test]
fn posix_mv_empty() {
    let dir = setup_test_env();
    let src = dir.path().join("source");
    let dst = dir.path().join("dest");
    fs::write(&src, "").unwrap();

    let result = run(&["mv", src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(fs::read_to_string(&dst).unwrap(), "");
}
