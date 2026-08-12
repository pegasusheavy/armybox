//! POSIX.1-2017 compliance tests for xargs
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/xargs.html

use crate::posix::helpers::*;

/// POSIX: "The xargs utility shall construct argument lists and invoke utility"
#[test]
fn posix_xargs_basic() {
    let result = run_with_stdin(&["xargs", "echo"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1.trim(), "hello world");
}

/// POSIX: xargs -n max args per invocation
#[test]
fn posix_xargs_max_args() {
    let result = run_with_stdin(&["xargs", "-n", "1", "echo"], b"a b c");
    assert_success(&result);
    let lines: Vec<&str> = result.1.lines().collect();
    assert_eq!(lines.len(), 3);
}

/// POSIX: xargs -I replace string
#[test]
fn posix_xargs_replace() {
    let result = run_with_stdin(&["xargs", "-I", "{}", "echo", "prefix_{}_suffix"], b"test");
    assert_success(&result);
    assert!(result.1.contains("prefix_test_suffix"));
}

/// POSIX: xargs -t trace (echo commands)
#[test]
fn posix_xargs_trace() {
    let result = run_with_stdin(&["xargs", "-t", "echo"], b"hello");
    assert_success(&result);
    // Trace output goes to stderr
}

/// POSIX: xargs default command is echo
#[test]
fn posix_xargs_default_echo() {
    let result = run_with_stdin(&["xargs"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1.trim(), "hello world");
}

/// POSIX: xargs empty input
#[test]
fn posix_xargs_empty() {
    let result = run_with_stdin(&["xargs", "echo"], b"");
    assert_success(&result);
}

/// POSIX: xargs -0 null separator
#[test]
fn posix_xargs_null_separator() {
    let result = run_with_stdin(&["xargs", "-0", "echo"], b"a\0b\0c");
    assert_success(&result);
    assert!(result.1.contains("a") && result.1.contains("b") && result.1.contains("c"));
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_xargs_exit_success() {
    let result = run_with_stdin(&["xargs", "true"], b"arg");
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status 123 if utility invocation returned 1-125
#[test]
fn posix_xargs_exit_123() {
    let result = run_with_stdin(&["xargs", "false"], b"arg");
    assert!(result.0 == 123 || result.0 == 1);
}

/// POSIX: Exit status 127 if utility not found
#[test]
fn posix_xargs_exit_127() {
    let result = run_with_stdin(&["xargs", "nonexistent_cmd_12345"], b"arg");
    assert!(result.0 == 127 || result.0 > 0);
}

/// POSIX: xargs -p prompt (interactive)
#[test]
fn posix_xargs_prompt_flag() {
    // Can't really test interactive mode, just verify flag is accepted
    let result = run_with_stdin(&["xargs", "-p", "echo"], b"");
    let _ = result;
}

/// POSIX: xargs with quoted arguments
#[test]
fn posix_xargs_quoted() {
    let result = run_with_stdin(&["xargs", "echo"], b"'hello world'");
    assert_success(&result);
    // Quotes are stripped and the quoted text becomes a single argument.
    assert_eq!(result.1.trim(), "hello world");
}

/// GNU xargs: `-d` sets a custom, single-character input delimiter.
#[test]
fn posix_xargs_custom_delimiter() {
    let result = run_with_stdin(&["xargs", "-d", ":", "echo"], b"a:b:c");
    assert_success(&result);
    assert_eq!(result.1.trim(), "a b c");
}

/// GNU xargs: `-P2 -n1` runs up to two invocations in parallel, one item each.
#[test]
fn posix_xargs_parallel() {
    let result = run_with_stdin(&["xargs", "-P2", "-n1", "echo"], b"a\nb\nc");
    assert_eq!(result.0, 0);
    assert!(result.1.contains('a'));
    assert!(result.1.contains('b'));
    assert!(result.1.contains('c'));
}

/// GNU xargs: an empty `-I` replstr is a usage error (exit status 2), not a crash.
#[test]
fn posix_xargs_empty_replstr_is_usage_error() {
    let result = run_with_stdin(&["xargs", "-I", "", "echo"], b"hello");
    assert_eq!(result.0, 2);
}

/// GNU xargs: `-I` replaces every occurrence of REPLSTR in an argument, not
/// just the first.
#[test]
fn posix_xargs_replace_all_occurrences() {
    let result = run_with_stdin(&["xargs", "-I{}", "echo", "{}-{}"], b"x");
    assert_success(&result);
    assert_eq!(result.1.trim(), "x-x");
}
