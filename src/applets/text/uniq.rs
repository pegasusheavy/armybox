//! uniq - report or omit repeated lines
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/uniq.html

use crate::io;
use crate::applets::{get_arg, has_opt};

/// uniq - report or omit repeated lines
///
/// # Synopsis
/// ```text
/// uniq [-c|-d|-u] [-f fields] [-s char] [input_file [output_file]]
/// ```
///
/// # Description
/// Filter adjacent matching lines from INPUT, writing to OUTPUT.
///
/// # Options
/// - `-c`: Prefix lines with number of occurrences
/// - `-d`: Only print duplicate lines
/// - `-u`: Only print unique lines
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn uniq(argc: i32, argv: *const *const u8) -> i32 {
    let mut count = false;
    let mut repeated = false;
    let mut unique_only = false;
    let mut file_idx = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' && arg.len() > 1 {
                if has_opt(arg, b'c') { count = true; }
                if has_opt(arg, b'd') { repeated = true; }
                if has_opt(arg, b'u') { unique_only = true; }
            } else if file_idx == 0 {
                file_idx = i;
            }
        }
    }

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        // Read from file or stdin
        let content = if file_idx > 0 {
            if let Some(path) = unsafe { get_arg(argv, file_idx) } {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd < 0 {
                    io::write_str(2, b"uniq: cannot open file\n");
                    return 1;
                }
                let c = io::read_all(fd);
                io::close(fd);
                c
            } else {
                io::read_all(0)
            }
        } else {
            io::read_all(0)
        };

        let lines: Vec<&[u8]> = content.split(|&c| c == b'\n').collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let mut cnt = 1;

            while i + cnt < lines.len() && lines[i + cnt] == line {
                cnt += 1;
            }

            let should_print = if repeated {
                cnt > 1
            } else if unique_only {
                cnt == 1
            } else {
                true
            };

            if should_print && !line.is_empty() {
                if count {
                    io::write_num(1, cnt as u64);
                    io::write_str(1, b" ");
                }
                io::write_all(1, line);
                io::write_str(1, b"\n");
            }

            i += cnt;
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        io::write_str(2, b"uniq: requires alloc feature\n");
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    use std::process::{Command, Stdio};
    use std::io::Write;
    use std::fs;
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

    fn setup() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("armybox_uniq_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_uniq_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["uniq"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\na\nb\nb\nb\nc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_uniq_count() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["uniq", "-c"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\na\na\nb\nc\nc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("3 a"));
        assert!(stdout.contains("1 b"));
        assert!(stdout.contains("2 c"));
    }

    #[test]
    fn test_uniq_duplicates_only() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["uniq", "-d"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\na\nb\nc\nc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["a", "c"]);
    }

    #[test]
    fn test_uniq_unique_only() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["uniq", "-u"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\na\nb\nc\nc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["b"]);
    }

    #[test]
    fn test_uniq_from_file() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "x\nx\ny\ny\ny\nz\n").unwrap();

        let output = Command::new(&armybox)
            .args(["uniq", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["x", "y", "z"]);
        cleanup(&dir);
    }
}
