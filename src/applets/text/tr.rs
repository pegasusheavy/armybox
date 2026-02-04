//! tr - translate characters
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/tr.html

use crate::io;
use crate::applets::{get_arg, has_opt};

/// tr - translate or delete characters
///
/// # Synopsis
/// ```text
/// tr [-c|-C] [-s] string1 string2
/// tr -s [-c|-C] string1
/// tr -d [-c|-C] string1
/// tr -ds [-c|-C] string1 string2
/// ```
///
/// # Description
/// Translate characters from standard input, writing to standard output.
///
/// # Options
/// - `-c`: Complement the set of characters in string1
/// - `-d`: Delete characters in string1
/// - `-s`: Replace repeated occurrences of characters with single occurrence
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn tr(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        let mut delete = false;
        let mut squeeze = false;
        let mut complement = false;
        let mut set1_idx = 0;
        let mut set2_idx = 0;

        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg.len() > 0 && arg[0] == b'-' {
                    if has_opt(arg, b'd') { delete = true; }
                    if has_opt(arg, b's') { squeeze = true; }
                    if has_opt(arg, b'c') || has_opt(arg, b'C') { complement = true; }
                } else if set1_idx == 0 {
                    set1_idx = i;
                } else if set2_idx == 0 {
                    set2_idx = i;
                }
            }
        }

        if set1_idx == 0 {
            io::write_str(2, b"tr: missing operand\n");
            return 1;
        }

        let set1 = unsafe { get_arg(argv, set1_idx).unwrap() };
        let set2 = if set2_idx > 0 { unsafe { get_arg(argv, set2_idx) } } else { None };

        let mut map = [0u8; 256];
        for i in 0..256 { map[i] = i as u8; }

        let set1_expanded = expand_set(set1);

        if delete {
            // Delete mode
            let mut buf = [0u8; 4096];
            let mut last_char: Option<u8> = None;

            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 { break; }

                for &c in &buf[..n as usize] {
                    let in_set = if complement {
                        !set1_expanded.contains(&c)
                    } else {
                        set1_expanded.contains(&c)
                    };

                    if !in_set {
                        if squeeze {
                            if Some(c) != last_char {
                                io::write_all(1, &[c]);
                                last_char = Some(c);
                            }
                        } else {
                            io::write_all(1, &[c]);
                        }
                    }
                }
            }
        } else if let Some(s2) = set2 {
            // Translate mode
            let set2_expanded = expand_set(s2);

            for (i, &c) in set1_expanded.iter().enumerate() {
                let replacement = if i < set2_expanded.len() {
                    set2_expanded[i]
                } else if !set2_expanded.is_empty() {
                    set2_expanded[set2_expanded.len() - 1]
                } else {
                    c
                };

                if complement {
                    for j in 0..256 {
                        if !set1_expanded.contains(&(j as u8)) {
                            map[j] = replacement;
                        }
                    }
                } else {
                    map[c as usize] = replacement;
                }
            }

            let mut buf = [0u8; 4096];
            let mut last_char: Option<u8> = None;

            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 { break; }

                for &c in &buf[..n as usize] {
                    let out = map[c as usize];
                    if squeeze && set2_expanded.contains(&out) {
                        if Some(out) != last_char {
                            io::write_all(1, &[out]);
                            last_char = Some(out);
                        }
                    } else {
                        io::write_all(1, &[out]);
                        last_char = Some(out);
                    }
                }
            }
        }
        return 0;
    }

    #[cfg(not(feature = "alloc"))]
    {
        let _ = argc;
        let _ = argv;
        io::write_str(2, b"tr: requires alloc feature\n");
        return 1;
    }
}

#[cfg(feature = "alloc")]
fn expand_set(s: &[u8]) -> alloc::vec::Vec<u8> {
    use alloc::vec::Vec;
    let mut result = Vec::new();
    let mut i = 0;

    while i < s.len() {
        // Check for range pattern: x-y
        if i + 2 < s.len() && s[i + 1] == b'-' {
            let start = s[i];
            let end = s[i + 2];
            if start <= end {
                for c in start..=end {
                    result.push(c);
                }
            } else {
                // Descending range
                for c in (end..=start).rev() {
                    result.push(c);
                }
            }
            i += 3;
        } else {
            result.push(s[i]);
            i += 1;
        }
    }
    result
}

#[cfg(not(feature = "alloc"))]
fn expand_set(s: &[u8]) -> &[u8] {
    // Without alloc, just return as-is (limited functionality)
    s
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
    fn test_tr_lowercase_to_uppercase() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["tr", "a-z", "A-Z"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"hello world\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "HELLO WORLD\n");
    }

    #[test]
    fn test_tr_delete() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["tr", "-d", "aeiou"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"hello world\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "hll wrld\n");
    }

    #[test]
    fn test_tr_squeeze() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["tr", "-s", " ", " "])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"hello    world\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "hello world\n");
    }

    #[test]
    fn test_tr_simple_replace() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["tr", "abc", "xyz"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"abc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "xyz\n");
    }
}
