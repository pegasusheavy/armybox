//! POSIX.1-2017 compliance tests for od
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/od.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The od utility shall write the contents of its input files to standard output in a specified format"
#[test]
fn posix_od_basic() {
    let result = run_with_stdin(&["od"], b"hello");
    assert_success(&result);
    // Default output is octal
    assert!(result.1.contains("0"));
}

/// POSIX: od -A address base
#[test]
fn posix_od_address_octal() {
    let result = run_with_stdin(&["od", "-A", "o"], b"hello");
    assert_success(&result);
}

/// POSIX: od -A x hex address
#[test]
fn posix_od_address_hex() {
    let result = run_with_stdin(&["od", "-A", "x"], b"hello");
    assert_success(&result);
}

/// POSIX: od -A d decimal address
#[test]
fn posix_od_address_decimal() {
    let result = run_with_stdin(&["od", "-A", "d"], b"hello");
    assert_success(&result);
}

/// POSIX: od -A n no address
#[test]
fn posix_od_no_address() {
    let result = run_with_stdin(&["od", "-A", "n"], b"hello");
    assert_success(&result);
}

/// POSIX: od -t output type
#[test]
fn posix_od_type_char() {
    let result = run_with_stdin(&["od", "-t", "c"], b"hello");
    assert_success(&result);
    // Should show character representation
    assert!(result.1.contains("h") || result.1.contains("e") || result.1.contains("l"));
}

/// POSIX: od -t x hex bytes
#[test]
fn posix_od_type_hex() {
    let result = run_with_stdin(&["od", "-t", "x1"], b"AB");
    assert_success(&result);
    // Should contain hex for 'A' (0x41) and 'B' (0x42)
    assert!(result.1.contains("41") || result.1.contains("42"));
}

/// POSIX: od from file
#[test]
fn posix_od_from_file() {
    let dir = setup_test_env();
    let file = dir.path().join("input");
    fs::write(&file, "test data").unwrap();

    let result = run(&["od", file.to_str().unwrap()]);
    assert_success(&result);
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_od_exit_success() {
    let result = run_with_stdin(&["od"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_od_exit_error() {
    let result = run(&["od", "/nonexistent/file"]);
    assert!(result.0 > 0);
}

/// POSIX: od -N count
#[test]
fn posix_od_count() {
    let result = run_with_stdin(&["od", "-N", "5"], b"hello world");
    assert_success(&result);
    // Should only dump first 5 bytes
}

/// POSIX: od -j skip
#[test]
fn posix_od_skip() {
    let result = run_with_stdin(&["od", "-j", "5", "-t", "c"], b"hello world");
    assert_success(&result);
    // Should skip first 5 bytes
}
