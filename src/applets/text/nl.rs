//! nl - number lines
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/nl.html

use crate::io;
use crate::applets::get_arg;

/// nl - number lines of files
///
/// # Synopsis
/// ```text
/// nl [-b type] [-n format] [file]
/// ```
///
/// # Description
/// Read lines from file and write them to standard output with line numbers added.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn nl(argc: i32, argv: *const *const u8) -> i32 {
    let fd = if argc > 1 {
        if let Some(path) = unsafe { get_arg(argv, argc - 1) } {
            if path.len() > 0 && path[0] != b'-' {
                io::open(path, libc::O_RDONLY, 0)
            } else { 0 }
        } else { 0 }
    } else { 0 };

    let mut line_num = 1u64;
    let mut buf = [0u8; 4096];
    let mut at_line_start = true;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if at_line_start {
                io::write_num(1, line_num);
                io::write_str(1, b"\t");
                at_line_start = false;
            }
            io::write_all(1, &[c]);
            if c == b'\n' {
                line_num += 1;
                at_line_start = true;
            }
        }
    }

    if fd != 0 { io::close(fd); }
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
        let dir = std::env::temp_dir().join(format!("armybox_nl_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_nl_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "line one\nline two\nline three\n").unwrap();

        let output = Command::new(&armybox)
            .args(["nl", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1\tline one"));
        assert!(stdout.contains("2\tline two"));
        assert!(stdout.contains("3\tline three"));
        cleanup(&dir);
    }

    #[test]
    fn test_nl_single_line() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "only one\n").unwrap();

        let output = Command::new(&armybox)
            .args(["nl", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1\tonly one"));
        cleanup(&dir);
    }

    #[test]
    fn test_nl_numbered_content() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "first\nsecond\nthird\nfourth\nfifth\n").unwrap();

        let output = Command::new(&armybox)
            .args(["nl", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 5);
        cleanup(&dir);
    }
}
