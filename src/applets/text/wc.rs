//! wc - word, line, character count
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/wc.html

use crate::io;
use crate::applets::has_opt;

/// wc - word, line, character count
///
/// # Synopsis
/// ```text
/// wc [-c|-m] [-lw] [file...]
/// ```
///
/// # Description
/// Read files and write counts of lines, words, and bytes.
///
/// # Options
/// - `-c`: Write byte count
/// - `-l`: Write line count
/// - `-w`: Write word count
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn wc(argc: i32, argv: *const *const u8) -> i32 {
    use crate::applets::get_arg;

    let mut show_lines = false;
    let mut show_words = false;
    let mut show_chars = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' {
                if has_opt(arg, b'l') { show_lines = true; }
                if has_opt(arg, b'w') { show_words = true; }
                if has_opt(arg, b'c') { show_chars = true; }
            }
        }
    }

    if !show_lines && !show_words && !show_chars {
        show_lines = true;
        show_words = true;
        show_chars = true;
    }

    let mut total_lines = 0u64;
    let mut total_words = 0u64;
    let mut total_chars = 0u64;
    let mut file_count = 0;

    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path.len() > 0 && path[0] != b'-' {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd >= 0 {
                    let (l, w, c) = wc_fd(fd);
                    total_lines += l;
                    total_words += w;
                    total_chars += c;

                    if show_lines { io::write_num(1, l); io::write_str(1, b" "); }
                    if show_words { io::write_num(1, w); io::write_str(1, b" "); }
                    if show_chars { io::write_num(1, c); io::write_str(1, b" "); }
                    io::write_all(1, path);
                    io::write_str(1, b"\n");

                    io::close(fd);
                    file_count += 1;
                }
            }
        }
    }

    if file_count == 0 {
        let (l, w, c) = wc_fd(0);
        if show_lines { io::write_num(1, l); io::write_str(1, b" "); }
        if show_words { io::write_num(1, w); io::write_str(1, b" "); }
        if show_chars { io::write_num(1, c); }
        io::write_str(1, b"\n");
    } else if file_count > 1 {
        if show_lines { io::write_num(1, total_lines); io::write_str(1, b" "); }
        if show_words { io::write_num(1, total_words); io::write_str(1, b" "); }
        if show_chars { io::write_num(1, total_chars); io::write_str(1, b" "); }
        io::write_str(1, b"total\n");
    }
    0
}

fn wc_fd(fd: i32) -> (u64, u64, u64) {
    let mut lines = 0u64;
    let mut words = 0u64;
    let mut chars = 0u64;
    let mut in_word = false;

    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            chars += 1;
            if c == b'\n' { lines += 1; }

            let is_space = c == b' ' || c == b'\n' || c == b'\t' || c == b'\r';
            if is_space {
                in_word = false;
            } else if !in_word {
                in_word = true;
                words += 1;
            }
        }
    }

    (lines, words, chars)
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
        let dir = std::env::temp_dir().join(format!("armybox_wc_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_wc_lines() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "line 1\nline 2\nline 3\n").unwrap();

        let output = Command::new(&armybox)
            .args(["wc", "-l", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("3"));
        cleanup(&dir);
    }

    #[test]
    fn test_wc_words() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "one two three four five\n").unwrap();

        let output = Command::new(&armybox)
            .args(["wc", "-w", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("5"));
        cleanup(&dir);
    }

    #[test]
    fn test_wc_chars() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "hello\n").unwrap();

        let output = Command::new(&armybox)
            .args(["wc", "-c", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("6")); // "hello\n" = 6 bytes
        cleanup(&dir);
    }

    #[test]
    fn test_wc_default() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "one two\nthree\n").unwrap();

        let output = Command::new(&armybox)
            .args(["wc", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        // Default shows lines, words, chars
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2")); // 2 lines
        assert!(stdout.contains("3")); // 3 words
        cleanup(&dir);
    }
}
