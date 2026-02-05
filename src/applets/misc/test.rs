//! test - evaluate conditional expressions
//!
//! POSIX conditional expression evaluation.

use crate::io;
use crate::sys;
use super::get_arg;

/// test - evaluate conditional expression
///
/// # Synopsis
/// ```text
/// test EXPRESSION
/// [ EXPRESSION ]
/// ```
///
/// # Description
/// Evaluate conditional expression and return 0 (true) or 1 (false).
///
/// # Exit Status
/// - 0: Expression is true
/// - 1: Expression is false or error
pub fn test(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }

    let arg1 = unsafe { get_arg(argv, 1).unwrap() };

    // Unary tests
    if argc == 3 {
        let op = arg1;
        let path = unsafe { get_arg(argv, 2).unwrap() };

        let mut st: libc::stat = unsafe { core::mem::zeroed() };
        let stat_ok = io::stat(path, &mut st) == 0;

        return match op {
            b"-e" => if stat_ok { 0 } else { 1 },
            b"-f" => if stat_ok && (st.st_mode & libc::S_IFMT) == libc::S_IFREG { 0 } else { 1 },
            b"-d" => if stat_ok && (st.st_mode & libc::S_IFMT) == libc::S_IFDIR { 0 } else { 1 },
            b"-r" => if unsafe { libc::access(path.as_ptr() as *const i8, libc::R_OK) } == 0 { 0 } else { 1 },
            b"-w" => if unsafe { libc::access(path.as_ptr() as *const i8, libc::W_OK) } == 0 { 0 } else { 1 },
            b"-x" => if unsafe { libc::access(path.as_ptr() as *const i8, libc::X_OK) } == 0 { 0 } else { 1 },
            b"-s" => if stat_ok && st.st_size > 0 { 0 } else { 1 },
            b"-n" => if !path.is_empty() { 0 } else { 1 },
            b"-z" => if path.is_empty() { 0 } else { 1 },
            b"-L" | b"-h" => if stat_ok && (st.st_mode & libc::S_IFMT) == libc::S_IFLNK { 0 } else { 1 },
            _ => 1,
        };
    }

    // Binary tests
    if argc == 4 {
        let left = arg1;
        let op = unsafe { get_arg(argv, 2).unwrap() };
        let right = unsafe { get_arg(argv, 3).unwrap() };

        return match op {
            b"=" | b"==" => if left == right { 0 } else { 1 },
            b"!=" => if left != right { 0 } else { 1 },
            b"-eq" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l == r { 0 } else { 1 }
            }
            b"-ne" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l != r { 0 } else { 1 }
            }
            b"-lt" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l < r { 0 } else { 1 }
            }
            b"-gt" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l > r { 0 } else { 1 }
            }
            b"-le" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l <= r { 0 } else { 1 }
            }
            b"-ge" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l >= r { 0 } else { 1 }
            }
            _ => 1,
        };
    }

    // Single arg - true if non-empty
    if !arg1.is_empty() { 0 } else { 1 }
}

/// bracket - alias for test ([ command)
pub fn bracket(argc: i32, argv: *const *const u8) -> i32 {
    test(argc, argv)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::path::PathBuf;
    use std::fs;

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
    fn test_file_exists() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = std::env::temp_dir().join("armybox_test_test");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("file.txt"), "test").unwrap();

        let output = Command::new(&armybox)
            .args(["test", "-e", dir.join("file.txt").to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_not_exists() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["test", "-e", "/nonexistent/path/file"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }

    #[test]
    fn test_string_equal() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["test", "foo", "=", "foo"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_string_not_equal() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["test", "foo", "!=", "bar"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_numeric_equal() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["test", "42", "-eq", "42"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_numeric_less_than() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["test", "5", "-lt", "10"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }
}
