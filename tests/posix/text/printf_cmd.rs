//! POSIX.1-2017 compliance tests for printf
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/printf.html

use crate::posix::helpers::*;

/// POSIX: "The printf utility shall write formatted output"
#[test]
fn posix_printf_basic() {
    let result = run(&["printf", "hello"]);
    assert_success(&result);
    assert_eq!(result.1, "hello");
}

/// POSIX: printf %s string format
#[test]
fn posix_printf_string() {
    let result = run(&["printf", "%s\n", "hello"]);
    assert_success(&result);
    assert_eq!(result.1, "hello\n");
}

/// POSIX: printf %d integer format
#[test]
fn posix_printf_integer() {
    let result = run(&["printf", "%d\n", "42"]);
    assert_success(&result);
    assert_eq!(result.1, "42\n");
}

/// POSIX: printf %x hex format
#[test]
fn posix_printf_hex() {
    let result = run(&["printf", "%x\n", "255"]);
    assert_success(&result);
    assert_eq!(result.1, "ff\n");
}

/// POSIX: printf %o octal format
#[test]
fn posix_printf_octal() {
    let result = run(&["printf", "%o\n", "8"]);
    assert_success(&result);
    assert_eq!(result.1, "10\n");
}

/// POSIX: printf width specifier
#[test]
fn posix_printf_width() {
    let result = run(&["printf", "%5d\n", "42"]);
    assert_success(&result);
    assert_eq!(result.1, "   42\n");
}

/// POSIX: printf precision
#[test]
fn posix_printf_precision() {
    let result = run(&["printf", "%.3s\n", "hello"]);
    assert_success(&result);
    assert_eq!(result.1, "hel\n");
}

/// POSIX: printf escape sequences
#[test]
fn posix_printf_escapes() {
    let result = run(&["printf", "hello\\nworld"]);
    assert_success(&result);
    assert_eq!(result.1, "hello\nworld");
}

/// POSIX: printf \\t tab
#[test]
fn posix_printf_tab() {
    let result = run(&["printf", "a\\tb"]);
    assert_success(&result);
    assert_eq!(result.1, "a\tb");
}

/// POSIX: printf multiple arguments reuse format
#[test]
fn posix_printf_reuse() {
    let result = run(&["printf", "%s\n", "a", "b", "c"]);
    assert_success(&result);
    assert_eq!(result.1, "a\nb\nc\n");
}

/// POSIX: printf %% literal percent
#[test]
fn posix_printf_percent() {
    let result = run(&["printf", "100%%"]);
    assert_success(&result);
    assert_eq!(result.1, "100%");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_printf_exit_success() {
    let result = run(&["printf", "test"]);
    assert_eq!(result.0, 0);
}

/// POSIX: printf %c character
#[test]
fn posix_printf_char() {
    let result = run(&["printf", "%c", "A"]);
    assert_success(&result);
    assert_eq!(result.1, "A");
}

/// POSIX: printf leading zeros
#[test]
fn posix_printf_leading_zeros() {
    let result = run(&["printf", "%05d\n", "42"]);
    assert_success(&result);
    assert_eq!(result.1, "00042\n");
}

/// POSIX: printf left justify
#[test]
fn posix_printf_left_justify() {
    let result = run(&["printf", "%-5d|\n", "42"]);
    assert_success(&result);
    assert_eq!(result.1, "42   |\n");
}

/// POSIX: printf %.2f float with precision
#[test]
fn posix_printf_float_precision() {
    let result = run(&["printf", "%.2f\n", "3.14159"]);
    assert_success(&result);
    assert_eq!(result.1, "3.14\n");
}

/// POSIX: printf %b interprets backslash escapes in its argument
#[test]
fn posix_printf_b_escape() {
    let result = run(&["printf", "%b", "a\\tb\\n"]);
    assert_success(&result);
    assert_eq!(result.1, "a\tb\n");
}

/// POSIX: printf %b stops all output at \c
#[test]
fn posix_printf_b_c_stops_output() {
    let result = run(&["printf", "%b|after", "before\\cignored"]);
    assert_success(&result);
    assert_eq!(result.1, "before");
}

/// POSIX: printf %d with a non-numeric argument fails with diagnostic
#[test]
fn posix_printf_d_invalid_numeric() {
    let result = run(&["printf", "%d\n", "abc"]);
    assert_eq!(result.0, 1);
    assert!(result.2.contains("numeric"), "stderr was: {}", result.2);
}
