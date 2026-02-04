//! awk - pattern scanning and processing
//!
//! POSIX.1-2017 compliant implementation (subset).
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// awk - pattern scanning and processing
///
/// # Synopsis
/// ```text
/// awk 'program' [file...]
/// ```
///
/// # Description
/// Read text from the input, applying pattern-action statements.
///
/// # Supported Programs
/// - `{print}` or `{print $0}`: Print entire line
/// - `{print $N}`: Print field N (1-indexed)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn awk(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"awk: missing program\n");
        return 1;
    }

    let program = unsafe { get_arg(argv, 1).unwrap() };

    // Very simple awk - just {print $N}
    let mut print_field: Option<usize> = None;
    let mut print_all = false;

    if program == b"{print}" || program == b"{print $0}" {
        print_all = true;
    } else if program.starts_with(b"{print $") {
        let end = program.iter().position(|&c| c == b'}').unwrap_or(program.len());
        if let Some(n) = sys::parse_u64(&program[8..end]) {
            print_field = Some(n as usize);
        }
    }

    let mut buf = [0u8; 4096];
    let mut line = [0u8; 4096];
    let mut line_len = 0;

    loop {
        let n = io::read(0, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                if print_all {
                    io::write_all(1, &line[..line_len]);
                    io::write_str(1, b"\n");
                } else if let Some(field) = print_field {
                    // Split by whitespace
                    let mut field_num = 0;
                    let mut start = 0;
                    let mut in_field = false;

                    for i in 0..=line_len {
                        let is_space = i == line_len || line[i] == b' ' || line[i] == b'\t';

                        if !in_field && !is_space {
                            in_field = true;
                            field_num += 1;
                            start = i;
                        } else if in_field && is_space {
                            if field_num == field {
                                io::write_all(1, &line[start..i]);
                                io::write_str(1, b"\n");
                                break;
                            }
                            in_field = false;
                        }
                    }
                }
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = c;
                line_len += 1;
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
    fn test_awk_print_all() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["awk", "{print}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"hello world\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "hello world\n");
    }

    #[test]
    fn test_awk_print_field() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["awk", "{print $2}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"one two three\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "two\n");
    }

    #[test]
    fn test_awk_print_first_field() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["awk", "{print $1}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"alpha beta gamma\ndelta epsilon\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["alpha", "delta"]);
    }

    #[test]
    fn test_awk_print_zero() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["awk", "{print $0}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"entire line\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "entire line\n");
    }
}
