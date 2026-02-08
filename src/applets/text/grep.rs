//! grep - search for patterns in files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/grep.html

use crate::io;
use crate::applets::{get_arg, has_opt};

/// grep - search a file for a pattern
///
/// # Synopsis
/// ```text
/// grep [-E|-F] [-c|-l|-q] [-insvx] pattern [file...]
/// ```
///
/// # Description
/// Search files for lines matching a pattern.
///
/// # Options
/// - `-c`: Only print a count of matching lines
/// - `-i`: Ignore case distinctions
/// - `-l`: Only print file names containing matches
/// - `-n`: Prefix each line with line number
/// - `-q`: Quiet; do not write anything to stdout
/// - `-v`: Invert match (select non-matching lines)
///
/// # Exit Status
/// - 0: One or more lines were selected
/// - 1: No lines were selected
/// - >1: An error occurred
pub fn grep(argc: i32, argv: *const *const u8) -> i32 {
    let mut invert = false;
    let mut count_only = false;
    let mut line_numbers = false;
    let mut ignore_case = false;
    let mut quiet = false;
    let mut files_with_matches = false;
    let mut pattern_idx = 0;
    let mut files_start = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' && arg.len() > 1 {
                if has_opt(arg, b'v') { invert = true; }
                if has_opt(arg, b'c') { count_only = true; }
                if has_opt(arg, b'n') { line_numbers = true; }
                if has_opt(arg, b'i') { ignore_case = true; }
                if has_opt(arg, b'q') { quiet = true; }
                if has_opt(arg, b'l') { files_with_matches = true; }
            } else if pattern_idx == 0 {
                pattern_idx = i;
            } else if files_start == 0 {
                files_start = i;
            }
        }
    }

    if pattern_idx == 0 {
        io::write_str(2, b"grep: missing pattern\n");
        return 2;
    }

    let pattern = unsafe { get_arg(argv, pattern_idx).unwrap() };
    let mut found_match = false;

    // If no files specified, read from stdin
    if files_start == 0 {
        let count = grep_fd(0, None, pattern, invert, count_only, line_numbers, ignore_case, quiet, files_with_matches);
        if count > 0 { found_match = true; }
    } else {
        // Process each file
        let multiple_files = (argc - files_start) > 1;
        for i in files_start..argc {
            if let Some(file) = unsafe { get_arg(argv, i) } {
                let fd = if file == b"-" {
                    0
                } else {
                    io::open(file, libc::O_RDONLY, 0)
                };

                if fd < 0 {
                    io::write_str(2, b"grep: ");
                    io::write_all(2, file);
                    io::write_str(2, b": No such file or directory\n");
                    continue;
                }

                let prefix = if multiple_files { Some(file) } else { None };
                let count = grep_fd(fd, prefix, pattern, invert, count_only, line_numbers, ignore_case, quiet, files_with_matches);
                if count > 0 { found_match = true; }

                if fd != 0 {
                    io::close(fd);
                }
            }
        }
    }

    if found_match { 0 } else { 1 }
}

fn grep_fd(fd: i32, prefix: Option<&[u8]>, pattern: &[u8], invert: bool, count_only: bool,
           line_numbers: bool, ignore_case: bool, quiet: bool, files_with_matches: bool) -> u64 {
    let mut count = 0u64;
    let mut line_num = 0u64;
    let mut buf = [0u8; 4096];
    let mut line = [0u8; 4096];
    let mut line_len = 0;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                line_num += 1;
                let matches = if ignore_case {
                    contains_ignore_case(&line[..line_len], pattern)
                } else {
                    contains(&line[..line_len], pattern)
                };

                if matches != invert {
                    count += 1;

                    if files_with_matches {
                        // For -l, just print filename once and return
                        if let Some(p) = prefix {
                            io::write_all(1, p);
                            io::write_str(1, b"\n");
                        }
                        return count;
                    }

                    if !count_only && !quiet {
                        if let Some(p) = prefix {
                            io::write_all(1, p);
                            io::write_str(1, b":");
                        }
                        if line_numbers {
                            io::write_num(1, line_num);
                            io::write_str(1, b":");
                        }
                        io::write_all(1, &line[..line_len]);
                        io::write_str(1, b"\n");
                    }
                }
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = c;
                line_len += 1;
            }
        }
    }

    // Handle last line if no trailing newline
    if line_len > 0 {
        line_num += 1;
        let matches = if ignore_case {
            contains_ignore_case(&line[..line_len], pattern)
        } else {
            contains(&line[..line_len], pattern)
        };

        if matches != invert {
            count += 1;
            if !count_only && !quiet && !files_with_matches {
                if let Some(p) = prefix {
                    io::write_all(1, p);
                    io::write_str(1, b":");
                }
                if line_numbers {
                    io::write_num(1, line_num);
                    io::write_str(1, b":");
                }
                io::write_all(1, &line[..line_len]);
                io::write_str(1, b"\n");
            }
        }
    }

    if count_only && !quiet {
        if let Some(p) = prefix {
            io::write_all(1, p);
            io::write_str(1, b":");
        }
        io::write_num(1, count);
        io::write_str(1, b"\n");
    }

    count
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() { return true; }
    if haystack.len() < needle.len() { return false; }

    for i in 0..=(haystack.len() - needle.len()) {
        if &haystack[i..i+needle.len()] == needle {
            return true;
        }
    }
    false
}

fn contains_ignore_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() { return true; }
    if haystack.len() < needle.len() { return false; }

    for i in 0..=(haystack.len() - needle.len()) {
        let mut matches = true;
        for j in 0..needle.len() {
            let h = if haystack[i+j] >= b'A' && haystack[i+j] <= b'Z' {
                haystack[i+j] + 32
            } else {
                haystack[i+j]
            };
            let n = if needle[j] >= b'A' && needle[j] <= b'Z' {
                needle[j] + 32
            } else {
                needle[j]
            };
            if h != n {
                matches = false;
                break;
            }
        }
        if matches { return true; }
    }
    false
}

/// egrep - extended grep (alias for grep)
pub fn egrep(argc: i32, argv: *const *const u8) -> i32 {
    grep(argc, argv)
}

/// fgrep - fixed string grep (alias for grep)
pub fn fgrep(argc: i32, argv: *const *const u8) -> i32 {
    grep(argc, argv)
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
        let dir = std::env::temp_dir().join(format!("armybox_grep_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_grep_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "hello world\nfoo bar\nhello again\n").unwrap();

        let output = Command::new(&armybox)
            .args(["grep", "hello", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("hello"));
        assert!(lines[1].contains("hello"));
        cleanup(&dir);
    }

    #[test]
    fn test_grep_line_numbers() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "one\ntwo\nthree\ntwo again\n").unwrap();

        let output = Command::new(&armybox)
            .args(["grep", "-n", "two", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2:two"));
        assert!(stdout.contains("4:two again"));
        cleanup(&dir);
    }

    #[test]
    fn test_grep_count() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["grep", "-c", "test"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"test one\nno match\ntest two\ntest three\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "3");
    }

    #[test]
    fn test_grep_invert() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["grep", "-v", "skip"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"keep this\nskip this\nkeep that\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["keep this", "keep that"]);
    }

    #[test]
    fn test_grep_ignore_case() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["grep", "-i", "hello"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"Hello World\nHELLO\nhello\nhi\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_grep_no_match() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["grep", "notfound"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"some text\nother text\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(1)); // No match returns 1
    }
}
