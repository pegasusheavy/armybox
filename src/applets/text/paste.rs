//! paste - merge lines of files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/paste.html

use crate::io;
use crate::applets::get_arg;

/// paste - merge lines of files
///
/// # Synopsis
/// ```text
/// paste [-s] [-d list] file...
/// ```
///
/// # Description
/// Merge corresponding or subsequent lines of files.
///
/// # Options
/// - `-d list`: Replace default delimiter TAB with list of characters
/// - `-s`: Paste one file at a time instead of in parallel
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn paste(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut delimiter = b'\t';
        let mut serial = false;
        let mut files: Vec<&[u8]> = Vec::new();

        let mut i = 1;
        while i < argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg.starts_with(b"-") && arg.len() > 1 {
                    if arg == b"-s" {
                        serial = true;
                    } else if arg == b"-d" || arg.starts_with(b"-d") {
                        // Delimiter
                        if arg.len() > 2 {
                            delimiter = arg[2];
                        } else if i + 1 < argc {
                            i += 1;
                            if let Some(d) = unsafe { get_arg(argv, i) } {
                                if !d.is_empty() {
                                    delimiter = d[0];
                                }
                            }
                        }
                    }
                } else if arg == b"-" {
                    files.push(b"-");
                } else {
                    files.push(arg);
                }
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(b"-");
        }

        if serial {
            // Serial mode: output each file on a single line, fields delimited
            for &file in &files {
                let fd = if file == b"-" {
                    0
                } else {
                    io::open(file, libc::O_RDONLY, 0)
                };
                if fd < 0 && file != b"-" {
                    io::write_str(2, b"paste: cannot open file\n");
                    continue;
                }

                let content = io::read_all(fd);
                if fd > 0 { io::close(fd); }

                let mut first = true;
                for line in content.split(|&c| c == b'\n') {
                    if line.is_empty() { continue; }
                    if !first {
                        io::write_all(1, &[delimiter]);
                    }
                    io::write_all(1, line);
                    first = false;
                }
                io::write_str(1, b"\n");
            }
        } else {
            // Normal mode: merge corresponding lines from each file
            let mut file_data: Vec<Vec<u8>> = Vec::new();
            let mut fds: Vec<i32> = Vec::new();

            for &file in &files {
                let fd = if file == b"-" {
                    0
                } else {
                    io::open(file, libc::O_RDONLY, 0)
                };
                if fd < 0 && file != b"-" {
                    io::write_str(2, b"paste: cannot open file\n");
                    file_data.push(Vec::new());
                    fds.push(-1);
                } else {
                    let content = io::read_all(fd);
                    if fd > 0 { io::close(fd); }
                    file_data.push(content);
                    fds.push(0);
                }
            }

            // Convert to lines
            let file_lines: Vec<Vec<&[u8]>> = file_data.iter()
                .map(|d| d.split(|&c| c == b'\n').collect::<Vec<_>>())
                .collect();

            // Find max number of lines
            let max_lines = file_lines.iter().map(|l| l.len()).max().unwrap_or(0);

            for line_idx in 0..max_lines {
                for (file_idx, lines) in file_lines.iter().enumerate() {
                    if file_idx > 0 {
                        io::write_all(1, &[delimiter]);
                    }
                    if line_idx < lines.len() {
                        io::write_all(1, lines[line_idx]);
                    }
                }
                io::write_str(1, b"\n");
            }
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        io::write_str(2, b"paste: requires alloc feature\n");
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
        let dir = std::env::temp_dir().join(format!("armybox_paste_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_paste_two_files() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file1 = dir.join("file1.txt");
        let file2 = dir.join("file2.txt");
        fs::write(&file1, "a\nb\nc\n").unwrap();
        fs::write(&file2, "1\n2\n3\n").unwrap();

        let output = Command::new(&armybox)
            .args(["paste", file1.to_str().unwrap(), file2.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("a\t1"));
        assert!(stdout.contains("b\t2"));
        assert!(stdout.contains("c\t3"));
        cleanup(&dir);
    }

    #[test]
    fn test_paste_serial() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "a\nb\nc\n").unwrap();

        let output = Command::new(&armybox)
            .args(["paste", "-s", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("a\tb\tc"));
        cleanup(&dir);
    }

    #[test]
    fn test_paste_custom_delimiter() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file1 = dir.join("file1.txt");
        let file2 = dir.join("file2.txt");
        fs::write(&file1, "a\nb\n").unwrap();
        fs::write(&file2, "1\n2\n").unwrap();

        let output = Command::new(&armybox)
            .args(["paste", "-d", ",", file1.to_str().unwrap(), file2.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("a,1"));
        assert!(stdout.contains("b,2"));
        cleanup(&dir);
    }
}
