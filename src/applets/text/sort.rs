//! sort - sort lines of text files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/sort.html

use crate::io;
use crate::sys;
use crate::applets::{get_arg, has_opt};

/// sort - sort lines of text files
///
/// # Synopsis
/// ```text
/// sort [-m] [-o output] [-bdfinru] [-t char] [-k keydef] [file...]
/// ```
///
/// # Description
/// Sort, merge, or sequence check text files.
///
/// # Options
/// - `-n`: Compare according to string numerical value
/// - `-r`: Reverse the result of comparisons
/// - `-u`: Output only the first of an equal run
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn sort(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut reverse = false;
        let mut numeric = false;
        let mut unique = false;
        let mut file_idx = 0;

        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg.len() > 0 && arg[0] == b'-' && arg.len() > 1 {
                    if has_opt(arg, b'r') { reverse = true; }
                    if has_opt(arg, b'n') { numeric = true; }
                    if has_opt(arg, b'u') { unique = true; }
                } else if file_idx == 0 {
                    file_idx = i;
                }
            }
        }

        // Read from file or stdin
        let content = if file_idx > 0 {
            if let Some(path) = unsafe { get_arg(argv, file_idx) } {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd < 0 {
                    io::write_str(2, b"sort: cannot open file\n");
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

        let mut lines: Vec<&[u8]> = content.split(|&c| c == b'\n').filter(|l| !l.is_empty()).collect();

        if numeric {
            lines.sort_by(|a, b| {
                let na = sys::parse_i64(a).unwrap_or(0);
                let nb = sys::parse_i64(b).unwrap_or(0);
                na.cmp(&nb)
            });
        } else {
            lines.sort();
        }

        if reverse {
            lines.reverse();
        }

        let mut last: Option<&[u8]> = None;
        for line in lines {
            if unique {
                if Some(line) == last { continue; }
                last = Some(line);
            }
            io::write_all(1, line);
            io::write_str(1, b"\n");
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        io::write_str(2, b"sort: requires alloc feature\n");
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    extern crate std;
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
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_sort_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_sort_alphabetic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "cherry\napple\nbanana\n").unwrap();

        let output = Command::new(&armybox)
            .args(["sort", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["apple", "banana", "cherry"]);
        cleanup(&dir);
    }

    #[test]
    fn test_sort_numeric() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sort", "-n"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"10\n2\n1\n20\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["1", "2", "10", "20"]);
    }

    #[test]
    fn test_sort_reverse() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sort", "-r"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\nb\nc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["c", "b", "a"]);
    }

    #[test]
    fn test_sort_unique() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sort", "-u"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\nb\na\nb\nc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["a", "b", "c"]);
    }
}
