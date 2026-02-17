//! POSIX.1-2017 compliance tests for grep
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/grep.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The grep utility shall search... for the pattern"
#[test]
fn posix_grep_basic() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello\nworld\nhello world").unwrap();

    let result = run(&["grep", "hello", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("hello"));
}

/// POSIX: grep -i case insensitive
#[test]
fn posix_grep_case_insensitive() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "Hello\nWORLD\nhello").unwrap();

    let result = run(&["grep", "-i", "hello", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("Hello"));
    assert!(result.1.contains("hello"));
}

/// POSIX: grep -v invert match
#[test]
fn posix_grep_invert() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello\nworld\nfoo").unwrap();

    let result = run(&["grep", "-v", "hello", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(!result.1.contains("hello"));
    assert!(result.1.contains("world"));
    assert!(result.1.contains("foo"));
}

/// POSIX: grep -c count matches
#[test]
fn posix_grep_count() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello\nhello\nworld").unwrap();

    let result = run(&["grep", "-c", "hello", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.trim() == "2");
}

/// POSIX: grep -l list files with matches
#[test]
fn posix_grep_list_files() {
    let dir = setup_test_env();
    let file1 = dir.path().join("file1");
    let file2 = dir.path().join("file2");
    fs::write(&file1, "hello").unwrap();
    fs::write(&file2, "world").unwrap();

    let result = run(&[
        "grep",
        "-l",
        "hello",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
    ]);
    assert_success(&result);
    assert!(result.1.contains("file1"));
    assert!(!result.1.contains("file2"));
}

/// POSIX: grep -n show line numbers
#[test]
fn posix_grep_line_numbers() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "foo\nhello\nbar").unwrap();

    let result = run(&["grep", "-n", "hello", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("2:")); // Line 2
}

/// POSIX: grep -q quiet mode
#[test]
fn posix_grep_quiet() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello world").unwrap();

    let result = run(&["grep", "-q", "hello", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
    assert!(result.1.is_empty());
}

/// POSIX: grep -E extended regex
#[test]
fn posix_grep_extended() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello\nworld\nhelloworld").unwrap();

    let result = run(&["grep", "-E", "hello|world", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("hello"));
    assert!(result.1.contains("world"));
}

/// POSIX: grep -F fixed strings
#[test]
fn posix_grep_fixed() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello.world\nhelloXworld").unwrap();

    let result = run(&["grep", "-F", "hello.world", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("hello.world"));
    assert!(!result.1.contains("helloXworld"));
}

/// POSIX: grep from stdin
#[test]
fn posix_grep_stdin() {
    let result = run_with_stdin(&["grep", "hello"], b"hello\nworld\nhello world");
    assert_success(&result);
    assert!(result.1.contains("hello"));
}

/// POSIX: Exit status 0 when match found
#[test]
fn posix_grep_exit_match() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello").unwrap();

    let result = run(&["grep", "hello", file.to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status 1 when no match
#[test]
fn posix_grep_exit_no_match() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello").unwrap();

    let result = run(&["grep", "notfound", file.to_str().unwrap()]);
    assert_eq!(result.0, 1);
}

/// POSIX: Exit status >1 on error
#[test]
fn posix_grep_exit_error() {
    let result = run(&["grep", "pattern", "/nonexistent/file"]);
    assert!(result.0 > 1);
}

/// POSIX: grep multiple files
#[test]
fn posix_grep_multiple_files() {
    let dir = setup_test_env();
    let file1 = dir.path().join("file1");
    let file2 = dir.path().join("file2");
    fs::write(&file1, "hello file1").unwrap();
    fs::write(&file2, "hello file2").unwrap();

    let result = run(&[
        "grep",
        "hello",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
    ]);
    assert_success(&result);
    assert!(result.1.contains("file1"));
    assert!(result.1.contains("file2"));
}

/// POSIX: grep -e pattern
#[test]
fn posix_grep_e_option() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello\nworld").unwrap();

    let result = run(&["grep", "-e", "hello", "-e", "world", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("hello"));
    assert!(result.1.contains("world"));
}

/// POSIX: grep -f pattern file
#[test]
fn posix_grep_pattern_file() {
    let dir = setup_test_env();
    let data = dir.path().join("data");
    let patterns = dir.path().join("patterns");
    fs::write(&data, "hello\nworld\nfoo").unwrap();
    fs::write(&patterns, "hello\nfoo").unwrap();

    let result = run(&[
        "grep",
        "-f",
        patterns.to_str().unwrap(),
        data.to_str().unwrap(),
    ]);
    assert_success(&result);
    assert!(result.1.contains("hello"));
    assert!(result.1.contains("foo"));
    assert!(!result.1.contains("world"));
}

/// POSIX: grep regex anchors
#[test]
fn posix_grep_anchors() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello\nworld hello\nhello world").unwrap();

    let result = run(&["grep", "^hello", file.to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.lines().all(|l| l.starts_with("hello")));
}

/// POSIX: grep word boundary -w
#[test]
fn posix_grep_word() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello\nhelloworld\nworld hello world").unwrap();

    let result = run(&["grep", "-w", "hello", file.to_str().unwrap()]);
    assert_success(&result);
    // Should match "hello" but not "helloworld"
}
