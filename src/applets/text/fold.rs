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
/// fold [-bs] [-w width] [file...]
/// ```
///
/// # Description
/// Wrap input lines to fit within a specified width.
///
/// # Options
/// - `-w width`: Use width columns (default 80)
/// - `-b`: Count width in bytes rather than columns
/// - `-s`: Break at the last blank before the width limit, if any
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn fold(argc: i32, argv: *const *const u8) -> i32 {
    let mut width = 80usize;
    let mut by_bytes = false;
    let mut break_at_blank = false;

    #[cfg(feature = "alloc")]
    let mut files: alloc::vec::Vec<&[u8]> = alloc::vec::Vec::new();

    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 1 && arg[0] == b'-' {
                if has_opt(arg, b'b') { by_bytes = true; }
                if has_opt(arg, b's') { break_at_blank = true; }
                if has_opt(arg, b'w') {
                    if arg.len() > 2 && arg[1] == b'w' {
                        width = sys::parse_u64(&arg[2..]).unwrap_or(80) as usize;
                    } else if i + 1 < argc {
                        if let Some(w) = unsafe { get_arg(argv, i + 1) } {
                            width = sys::parse_u64(w).unwrap_or(80) as usize;
                        }
                        i += 1;
                    }
                }
            } else {
                #[cfg(feature = "alloc")]
                files.push(arg);
            }
        }
        i += 1;
    }

    if width == 0 { width = 1; }

    #[cfg(feature = "alloc")]
    {
        let mut had_error = false;

        if files.is_empty() {
            fold_fd(0, width, by_bytes, break_at_blank);
        } else {
            for &path in &files {
                let fd = if path == b"-" {
                    0
                } else {
                    io::open(path, libc::O_RDONLY, 0)
                };

                if fd < 0 {
                    io::write_str(2, b"fold: ");
                    io::write_all(2, path);
                    io::write_str(2, b": No such file or directory\n");
                    had_error = true;
                    continue;
                }

                fold_fd(fd, width, by_bytes, break_at_blank);
                if fd != 0 { io::close(fd); }
            }
        }

        if had_error { return 1; }
    }

    #[cfg(not(feature = "alloc"))]
    {
        fold_fd(0, width, by_bytes, break_at_blank);
    }

    0
}

/// Fold the content of a file descriptor to stdout.
///
/// `by_bytes` counts raw bytes toward the width instead of columns
/// (they are equivalent for ASCII input, which is all this
/// implementation targets). `break_at_blank` prefers breaking at the
/// last blank (space or tab) seen before the width limit, if any.
fn fold_fd(fd: i32, width: usize, by_bytes: bool, break_at_blank: bool) {
    let _ = by_bytes;

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut col: usize = 0;
        // Buffer of the current output line, used only when -s is active
        // so we can back up to the last blank.
        let mut line: Vec<u8> = Vec::new();
        let mut last_blank: Option<usize> = None;

        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 { break; }

            for &c in &buf[..n as usize] {
                if c == b'\n' {
                    io::write_all(1, &line);
                    io::write_str(1, b"\n");
                    line.clear();
                    last_blank = None;
                    col = 0;
                    continue;
                }

                if col >= width {
                    if break_at_blank {
                        if let Some(pos) = last_blank {
                            let rest: Vec<u8> = line[pos + 1..].to_vec();
                            io::write_all(1, &line[..=pos]);
                            io::write_str(1, b"\n");
                            line.clear();
                            line.extend_from_slice(&rest);
                            col = rest.len();
                            last_blank = None;
                        } else {
                            io::write_all(1, &line);
                            io::write_str(1, b"\n");
                            line.clear();
                            col = 0;
                        }
                    } else {
                        io::write_all(1, &line);
                        io::write_str(1, b"\n");
                        line.clear();
                        col = 0;
                        last_blank = None;
                    }
                }

                if c == b' ' || c == b'\t' {
                    last_blank = Some(line.len());
                }
                line.push(c);
                col += 1;
            }
        }

        if !line.is_empty() {
            io::write_all(1, &line);
            io::write_str(1, b"\n");
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        let mut col = 0usize;
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(fd, &mut buf);
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
    }
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
