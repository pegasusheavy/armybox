//! expr - evaluate expressions
//!
//! Evaluate expressions and print result.

use crate::io;
use crate::sys;
use super::get_arg;

/// expr - evaluate expressions
///
/// # Synopsis
/// ```text
/// expr EXPRESSION
/// ```
///
/// # Description
/// Evaluate EXPRESSION and print result.
///
/// # Exit Status
/// - 0: Expression is non-zero and non-null
/// - 1: Expression is zero or null
/// - 2: Invalid expression
pub fn expr(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 2; }

    if argc == 2 {
        let arg = unsafe { get_arg(argv, 1).unwrap() };
        io::write_all(1, arg);
        io::write_str(1, b"\n");
        return if arg.is_empty() || arg == b"0" { 1 } else { 0 };
    }

    if argc == 4 {
        let left = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(0);
        let op = unsafe { get_arg(argv, 2).unwrap() };
        let right = sys::parse_i64(unsafe { get_arg(argv, 3).unwrap() }).unwrap_or(0);

        let result = match op {
            b"+" => left + right,
            b"-" => left - right,
            b"*" => left * right,
            b"/" => if right != 0 { left / right } else { 0 },
            b"%" => if right != 0 { left % right } else { 0 },
            _ => 0,
        };

        io::write_signed(1, result);
        io::write_str(1, b"\n");
        return if result == 0 { 1 } else { 0 };
    }
    2
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() { return release; }
        manifest_dir.join("target/debug/armybox")
    }

    #[test]
    fn test_expr_addition() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "5", "+", "3"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "8");
    }

    #[test]
    fn test_expr_subtraction() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "10", "-", "3"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "7");
    }

    #[test]
    fn test_expr_multiplication() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "6", "*", "7"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "42");
    }

    #[test]
    fn test_expr_division() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "20", "/", "4"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "5");
    }

    #[test]
    fn test_expr_single_value() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "42"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "42");
    }
}
