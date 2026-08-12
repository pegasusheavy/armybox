//! head - output the first part of files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/head.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// What to count when trimming output.
enum Mode {
    Lines(i64),
    Bytes(i64),
}

/// head - output the first part of files
///
/// # Synopsis
/// ```text
/// head [-n number | -c number] [file...]
/// ```
///
/// # Description
/// Copy the first N lines (default 10) of each FILE to standard output.
/// With no FILE, or when FILE is -, read standard input. When more than
/// one FILE is given, a `==> FILE <==` header is printed before each
/// file's output, separated by a blank line.
///
/// # Options
/// - `-n N`: Print the first N lines instead of the first 10. A negative
///   N means "all but the last N lines".
/// - `-c N`: Print the first N bytes instead of lines.
/// - `-N`: Legacy form of `-n N`.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred reading one or more files
pub fn head(argc: i32, argv: *const *const u8) -> i32 {
    let mut mode = Mode::Lines(10);

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut files: Vec<&'static [u8]> = Vec::new();

        let mut i = 1;
        while i < argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg.len() > 1 && arg[0] == b'-' && arg != b"-" {
                    if arg[1] == b'n' {
                        if arg.len() > 2 {
                            // -nN attached form
                            mode = Mode::Lines(sys::parse_i64(&arg[2..]).unwrap_or(10));
                        } else if i + 1 < argc {
                            if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                                mode = Mode::Lines(sys::parse_i64(n).unwrap_or(10));
                            }
                            i += 1;
                        }
                    } else if arg[1] == b'c' {
                        if arg.len() > 2 {
                            mode = Mode::Bytes(sys::parse_i64(&arg[2..]).unwrap_or(0));
                        } else if i + 1 < argc {
                            if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                                mode = Mode::Bytes(sys::parse_i64(n).unwrap_or(0));
                            }
                            i += 1;
                        }
                    } else if arg[1] >= b'0' && arg[1] <= b'9' {
                        // Legacy -N form
                        mode = Mode::Lines(sys::parse_i64(&arg[1..]).unwrap_or(10));
                    }
                    // Unknown option flag(s) are otherwise ignored.
                } else if let Some(path) = unsafe { get_arg(argv, i) } {
                    files.push(path);
                }
            }
            i += 1;
        }

        let mut exit_code = 0;

        if files.is_empty() {
            head_fd(0, &mode);
        } else {
            let multiple = files.len() > 1;
            for (idx, &path) in files.iter().enumerate() {
                if idx > 0 && multiple {
                    io::write_str(1, b"\n");
                }

                if path == b"-" {
                    if multiple {
                        print_header(b"standard input");
                    }
                    head_fd(0, &mode);
                } else {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd < 0 {
                        io::write_str(2, b"head: ");
                        sys::perror(path);
                        exit_code = 1;
                        continue;
                    }
                    if multiple {
                        print_header(path);
                    }
                    head_fd(fd, &mode);
                    io::close(fd);
                }
            }
        }

        exit_code
    }

    #[cfg(not(feature = "alloc"))]
    {
        let _ = argc;
        let _ = argv;
        let _ = &mode;
        io::write_str(2, b"head: requires alloc feature\n");
        1
    }
}

fn print_header(name: &[u8]) {
    io::write_str(1, b"==> ");
    io::write_all(1, name);
    io::write_str(1, b" <==\n");
}

/// Buffer used for batching output writes.
const OUT_BUF_SIZE: usize = 8192;

fn head_fd(fd: i32, mode: &Mode) {
    match mode {
        Mode::Bytes(n) => head_bytes(fd, *n),
        Mode::Lines(n) => {
            if *n >= 0 {
                head_lines(fd, *n as u64);
            } else {
                head_lines_all_but_last(fd, (-*n) as u64);
            }
        }
    }
}

/// Print the first `n` bytes of `fd`, buffering output.
fn head_bytes(fd: i32, n: i64) {
    if n <= 0 {
        return;
    }
    let mut remaining = n as u64;
    let mut in_buf = [0u8; 4096];
    let mut out_buf = [0u8; OUT_BUF_SIZE];
    let mut out_len = 0usize;

    while remaining > 0 {
        let r = io::read(fd, &mut in_buf);
        if r <= 0 {
            break;
        }
        let mut take = r as usize;
        if take as u64 > remaining {
            take = remaining as usize;
        }
        let mut off = 0usize;
        while off < take {
            let space = OUT_BUF_SIZE - out_len;
            let chunk = core::cmp::min(space, take - off);
            out_buf[out_len..out_len + chunk].copy_from_slice(&in_buf[off..off + chunk]);
            out_len += chunk;
            off += chunk;
            if out_len == OUT_BUF_SIZE {
                io::write_all(1, &out_buf[..out_len]);
                out_len = 0;
            }
        }
        remaining -= take as u64;
    }

    if out_len > 0 {
        io::write_all(1, &out_buf[..out_len]);
    }
}

/// Print the first `n` lines of `fd`, buffering output rather than
/// writing byte-by-byte.
fn head_lines(fd: i32, mut n: u64) {
    if n == 0 {
        return;
    }
    let mut in_buf = [0u8; 4096];
    let mut out_buf = [0u8; OUT_BUF_SIZE];
    let mut out_len = 0usize;

    'outer: loop {
        let r = io::read(fd, &mut in_buf);
        if r <= 0 {
            break;
        }
        for &b in &in_buf[..r as usize] {
            if out_len == OUT_BUF_SIZE {
                io::write_all(1, &out_buf[..out_len]);
                out_len = 0;
            }
            out_buf[out_len] = b;
            out_len += 1;
            if b == b'\n' {
                n -= 1;
                if n == 0 {
                    break 'outer;
                }
            }
        }
    }

    if out_len > 0 {
        io::write_all(1, &out_buf[..out_len]);
    }
}

/// Print all but the last `n` lines of `fd` (head -n -N).
///
/// Reads the whole input, buffering complete lines, then holds back the
/// last `n` of them.
#[cfg(feature = "alloc")]
fn head_lines_all_but_last(fd: i32, n: u64) {
    use alloc::vec::Vec;
    use alloc::collections::VecDeque;

    if n == 0 {
        // Nothing to hold back; print everything.
        head_lines(fd, u64::MAX);
        return;
    }

    let cap = n as usize;
    let mut queue: VecDeque<Vec<u8>> = VecDeque::with_capacity(cap + 1);
    let mut current_line = Vec::new();
    let mut buf = [0u8; 4096];

    loop {
        let r = io::read(fd, &mut buf);
        if r <= 0 {
            break;
        }
        for &b in &buf[..r as usize] {
            current_line.push(b);
            if b == b'\n' {
                queue.push_back(core::mem::take(&mut current_line));
                if queue.len() > cap {
                    if let Some(line) = queue.pop_front() {
                        io::write_all(1, &line);
                    }
                }
            }
        }
    }

    if !current_line.is_empty() {
        queue.push_back(current_line);
        if queue.len() > cap {
            if let Some(line) = queue.pop_front() {
                io::write_all(1, &line);
            }
        }
    }
    // Any lines left in `queue` are the last `n` lines; discard them.
}

#[cfg(not(feature = "alloc"))]
fn head_lines_all_but_last(_fd: i32, _n: u64) {
    io::write_str(2, b"head: -n negative requires alloc feature\n");
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
