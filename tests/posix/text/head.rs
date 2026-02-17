//! POSIX.1-2017 compliance tests for head
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/head.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The head utility shall copy the first count lines of each file"
#[test]
fn posix_head_default() {
    let input = (1..=15).map(|n| format!("line{}", n)).collect::<Vec<_>>().join("\n");
    let result = run_with_stdin(&["head"], input.as_bytes());
    assert_success(&result);
    assert_eq!(count_stdout_lines(&result), 10);
}

/// POSIX: head -n count
#[test]
fn posix_head_count() {
    let input = (1..=10).map(|n| format!("line{}", n)).collect::<Vec<_>>().join("\n");
    let result = run_with_stdin(&["head", "-n", "5"], input.as_bytes());
    assert_success(&result);
    assert_eq!(count_stdout_lines(&result), 5);
}

/// POSIX: head -n with negative count (all but last N)
#[test]
fn posix_head_negative() {
    let input = (1..=10).map(|n| format!("line{}", n)).collect::<Vec<_>>().join("\n");
    let result = run_with_stdin(&["head", "-n", "-3"], input.as_bytes());
    assert_success(&result);
    assert_eq!(count_stdout_lines(&result), 7);
}

/// POSIX: head -c bytes
#[test]
fn posix_head_bytes() {
    let result = run_with_stdin(&["head", "-c", "5"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello");
}

/// POSIX: head from file
#[test]
fn posix_head_from_file() {
    let dir = setup_test_env();
    let file = dir.path().join("input");
    let content = (1..=15).map(|n| format!("line{}", n)).collect::<Vec<_>>().join("\n");
    fs::write(&file, &content).unwrap();

    let result = run(&["head", file.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(count_stdout_lines(&result), 10);
}

/// POSIX: head multiple files
#[test]
fn posix_head_multiple_files() {
    let dir = setup_test_env();
    let file1 = dir.path().join("file1");
    let file2 = dir.path().join("file2");
    fs::write(&file1, "content1\n").unwrap();
    fs::write(&file2, "content2\n").unwrap();

    let result = run(&[
        "head",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
    ]);
    assert_success(&result);
    assert!(result.1.contains("content1"));
    assert!(result.1.contains("content2"));
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_head_exit_success() {
    let result = run_with_stdin(&["head"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_head_exit_error() {
    let result = run(&["head", "/nonexistent/file"]);
    assert!(result.0 > 0);
}
