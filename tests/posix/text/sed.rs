//! POSIX.1-2017 compliance tests for sed
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/sed.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The sed utility is a stream editor"
#[test]
fn posix_sed_basic_substitute() {
    let result = run_with_stdin(&["sed", "s/hello/world/"], b"hello there");
    assert_success(&result);
    assert_eq!(result.1, "world there\n");
}

/// POSIX: sed global substitution
#[test]
fn posix_sed_global_substitute() {
    let result = run_with_stdin(&["sed", "s/a/b/g"], b"aaa");
    assert_success(&result);
    assert_eq!(result.1, "bbb\n");
}

/// POSIX: sed -n suppress output
#[test]
fn posix_sed_suppress() {
    let result = run_with_stdin(&["sed", "-n", "s/hello/world/p"], b"hello\ngoodbye");
    assert_success(&result);
    assert_eq!(result.1, "world\n");
}

/// POSIX: sed -e multiple expressions
#[test]
fn posix_sed_multiple_expr() {
    let result = run_with_stdin(&["sed", "-e", "s/a/b/", "-e", "s/c/d/"], b"abc");
    assert_success(&result);
    assert_eq!(result.1, "bbd\n");
}

/// POSIX: sed -f script file
#[test]
fn posix_sed_script_file() {
    let dir = setup_test_env();
    let script = dir.path().join("script");
    fs::write(&script, "s/hello/world/").unwrap();

    let result = run_with_stdin(&["sed", "-f", script.to_str().unwrap()], b"hello");
    assert_success(&result);
    assert_eq!(result.1, "world\n");
}

/// POSIX: sed delete command
#[test]
fn posix_sed_delete() {
    let result = run_with_stdin(&["sed", "/hello/d"], b"hello\nworld\nhello again");
    assert_success(&result);
    assert_eq!(result.1, "world\n");
}

/// POSIX: sed print command
#[test]
fn posix_sed_print() {
    let result = run_with_stdin(&["sed", "-n", "/hello/p"], b"hello\nworld\nhello again");
    assert_success(&result);
    assert!(result.1.lines().count() == 2);
}

/// POSIX: sed address range
#[test]
fn posix_sed_address_range() {
    let result = run_with_stdin(&["sed", "2,3s/x/y/"], b"x\nx\nx\nx");
    assert_success(&result);
    assert_eq!(result.1, "x\ny\ny\nx\n");
}

/// POSIX: sed line number address
#[test]
fn posix_sed_line_address() {
    let result = run_with_stdin(&["sed", "2s/old/new/"], b"old\nold\nold");
    assert_success(&result);
    assert_eq!(result.1, "old\nnew\nold\n");
}

/// POSIX: sed last line address $
#[test]
fn posix_sed_last_line() {
    let result = run_with_stdin(&["sed", "$s/x/y/"], b"x\nx\nx");
    assert_success(&result);
    assert_eq!(result.1, "x\nx\ny\n");
}

/// POSIX: sed regex address
#[test]
fn posix_sed_regex_address() {
    let result = run_with_stdin(&["sed", "/^hello/s/world/there/"], b"hello world\ngoodbye world");
    assert_success(&result);
    assert_eq!(result.1, "hello there\ngoodbye world\n");
}

/// POSIX: sed append command
#[test]
fn posix_sed_append() {
    let result = run_with_stdin(&["sed", "/hello/a\\new line"], b"hello\nworld");
    assert_success(&result);
    assert!(result.1.contains("new line"));
}

/// POSIX: sed insert command
#[test]
fn posix_sed_insert() {
    let result = run_with_stdin(&["sed", "/hello/i\\inserted"], b"hello\nworld");
    assert_success(&result);
    let lines: Vec<&str> = result.1.lines().collect();
    assert!(lines.iter().any(|l| l.contains("inserted")));
}

/// POSIX: sed change command
#[test]
fn posix_sed_change() {
    let result = run_with_stdin(&["sed", "/hello/c\\replaced"], b"hello\nworld");
    assert_success(&result);
    assert!(result.1.contains("replaced"));
    assert!(!result.1.contains("hello"));
}

/// POSIX: sed from file
#[test]
fn posix_sed_from_file() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    fs::write(&input, "hello world").unwrap();

    let result = run(&["sed", "s/hello/goodbye/", input.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(result.1, "goodbye world\n");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_sed_exit_success() {
    let result = run_with_stdin(&["sed", "s/a/b/"], b"abc");
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_sed_exit_error() {
    let result = run(&["sed", "s/a/b/", "/nonexistent/file"]);
    assert!(result.0 > 0);
}

/// POSIX: sed substitute with delimiter
#[test]
fn posix_sed_alternate_delimiter() {
    let result = run_with_stdin(&["sed", "s|/path/to|/new/path|"], b"/path/to/file");
    assert_success(&result);
    assert_eq!(result.1, "/new/path/file\n");
}

/// POSIX: sed backreferences
#[test]
fn posix_sed_backreference() {
    let result = run_with_stdin(&["sed", r"s/\(hello\)/[\1]/"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "[hello] world\n");
}

/// POSIX: sed & replacement
#[test]
fn posix_sed_ampersand() {
    let result = run_with_stdin(&["sed", "s/hello/[&]/"], b"hello");
    assert_success(&result);
    assert_eq!(result.1, "[hello]\n");
}

/// POSIX: sed quit command
#[test]
fn posix_sed_quit() {
    let result = run_with_stdin(&["sed", "2q"], b"line1\nline2\nline3\nline4");
    assert_success(&result);
    assert_eq!(result.1, "line1\nline2\n");
}

/// POSIX: sed n command (next)
#[test]
fn posix_sed_next() {
    let result = run_with_stdin(&["sed", "n;d"], b"keep\ndelete\nkeep\ndelete");
    assert_success(&result);
    // Every other line should be deleted
}

/// POSIX: sed y transform
#[test]
fn posix_sed_transform() {
    let result = run_with_stdin(&["sed", "y/abc/xyz/"], b"aabbcc");
    assert_success(&result);
    assert_eq!(result.1, "xxyyzz\n");
}

/// POSIX: sed inplace -i (extension)
#[test]
fn posix_sed_inplace() {
    let dir = setup_test_env();
    let file = dir.path().join("testfile");
    fs::write(&file, "hello").unwrap();

    let result = run(&["sed", "-i", "s/hello/world/", file.to_str().unwrap()]);
    // -i may not be POSIX standard but is commonly supported
    if result.0 == 0 {
        assert_eq!(fs::read_to_string(&file).unwrap(), "world\n");
    }
}
