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
/// - `-m`: Write character count
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn wc(argc: i32, argv: *const *const u8) -> i32 {
    use crate::applets::get_arg;

    let mut show_lines = false;
    let mut show_words = false;
    let mut show_bytes = false;
    let mut show_chars = false;

    #[cfg(feature = "alloc")]
    let mut files: alloc::vec::Vec<&[u8]> = alloc::vec::Vec::new();

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 1 && arg[0] == b'-' {
                if has_opt(arg, b'l') { show_lines = true; }
                if has_opt(arg, b'w') { show_words = true; }
                if has_opt(arg, b'c') { show_bytes = true; }
                if has_opt(arg, b'm') { show_chars = true; }
            } else {
                #[cfg(feature = "alloc")]
                files.push(arg);
            }
        }
    }

    if !show_lines && !show_words && !show_bytes && !show_chars {
        show_lines = true;
        show_words = true;
        show_bytes = true;
    }

    #[cfg(feature = "alloc")]
    {
        let mut total_lines = 0u64;
        let mut total_words = 0u64;
        let mut total_bytes = 0u64;
        let mut total_chars = 0u64;
        let mut had_error = false;

        if files.is_empty() {
            let (l, w, b, c) = wc_fd(0);
            print_counts(show_lines, show_words, show_bytes, show_chars, l, w, b, c, None);
        } else {
            for &path in &files {
                let fd = if path == b"-" {
                    0
                } else {
                    io::open(path, libc::O_RDONLY, 0)
                };

                if fd < 0 {
                    io::write_str(2, b"wc: ");
                    io::write_all(2, path);
                    io::write_str(2, b": No such file or directory\n");
                    had_error = true;
                    continue;
                }

                let (l, w, b, c) = wc_fd(fd);
                if fd != 0 { io::close(fd); }

                total_lines += l;
                total_words += w;
                total_bytes += b;
                total_chars += c;

                print_counts(show_lines, show_words, show_bytes, show_chars, l, w, b, c, Some(path));
            }

            if files.len() > 1 {
                print_counts(show_lines, show_words, show_bytes, show_chars, total_lines, total_words, total_bytes, total_chars, Some(b"total"));
            }
        }

        if had_error { return 1; }
    }

    #[cfg(not(feature = "alloc"))]
    {
        let (l, w, b, c) = wc_fd(0);
        print_counts(show_lines, show_words, show_bytes, show_chars, l, w, b, c, None);
    }

    0
}

fn print_counts(
    show_lines: bool,
    show_words: bool,
    show_bytes: bool,
    show_chars: bool,
    lines: u64,
    words: u64,
    bytes: u64,
    chars: u64,
    name: Option<&[u8]>,
) {
    let mut first = true;
    if show_lines {
        if !first { io::write_str(1, b" "); }
        io::write_num(1, lines);
        first = false;
    }
    if show_words {
        if !first { io::write_str(1, b" "); }
        io::write_num(1, words);
        first = false;
    }
    if show_bytes {
        if !first { io::write_str(1, b" "); }
        io::write_num(1, bytes);
        first = false;
    }
    if show_chars {
        if !first { io::write_str(1, b" "); }
        io::write_num(1, chars);
        first = false;
    }
    if let Some(name) = name {
        if !first { io::write_str(1, b" "); }
        io::write_all(1, name);
    }
    io::write_str(1, b"\n");
}

fn wc_fd(fd: i32) -> (u64, u64, u64, u64) {
    let mut lines = 0u64;
    let mut words = 0u64;
    let mut bytes = 0u64;
    let mut chars = 0u64;
    let mut in_word = false;

    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            bytes += 1;
            // Count UTF-8 characters (code points): every byte that is not a
            // continuation byte (0b10xxxxxx) starts a new character.
            if (c & 0xC0) != 0x80 { chars += 1; }
            if c == b'\n' { lines += 1; }

            let is_space = c == b' ' || c == b'\n' || c == b'\t' || c == b'\r'
                || c == 0x0b || c == 0x0c;
            if is_space {
                in_word = false;
            } else if !in_word {
                in_word = true;
                words += 1;
            }
        }
    }

    (lines, words, bytes, chars)
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
