//! POSIX.1-2017 compliance tests for cut
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cut.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The cut utility shall cut out bytes, characters, or fields from each line"
#[test]
fn posix_cut_fields() {
    let result = run_with_stdin(&["cut", "-f1"], b"hello\tworld");
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: cut -d delimiter
#[test]
fn posix_cut_delimiter() {
    let result = run_with_stdin(&["cut", "-d:", "-f1"], b"root:x:0:0");
    assert_success(&result);
    assert_eq!(result.1, "root\n");
}

/// POSIX: cut -f field range
#[test]
fn posix_cut_field_range() {
    let result = run_with_stdin(&["cut", "-d:", "-f1-3"], b"a:b:c:d:e");
    assert_success(&result);
    assert_eq!(result.1, "a:b:c\n");
}

/// POSIX: cut -f comma-separated fields
#[test]
fn posix_cut_field_list() {
    let result = run_with_stdin(&["cut", "-d:", "-f1,3"], b"a:b:c:d:e");
    assert_success(&result);
    assert_eq!(result.1, "a:c\n");
}

/// POSIX: cut -c character positions
#[test]
fn posix_cut_chars() {
    let result = run_with_stdin(&["cut", "-c1-5"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: cut -b byte positions
#[test]
fn posix_cut_bytes() {
    let result = run_with_stdin(&["cut", "-b1-5"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: cut multiple lines
#[test]
fn posix_cut_multiple_lines() {
    let result = run_with_stdin(&["cut", "-d:", "-f1"], b"a:b\nc:d\ne:f");
    assert_success(&result);
    assert_eq!(result.1, "a\nc\ne\n");
}

/// POSIX: cut -s suppress lines without delimiter
#[test]
fn posix_cut_suppress() {
    let result = run_with_stdin(&["cut", "-d:", "-f1", "-s"], b"a:b\nno delimiter\nc:d");
    assert_success(&result);
    assert!(!result.1.contains("no delimiter"));
}

/// POSIX: cut from file
#[test]
fn posix_cut_from_file() {
    let dir = setup_test_env();
    let file = dir.path().join("input");
    fs::write(&file, "hello:world").unwrap();

    let result = run(&["cut", "-d:", "-f2", file.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(result.1, "world\n");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_cut_exit_success() {
    let result = run_with_stdin(&["cut", "-f1"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_cut_exit_error() {
    let result = run(&["cut", "-f1", "/nonexistent/file"]);
    assert!(result.0 > 0);
}

/// POSIX: cut open-ended range (-N)
#[test]
fn posix_cut_open_start() {
    let result = run_with_stdin(&["cut", "-c-5"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: cut open-ended range (N-)
#[test]
fn posix_cut_open_end() {
    let result = run_with_stdin(&["cut", "-c7-"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "world\n");
}
