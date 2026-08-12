//! tail - output the last part of files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/tail.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// What to count when trimming output, and whether it is anchored to the
/// end (last N) or to the start (`+N` forms, meaning "starting at N").
enum Mode {
    /// Last N lines.
    LastLines(u64),
    /// Starting at line N (1-based).
    FromLine(u64),
    /// Last N bytes.
    LastBytes(u64),
    /// Starting at byte N (1-based).
    FromByte(u64),
}

/// Parse a `-n`/`-c` argument, handling the `+N` "from start" form.
fn parse_count(s: &[u8], last: fn(u64) -> Mode, start: fn(u64) -> Mode) -> Mode {
    if !s.is_empty() && s[0] == b'+' {
        let n = sys::parse_u64(&s[1..]).unwrap_or(0);
        start(n)
    } else {
        let n = sys::parse_u64(s).unwrap_or(10);
        last(n)
    }
}

/// tail - output the last part of files
///
/// # Synopsis
/// ```text
/// tail [-f] [-q] [-n number | -c number] [file...]
/// ```
///
/// # Description
/// Copy the last N lines (default 10) of each FILE to standard output.
/// With no FILE, or when FILE is -, read standard input. When more than
/// one FILE is given, a `==> FILE <==` header is printed before each
/// file's output, unless `-q` is given.
///
/// # Options
/// - `-n N`: Print the last N lines instead of the last 10. `-n +N`
///   prints starting at line N (1-based) instead.
/// - `-c N`: Print the last N bytes instead of lines. `-c +N` prints
///   starting at byte N (1-based) instead.
/// - `-N`: Legacy form of `-n N`.
/// - `-f`: Follow the file (output appended data as the file grows).
/// - `-q`: Never print file name headers.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred reading one or more files
pub fn tail(argc: i32, argv: *const *const u8) -> i32 {
    let mut mode = Mode::LastLines(10);
    let mut follow = false;
    let mut quiet = false;

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
                            let spec = &arg[2..];
                            mode = parse_count(spec, Mode::LastLines, Mode::FromLine);
                        } else if i + 1 < argc {
                            if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                                mode = parse_count(n, Mode::LastLines, Mode::FromLine);
                            }
                            i += 1;
                        }
                    } else if arg[1] == b'c' {
                        if arg.len() > 2 {
                            let spec = &arg[2..];
                            mode = parse_count(spec, Mode::LastBytes, Mode::FromByte);
                        } else if i + 1 < argc {
                            if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                                mode = parse_count(n, Mode::LastBytes, Mode::FromByte);
                            }
                            i += 1;
                        }
                    } else if arg[1] >= b'0' && arg[1] <= b'9' {
                        // Legacy -N form
                        mode = Mode::LastLines(sys::parse_u64(&arg[1..]).unwrap_or(10));
                    } else {
                        for &b in &arg[1..] {
                            match b {
                                b'f' => follow = true,
                                b'q' => quiet = true,
                                _ => {}
                            }
                        }
                    }
                } else if let Some(path) = unsafe { get_arg(argv, i) } {
                    files.push(path);
                }
            }
            i += 1;
        }

        let mut exit_code = 0;

        if files.is_empty() {
            tail_fd(0, &mode);
            follow_fd(0, follow);
        } else {
            let multiple = files.len() > 1 && !quiet;
            for (idx, &path) in files.iter().enumerate() {
                if idx > 0 && multiple {
                    io::write_str(1, b"\n");
                }

                if path == b"-" {
                    if multiple {
                        print_header(b"standard input");
                    }
                    tail_fd(0, &mode);
                    follow_fd(0, follow);
                } else {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd < 0 {
                        io::write_str(2, b"tail: ");
                        sys::perror(path);
                        exit_code = 1;
                        continue;
                    }
                    if multiple {
                        print_header(path);
                    }
                    tail_fd(fd, &mode);
                    follow_fd(fd, follow);
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
        let _ = follow;
        let _ = quiet;
        io::write_str(2, b"tail: requires alloc feature\n");
        1
    }
}

fn print_header(name: &[u8]) {
    io::write_str(1, b"==> ");
    io::write_all(1, name);
    io::write_str(1, b" <==\n");
}

fn tail_fd(fd: i32, mode: &Mode) {
    match mode {
        Mode::LastLines(n) => tail_last_lines(fd, *n),
        Mode::FromLine(n) => tail_from_line(fd, *n),
        Mode::LastBytes(n) => tail_last_bytes(fd, *n),
        Mode::FromByte(n) => tail_from_byte(fd, *n),
    }
}

/// If `-f` was requested and the fd supports it, keep reading and echoing
/// appended data. `fd <= 0` (stdin) is followed too, matching GNU tail's
/// behavior for pipes.
fn follow_fd(fd: i32, follow: bool) {
    if !follow {
        return;
    }
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n > 0 {
            io::write_all(1, &buf[..n as usize]);
        } else {
            unsafe { libc::usleep(100_000) };
        }
    }
}

