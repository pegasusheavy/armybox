//! POSIX.1-2017 compliance tests for awk
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The awk utility shall execute programs"
#[test]
fn posix_awk_print() {
    let result = run_with_stdin(&["awk", "{print}"], b"hello\nworld");
    assert_success(&result);
    assert_eq!(result.1, "hello\nworld\n");
}

/// POSIX: awk field access
#[test]
fn posix_awk_fields() {
    let result = run_with_stdin(&["awk", "{print $1}"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: awk -F field separator
#[test]
fn posix_awk_field_separator() {
    let result = run_with_stdin(&["awk", "-F:", "{print $1}"], b"root:x:0:0");
    assert_success(&result);
    assert_eq!(result.1, "root\n");
}

/// POSIX: awk -v variable assignment
#[test]
fn posix_awk_variable() {
    let result = run_with_stdin(&["awk", "-v", "x=42", "{print x}"], b"ignored");
    assert_success(&result);
    assert_eq!(result.1, "42\n");
}

/// POSIX: awk pattern matching
#[test]
fn posix_awk_pattern() {
    let result = run_with_stdin(&["awk", "/hello/{print}"], b"hello\nworld\nhello again");
    assert_success(&result);
    let lines: Vec<&str> = result.1.lines().collect();
    assert_eq!(lines.len(), 2);
}

/// POSIX: awk BEGIN block
#[test]
fn posix_awk_begin() {
    let result = run_with_stdin(&["awk", "BEGIN{print \"start\"}"], b"");
    assert_success(&result);
    assert_eq!(result.1, "start\n");
}

/// POSIX: awk END block
#[test]
fn posix_awk_end() {
    let result = run_with_stdin(&["awk", "END{print \"done\"}"], b"line1\nline2");
    assert_success(&result);
    assert_eq!(result.1, "done\n");
}

/// POSIX: awk NR (record number)
#[test]
fn posix_awk_nr() {
    let result = run_with_stdin(&["awk", "{print NR, $0}"], b"a\nb\nc");
    assert_success(&result);
    assert!(result.1.contains("1 a"));
    assert!(result.1.contains("2 b"));
    assert!(result.1.contains("3 c"));
}

/// POSIX: awk NF (number of fields)
#[test]
fn posix_awk_nf() {
    let result = run_with_stdin(&["awk", "{print NF}"], b"one two three");
    assert_success(&result);
    assert_eq!(result.1, "3\n");
}

/// POSIX: awk arithmetic
#[test]
fn posix_awk_arithmetic() {
    let result = run_with_stdin(&["awk", "{print $1 + $2}"], b"10 20");
    assert_success(&result);
    assert_eq!(result.1, "30\n");
}

/// POSIX: awk string concatenation
#[test]
fn posix_awk_concat() {
    let result = run_with_stdin(&["awk", "{print $1 $2}"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "helloworld\n");
}

/// POSIX: awk conditional
#[test]
fn posix_awk_conditional() {
    let result = run_with_stdin(&["awk", "$1 > 5 {print}"], b"3\n7\n2\n9");
    assert_success(&result);
    assert!(result.1.contains("7"));
    assert!(result.1.contains("9"));
    assert!(!result.1.contains("3"));
}

/// POSIX: awk length function
#[test]
fn posix_awk_length() {
    let result = run_with_stdin(&["awk", "{print length($0)}"], b"hello");
    assert_success(&result);
    assert_eq!(result.1, "5\n");
}

/// POSIX: awk substr function
#[test]
fn posix_awk_substr() {
    let result = run_with_stdin(&["awk", "{print substr($0, 2, 3)}"], b"hello");
    assert_success(&result);
    assert_eq!(result.1, "ell\n");
}

/// POSIX: awk gsub function
#[test]
fn posix_awk_gsub() {
    let result = run_with_stdin(&["awk", "{gsub(/a/, \"b\"); print}"], b"aaa");
    assert_success(&result);
    assert_eq!(result.1, "bbb\n");
}

/// POSIX: awk split function
#[test]
fn posix_awk_split() {
    let result = run_with_stdin(
        &["awk", "{n=split($0, a, \":\"); print a[1], a[2]}"],
        b"foo:bar:baz",
    );
    assert_success(&result);
    assert_eq!(result.1, "foo bar\n");
}

/// POSIX: awk printf
#[test]
fn posix_awk_printf() {
    let result = run_with_stdin(&["awk", "{printf \"%05d\\n\", $1}"], b"42");
    assert_success(&result);
    assert_eq!(result.1, "00042\n");
}

/// POSIX: awk from file
#[test]
fn posix_awk_from_file() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    fs::write(&input, "hello world").unwrap();

    let result = run(&["awk", "{print $1}", input.to_str().unwrap()]);
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: awk -f program file
#[test]
fn posix_awk_program_file() {
    let dir = setup_test_env();
    let prog = dir.path().join("prog.awk");
    fs::write(&prog, "{print $1}").unwrap();

    let result = run_with_stdin(&["awk", "-f", prog.to_str().unwrap()], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_awk_exit_success() {
    let result = run_with_stdin(&["awk", "{print}"], b"test");
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_awk_exit_error() {
    let result = run(&["awk", "{print}", "/nonexistent/file"]);
    assert!(result.0 > 0);
}

/// POSIX: awk OFS (output field separator)
#[test]
fn posix_awk_ofs() {
    let result = run_with_stdin(&["awk", "BEGIN{OFS=\",\"}{print $1, $2}"], b"hello world");
    assert_success(&result);
    assert_eq!(result.1, "hello,world\n");
}

/// POSIX: awk ORS (output record separator)
#[test]
fn posix_awk_ors() {
    let result = run_with_stdin(&["awk", "BEGIN{ORS=\";\"}{print $0}"], b"a\nb");
    assert_success(&result);
    assert_eq!(result.1, "a;b;");
}

/// POSIX: awk arrays
#[test]
fn posix_awk_array() {
    let result = run_with_stdin(
        &["awk", "{a[$1]++} END{for(k in a) print k, a[k]}"],
        b"foo\nbar\nfoo",
    );
    assert_success(&result);
    assert!(result.1.contains("foo 2"));
    assert!(result.1.contains("bar 1"));
}

/// POSIX: awk regex match operator
#[test]
fn posix_awk_regex_match() {
    let result = run_with_stdin(&["awk", "$0 ~ /^hello/ {print}"], b"hello\nworld\nhello again");
    assert_success(&result);
    let lines: Vec<&str> = result.1.lines().collect();
    assert_eq!(lines.len(), 2);
}

/// POSIX: awk tolower/toupper
#[test]
fn posix_awk_case() {
    let result = run_with_stdin(&["awk", "{print tolower($0)}"], b"HELLO");
    assert_success(&result);
    assert_eq!(result.1, "hello\n");

    let result = run_with_stdin(&["awk", "{print toupper($0)}"], b"hello");
    assert_success(&result);
    assert_eq!(result.1, "HELLO\n");
}

/// POSIX: awk exit statement
#[test]
fn posix_awk_exit() {
    let result = run_with_stdin(&["awk", "{print; exit}"], b"line1\nline2\nline3");
    assert_success(&result);
    assert_eq!(result.1, "line1\n");
}

/// POSIX: awk next statement
#[test]
fn posix_awk_next() {
    let result = run_with_stdin(&["awk", "/skip/{next}{print}"], b"keep\nskip\nkeep");
    assert_success(&result);
    assert!(!result.1.contains("skip"));
}
