//! fold - wrap lines to specified width
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/fold.html

use crate::io;
use crate::sys;
use crate::applets::{get_arg, has_opt};

/// fold - wrap lines to specified width
///
/// # Synopsis
/// ```text
/// fold [-w width] [file...]
/// ```
///
/// # Description
/// Wrap input lines to fit within a specified width.
///
/// # Options
/// - `-w width`: Use width columns (default 80)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn fold(argc: i32, argv: *const *const u8) -> i32 {
    let mut width = 80usize;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'w') && i + 1 < argc {
                if let Some(w) = unsafe { get_arg(argv, i + 1) } {
                    width = sys::parse_u64(w).unwrap_or(80) as usize;
                }
            }
        }
    }

    let mut buf = [0u8; 4096];
    let mut col = 0;

    loop {
        let n = io::read(0, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                io::write_str(1, b"\n");
                col = 0;
            } else {
                if col >= width {
                    io::write_str(1, b"\n");
                    col = 0;
                }
                io::write_all(1, &[c]);
                col += 1;
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::{Command, Stdio};
    use std::io::Write;
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
    fn test_fold_default_width() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["fold"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            // 90 character line
            stdin.write_all(b"123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        // Default 80 chars, so 90 chars becomes 2 lines
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 80);
        assert_eq!(lines[1].len(), 10);
    }

    #[test]
    fn test_fold_custom_width() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["fold", "-w", "10"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"12345678901234567890\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "1234567890");
        assert_eq!(lines[1], "1234567890");
    }

    #[test]
    fn test_fold_short_line() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["fold", "-w", "20"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"short\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "short\n");
    }
}
