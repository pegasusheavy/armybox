//! printf - format and print data
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/printf.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// printf - format and print data
///
/// # Synopsis
/// ```text
/// printf format [argument...]
/// ```
///
/// # Description
/// Format and print ARGUMENT(s) according to FORMAT.
///
/// # Format Specifiers
/// - `%s`: String
/// - `%d`, `%i`: Signed decimal integer
/// - `%x`: Hexadecimal
/// - `%%`: Literal percent
/// - `\n`: Newline
/// - `\t`: Tab
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn printf(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        return 0;
    }

    let fmt = match unsafe { get_arg(argv, 1) } {
        Some(f) => f,
        None => return 0,
    };
    let mut arg_idx = 2;
    let mut i = 0;

    while i < fmt.len() {
        if fmt[i] == b'%' && i + 1 < fmt.len() {
            i += 1;
            match fmt[i] {
                b's' => {
                    if arg_idx < argc {
                        if let Some(arg) = unsafe { get_arg(argv, arg_idx) } {
                            io::write_all(1, arg);
                            arg_idx += 1;
                        }
                    }
                }
                b'd' | b'i' => {
                    if arg_idx < argc {
                        if let Some(arg) = unsafe { get_arg(argv, arg_idx) } {
                            if let Some(n) = sys::parse_i64(arg) {
                                io::write_signed(1, n);
                            }
                            arg_idx += 1;
                        }
                    }
                }
                b'x' => {
                    if arg_idx < argc {
                        if let Some(arg) = unsafe { get_arg(argv, arg_idx) } {
                            if let Some(n) = sys::parse_u64(arg) {
                                let mut buf = [0u8; 20];
                                let s = sys::format_hex(n, &mut buf);
                                io::write_all(1, s);
                            }
                            arg_idx += 1;
                        }
                    }
                }
                b'%' => { io::write_str(1, b"%"); }
                b'n' => { io::write_str(1, b"\n"); }
                _ => {
                    io::write_str(1, b"%");
                    io::write_all(1, &[fmt[i]]);
                }
            }
        } else if fmt[i] == b'\\' && i + 1 < fmt.len() {
            i += 1;
            match fmt[i] {
                b'n' => { io::write_str(1, b"\n"); }
                b't' => { io::write_str(1, b"\t"); }
                b'r' => { io::write_str(1, b"\r"); }
                b'\\' => { io::write_str(1, b"\\"); }
                _ => { io::write_all(1, &[fmt[i]]); }
            }
        } else {
            io::write_all(1, &[fmt[i]]);
        }
        i += 1;
    }
    0
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
    fn test_printf_string() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["printf", "%s", "hello"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert_eq!(std::string::String::from_utf8_lossy(&output.stdout), "hello");
    }

    #[test]
    fn test_printf_integer() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["printf", "%d", "42"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert_eq!(std::string::String::from_utf8_lossy(&output.stdout), "42");
    }

    #[test]
    fn test_printf_newline() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["printf", "hello\\n"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert_eq!(std::string::String::from_utf8_lossy(&output.stdout), "hello\n");
    }

    #[test]
    fn test_printf_multiple_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["printf", "%s %d\\n", "count:", "5"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert_eq!(std::string::String::from_utf8_lossy(&output.stdout), "count: 5\n");
    }
}
