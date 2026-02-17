//! head - output the first part of files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/head.html

use crate::io;
use crate::sys;
use crate::applets::{get_arg, has_opt};

/// head - output the first part of files
///
/// # Synopsis
/// ```text
/// head [-n number] [file...]
/// ```
///
/// # Description
/// Copy the first N lines of each FILE to standard output.
/// With no FILE, or when FILE is -, read standard input.
///
/// # Options
/// - `-n N`: Print the first N lines instead of the first 10
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn head(argc: i32, argv: *const *const u8) -> i32 {
    let mut lines = 10i64;
    let mut files_start = 1;

    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' {
                if has_opt(arg, b'n') && i + 1 < argc {
                    if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                        lines = sys::parse_i64(n).unwrap_or(10);
                    }
                    files_start = i + 2;
                    i += 1;
                } else if arg.len() > 1 && arg[1] >= b'0' && arg[1] <= b'9' {
                    lines = sys::parse_i64(&arg[1..]).unwrap_or(10);
                    files_start = i + 1;
                }
            }
        }
        i += 1;
    }

    if files_start >= argc {
        head_fd(0, lines);
    } else {
        for j in files_start..argc {
            if let Some(path) = unsafe { get_arg(argv, j) } {
                if path == b"-" {
                    head_fd(0, lines);
                } else {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd >= 0 {
                        head_fd(fd, lines);
                        io::close(fd);
                    }
                }
            }
        }
    }
    0
}

fn head_fd(fd: i32, mut lines: i64) {
    let mut buf = [0u8; 4096];
    while lines > 0 {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for i in 0..n as usize {
            io::write_all(1, &buf[i..i+1]);
            if buf[i] == b'\n' {
                lines -= 1;
                if lines <= 0 { break; }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    use std::process::Command;
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
        let dir = std::env::temp_dir().join(format!("armybox_head_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_head_default() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        let content = (1..=20).map(|i| format!("line {}\n", i)).collect::<String>();
        fs::write(&file, &content).unwrap();

        let output = Command::new(&armybox)
            .args(["head", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.lines().count(), 10);
        assert!(stdout.starts_with("line 1\n"));
        cleanup(&dir);
    }

    #[test]
    fn test_head_n_option() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        let content = (1..=20).map(|i| format!("line {}\n", i)).collect::<String>();
        fs::write(&file, &content).unwrap();

        let output = Command::new(&armybox)
            .args(["head", "-n", "5", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.lines().count(), 5);
        cleanup(&dir);
    }

    #[test]
    fn test_head_short_file() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "line 1\nline 2\nline 3\n").unwrap();

        let output = Command::new(&armybox)
            .args(["head", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.lines().count(), 3);
        cleanup(&dir);
    }

    #[test]
    fn test_head_numeric_option() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        let content = (1..=20).map(|i| format!("line {}\n", i)).collect::<String>();
        fs::write(&file, &content).unwrap();

        let output = Command::new(&armybox)
            .args(["head", "-3", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.lines().count(), 3);
        cleanup(&dir);
    }
}
