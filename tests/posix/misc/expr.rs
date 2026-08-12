//! POSIX.1-2017 compliance tests for expr
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/expr.html

use crate::posix::helpers::*;

/// POSIX: "The expr utility shall evaluate an expression"
#[test]
fn posix_expr_addition() {
    let result = run(&["expr", "2", "+", "3"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "5");
}

/// POSIX: expr subtraction
#[test]
fn posix_expr_subtraction() {
    let result = run(&["expr", "10", "-", "3"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "7");
}

/// POSIX: expr multiplication
#[test]
fn posix_expr_multiplication() {
    let result = run(&["expr", "4", "*", "5"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "20");
}

/// POSIX: expr division
#[test]
fn posix_expr_division() {
    let result = run(&["expr", "15", "/", "3"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "5");
}

/// POSIX: expr modulo
#[test]
fn posix_expr_modulo() {
    let result = run(&["expr", "17", "%", "5"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "2");
}

/// POSIX: expr comparison (equal)
#[test]
fn posix_expr_equal() {
    let result = run(&["expr", "5", "=", "5"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "1"); // true

    let result = run(&["expr", "5", "=", "6"]);
    assert_eq!(result.0, 1);
    assert_eq!(result.1.trim(), "0"); // false
}

/// POSIX: expr comparison (not equal)
#[test]
fn posix_expr_not_equal() {
    let result = run(&["expr", "5", "!=", "6"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "1");
}

/// POSIX: expr comparison (less than)
#[test]
fn posix_expr_less_than() {
    let result = run(&["expr", "3", "<", "5"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "1");
}

/// POSIX: expr comparison (greater than)
#[test]
fn posix_expr_greater_than() {
    let result = run(&["expr", "5", ">", "3"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "1");
}

/// POSIX: expr string length
#[test]
fn posix_expr_length() {
    let result = run(&["expr", "length", "hello"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "5");
}

/// POSIX: expr substr
#[test]
fn posix_expr_substr() {
    let result = run(&["expr", "substr", "hello", "2", "3"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "ell");
}

/// POSIX: expr index
#[test]
fn posix_expr_index() {
    let result = run(&["expr", "index", "hello", "l"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "3");
}

/// POSIX: expr regex match (:)
#[test]
fn posix_expr_match() {
    let result = run(&["expr", "hello", ":", "hel"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "3");
}

/// POSIX: Exit status 0 if result is non-zero/non-null
#[test]
fn posix_expr_exit_nonzero_result() {
    let result = run(&["expr", "1", "+", "1"]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status 1 if result is zero/null
#[test]
fn posix_expr_exit_zero_result() {
    let result = run(&["expr", "0"]);
    assert_eq!(result.0, 1);
}

/// POSIX: Exit status 2 on error
#[test]
fn posix_expr_exit_error() {
    let result = run(&["expr", "1", "/", "0"]);
    assert_eq!(result.0, 2);
}

/// POSIX: `:` with a `\(...\)` capture group returns the captured substring.
#[test]
fn posix_expr_match_capture_group() {
    let result = run(&["expr", "hello-world", ":", r"\(hello\)-.*"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "hello");
}

/// POSIX: the `match STR RE` keyword form is equivalent to `STR : RE`.
#[test]
fn posix_expr_match_keyword() {
    let result = run(&["expr", "match", "hello", "hel"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "3");

    let result = run(&["expr", "match", "hello-world", r"\(hello\)-.*"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "hello");
}

/// POSIX: interval quantifier `\{n,m\}` in the BRE engine.
#[test]
fn posix_expr_match_interval() {
    // Exactly three digits.
    let result = run(&["expr", "12345", ":", r"[0-9]\{3\}"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "3");

    // Bounded {2,4}: greedy, matches four.
    let result = run(&["expr", "aaaaa", ":", r"a\{2,4\}"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "4");

    // Fewer than the minimum: no match.
    let result = run(&["expr", "a", ":", r"a\{2,4\}"]);
    assert_eq!(result.0, 1);
    assert_eq!(result.1.trim(), "0");
}

/// POSIX: logical OR `|` returns the first operand if non-null/non-zero,
/// otherwise the second.
#[test]
fn posix_expr_logical_or() {
    let result = run(&["expr", "0", "|", "5"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "5");
}

/// POSIX: logical AND `&` returns the first operand if both are non-null and
/// non-zero, otherwise 0.
#[test]
fn posix_expr_logical_and() {
    let result = run(&["expr", "1", "&", "0"]);
    assert_eq!(result.0, 1);
    assert_eq!(result.1.trim(), "0");
}

/// POSIX: parenthesized grouping overrides precedence.
#[test]
fn posix_expr_grouping() {
    let result = run(&["expr", "(", "1", "+", "2", ")", "*", "3"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "9");
}

/// POSIX: comparison of non-integer operands is lexicographic on bytes.
#[test]
fn posix_expr_string_comparison() {
    let result = run(&["expr", "abc", "<", "abd"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "1");
}

/// Overflow in arithmetic is an error (exit 2), not a silent wrap.
#[test]
fn posix_expr_overflow_errors() {
    let result = run(&["expr", "9223372036854775807", "+", "1"]);
    assert_eq!(result.0, 2);
    assert!(result.2.contains("overflow"), "stderr: {}", result.2);
}

/// A literal at the edge of the i64 range must not panic; a magnitude beyond
/// the range is treated as a string operand.
#[test]
fn posix_expr_min_operand_no_panic() {
    // Would panic via `-(i64::MIN)` if parsed as an integer during negation.
    let result = run(&["expr", "-9223372036854775808", "+", "0"]);
    assert_eq!(result.0, 0);
    assert_eq!(result.1.trim(), "-9223372036854775808");
}

/// Division by zero writes a diagnostic to stderr and exits 2.
#[test]
fn posix_expr_division_by_zero_message() {
    let result = run(&["expr", "1", "/", "0"]);
    assert_eq!(result.0, 2);
    assert!(
        result.2.contains("division by zero"),
        "stderr: {}",
        result.2
    );
}
