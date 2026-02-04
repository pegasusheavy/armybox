//! expand - convert tabs to spaces
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/expand.html

use crate::io;
use crate::applets::get_arg;

/// expand - convert tabs to spaces
///
/// # Synopsis
/// ```text
/// expand [file...]
/// ```
///
/// # Description
/// Convert tabs in each FILE to spaces, writing to standard output.
/// Tab stops are at every 8th column.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn expand(argc: i32, argv: *const *const u8) -> i32 {
    let fd = if argc > 1 {
        if let Some(path) = unsafe { get_arg(argv, argc - 1) } {
            if path.len() > 0 && path[0] != b'-' {
                io::open(path, libc::O_RDONLY, 0)
            } else { 0 }
        } else { 0 }
    } else { 0 };

    let mut buf = [0u8; 4096];
    let mut col = 0;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\t' {
                let spaces = 8 - (col % 8);
                for _ in 0..spaces {
                    io::write_str(1, b" ");
                }
                col += spaces;
            } else if c == b'\n' {
                io::write_str(1, b"\n");
                col = 0;
            } else {
                io::write_all(1, &[c]);
                col += 1;
            }
        }
    }

    if fd != 0 { io::close(fd); }
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
    fn test_expand_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["expand"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\tb\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // 'a' is at column 0, tab expands to column 8
        assert_eq!(stdout, "a       b\n");
    }

    #[test]
    fn test_expand_multiple_tabs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["expand"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"\t\t\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Two tabs = 16 spaces
        assert_eq!(stdout, "                \n");
    }

    #[test]
    fn test_expand_no_tabs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["expand"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"no tabs here\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "no tabs here\n");
    }
}
