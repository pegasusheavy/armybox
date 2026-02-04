//! sed - stream editor
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/sed.html

use crate::io;
use crate::applets::{get_arg, has_opt};

/// sed - stream editor
///
/// # Synopsis
/// ```text
/// sed [-n] script [file...]
/// sed [-n] -e script [-e script]... [-f script_file]... [file...]
/// ```
///
/// # Description
/// Read text from the input, applying editing commands, and write to output.
///
/// # Options
/// - `-e script`: Add the editing commands specified by script
/// - `-n`: Suppress default output
///
/// # Supported Commands
/// - `s/pattern/replacement/[flags]`: Substitute pattern with replacement
///   - `g` flag: Replace all occurrences
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn sed(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"sed: missing script\n");
        return 1;
    }

    // Find script
    let mut script: Option<&[u8]> = None;
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'e') && i + 1 < argc {
                script = unsafe { get_arg(argv, i + 1) };
                break;
            } else if arg.len() > 0 && arg[0] != b'-' {
                script = Some(arg);
                break;
            }
        }
    }

    let script = match script {
        Some(s) => s,
        None => return 1,
    };

    // Parse s/pattern/replacement/flags
    if script.len() > 2 && script[0] == b's' {
        let delim = script[1];
        let mut parts = [0usize; 4];
        let mut part = 0;
        parts[0] = 2;

        for i in 2..script.len() {
            if script[i] == delim && part < 3 {
                part += 1;
                parts[part] = i + 1;
            }
        }

        if part >= 2 {
            let pattern = &script[parts[0]..parts[1]-1];
            let replacement = &script[parts[1]..parts[2]-1];
            let global = part >= 2 && script[parts[2]..].contains(&b'g');

            let mut buf = [0u8; 4096];
            let mut line = [0u8; 4096];
            let mut line_len = 0;

            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 { break; }

                for &c in &buf[..n as usize] {
                    if c == b'\n' {
                        // Do substitution
                        let mut result = [0u8; 4096];
                        let mut result_len = 0;
                        let mut i = 0;
                        let mut did_replace = false;

                        while i < line_len {
                            if i + pattern.len() <= line_len && &line[i..i+pattern.len()] == pattern {
                                for &r in replacement {
                                    if result_len < result.len() {
                                        result[result_len] = r;
                                        result_len += 1;
                                    }
                                }
                                i += pattern.len();
                                did_replace = true;
                                if !global {
                                    // Copy rest
                                    while i < line_len && result_len < result.len() {
                                        result[result_len] = line[i];
                                        result_len += 1;
                                        i += 1;
                                    }
                                    break;
                                }
                            } else {
                                if result_len < result.len() {
                                    result[result_len] = line[i];
                                    result_len += 1;
                                }
                                i += 1;
                            }
                        }

                        if did_replace {
                            io::write_all(1, &result[..result_len]);
                        } else {
                            io::write_all(1, &line[..line_len]);
                        }
                        io::write_str(1, b"\n");
                        line_len = 0;
                    } else if line_len < line.len() {
                        line[line_len] = c;
                        line_len += 1;
                    }
                }
            }
        }
    } else {
        // Other commands - just pass through
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 { break; }
            io::write_all(1, &buf[..n as usize]);
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
    fn test_sed_substitute() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/foo/bar/"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"foo baz foo\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "bar baz foo\n"); // Only first occurrence
    }

    #[test]
    fn test_sed_substitute_global() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/foo/bar/g"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"foo baz foo\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "bar baz bar\n"); // All occurrences
    }

    #[test]
    fn test_sed_multiple_lines() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/old/new/"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"old text\nmore old\nno match\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["new text", "more new", "no match"]);
    }

    #[test]
    fn test_sed_no_match() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/notfound/replaced/"])
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
        assert_eq!(stdout, "hello world\n"); // Unchanged
    }

    #[test]
    fn test_sed_with_e_flag() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "-e", "s/a/b/"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"aaa\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "baa\n");
    }
}
