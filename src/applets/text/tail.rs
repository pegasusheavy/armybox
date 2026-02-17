//! tail - output the last part of files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/tail.html

use crate::io;
use crate::sys;
use crate::applets::{get_arg, has_opt};

/// tail - output the last part of files
///
/// # Synopsis
/// ```text
/// tail [-n number] [file...]
/// ```
///
/// # Description
/// Copy the last N lines of each FILE to standard output.
/// With no FILE, or when FILE is -, read standard input.
///
/// # Options
/// - `-n N`: Print the last N lines instead of the last 10
/// - `-f`: Follow the file (output appended data as the file grows)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn tail(argc: i32, argv: *const *const u8) -> i32 {
    let mut lines = 10usize;
    let mut follow = false;
    let mut files_start = 1;

    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' {
                if has_opt(arg, b'f') { follow = true; }
                if has_opt(arg, b'n') && i + 1 < argc {
                    if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                        lines = sys::parse_u64(n).unwrap_or(10) as usize;
                    }
                    files_start = i + 2;
                    i += 1;
                } else if arg.len() > 1 && arg[1] >= b'0' && arg[1] <= b'9' {
                    lines = sys::parse_u64(&arg[1..]).unwrap_or(10) as usize;
                    files_start = i + 1;
                }
            }
        }
        i += 1;
    }

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;
        use alloc::collections::VecDeque;

        let fd = if files_start < argc {
            if let Some(path) = unsafe { get_arg(argv, files_start) } {
                if path == b"-" {
                    0
                } else {
                    let f = io::open(path, libc::O_RDONLY, 0);
                    if f < 0 { return 1; }
                    f
                }
            } else {
                0
            }
        } else {
            0
        };

        // Read all lines into ring buffer
        let mut line_buf: VecDeque<Vec<u8>> = VecDeque::with_capacity(lines + 1);
        let mut current_line = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 { break; }

            for j in 0..n as usize {
                current_line.push(buf[j]);
                if buf[j] == b'\n' {
                    if line_buf.len() >= lines {
                        line_buf.pop_front();
                    }
                    line_buf.push_back(core::mem::take(&mut current_line));
                }
            }
        }

        // Handle last line without newline
        if !current_line.is_empty() {
            if line_buf.len() >= lines {
                line_buf.pop_front();
            }
            line_buf.push_back(current_line);
        }

        // Output lines
        for line in line_buf {
            io::write_all(1, &line);
        }

        // Follow mode
        if follow && fd > 0 {
            loop {
                let n = io::read(fd, &mut buf);
                if n > 0 {
                    io::write_all(1, &buf[..n as usize]);
                } else {
                    unsafe { libc::usleep(100000) };
                }
            }
        }

        if fd > 0 { io::close(fd); }
    }

    #[cfg(not(feature = "alloc"))]
    {
        let _ = lines;
        let _ = follow;
        let _ = files_start;
        io::write_str(2, b"tail: requires alloc feature\n");
        return 1;
    }

    0
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
        let dir = std::env::temp_dir().join(format!("armybox_tail_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_tail_default() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        let content = (1..=20).map(|i| format!("line {}\n", i)).collect::<String>();
        fs::write(&file, &content).unwrap();

        let output = Command::new(&armybox)
            .args(["tail", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.lines().count(), 10);
        assert!(stdout.contains("line 20"));
        cleanup(&dir);
    }

    #[test]
    fn test_tail_n_option() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        let content = (1..=20).map(|i| format!("line {}\n", i)).collect::<String>();
        fs::write(&file, &content).unwrap();

        let output = Command::new(&armybox)
            .args(["tail", "-n", "5", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.lines().count(), 5);
        assert!(stdout.contains("line 16"));
        cleanup(&dir);
    }

    #[test]
    fn test_tail_short_file() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "line 1\nline 2\nline 3\n").unwrap();

        let output = Command::new(&armybox)
            .args(["tail", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.lines().count(), 3);
        cleanup(&dir);
    }
}