/// Print the last `n` lines of `fd`.
///
/// Reads the whole input, keeping only the most recent `n` complete
/// (or final, possibly-unterminated) lines in a ring buffer, then
/// writes them out in one batch.
#[cfg(feature = "alloc")]
fn tail_last_lines(fd: i32, lines: u64) {
    use alloc::vec::Vec;
    use alloc::collections::VecDeque;

    let cap = lines as usize;
    let mut line_buf: VecDeque<Vec<u8>> = VecDeque::with_capacity(cap + 1);
    let mut current_line = Vec::new();
    let mut buf = [0u8; 4096];

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 {
            break;
        }

        for j in 0..n as usize {
            current_line.push(buf[j]);
            if buf[j] == b'\n' {
                if cap > 0 {
                    if line_buf.len() >= cap {
                        line_buf.pop_front();
                    }
                    line_buf.push_back(core::mem::take(&mut current_line));
                } else {
                    current_line.clear();
                }
            }
        }
    }

    if !current_line.is_empty() && cap > 0 {
        if line_buf.len() >= cap {
            line_buf.pop_front();
        }
        line_buf.push_back(current_line);
    }

    for line in line_buf {
        io::write_all(1, &line);
    }
}

/// Print lines starting at 1-based line number `n` (tail -n +N).
fn tail_from_line(fd: i32, n: u64) {
    let target = if n == 0 { 1 } else { n };
    let mut line_no: u64 = 1;
    let mut buf = [0u8; 4096];
    let mut out_buf = [0u8; 8192];
    let mut out_len = 0usize;

    loop {
        let r = io::read(fd, &mut buf);
        if r <= 0 {
            break;
        }
        for &b in &buf[..r as usize] {
            if line_no >= target {
                if out_len == out_buf.len() {
                    io::write_all(1, &out_buf[..out_len]);
                    out_len = 0;
                }
                out_buf[out_len] = b;
                out_len += 1;
            }
            if b == b'\n' {
                line_no += 1;
            }
        }
    }

    if out_len > 0 {
        io::write_all(1, &out_buf[..out_len]);
    }
}

/// Print the last `n` bytes of `fd`.
#[cfg(feature = "alloc")]
fn tail_last_bytes(fd: i32, n: u64) {
    use alloc::vec::Vec;
    use alloc::collections::VecDeque;

    let cap = n as usize;
    let mut ring: VecDeque<u8> = VecDeque::with_capacity(cap + 1);
    let mut buf = [0u8; 4096];

    loop {
        let r = io::read(fd, &mut buf);
        if r <= 0 {
            break;
        }
        for &b in &buf[..r as usize] {
            if cap > 0 {
                if ring.len() >= cap {
                    ring.pop_front();
                }
                ring.push_back(b);
            }
        }
    }

    let out: Vec<u8> = ring.into_iter().collect();
    io::write_all(1, &out);
}

/// Print bytes starting at 1-based byte offset `n` (tail -c +N).
fn tail_from_byte(fd: i32, n: u64) {
    let target = if n == 0 { 1 } else { n };
    let mut byte_no: u64 = 1;
    let mut buf = [0u8; 4096];
    let mut out_buf = [0u8; 8192];
    let mut out_len = 0usize;

    loop {
        let r = io::read(fd, &mut buf);
        if r <= 0 {
            break;
        }
        for &b in &buf[..r as usize] {
            if byte_no >= target {
                if out_len == out_buf.len() {
                    io::write_all(1, &out_buf[..out_len]);
                    out_len = 0;
                }
                out_buf[out_len] = b;
                out_len += 1;
            }
            byte_no += 1;
        }
    }

    if out_len > 0 {
        io::write_all(1, &out_buf[..out_len]);
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
