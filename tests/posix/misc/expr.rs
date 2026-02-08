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
