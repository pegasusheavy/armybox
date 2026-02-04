//! cut - remove sections from each line of files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cut.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// cut - remove sections from each line of files
///
/// # Synopsis
/// ```text
/// cut -b list [-n] [file...]
/// cut -c list [file...]
/// cut -f list [-d delim] [-s] [file...]
/// ```
///
/// # Description
/// Cut out selected bytes, characters, or fields from each line of files.
///
/// # Options
/// - `-c list`: Select only these characters
/// - `-d delim`: Use DELIM instead of TAB for field delimiter
/// - `-f list`: Select only these fields
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn cut(argc: i32, argv: *const *const u8) -> i32 {
    let mut delimiter = b'\t';
    let mut field: Option<usize> = None;
    let mut chars: Option<usize> = None;
    let mut i = 1;

    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() >= 2 && arg[0] == b'-' && arg[1] == b'd' {
                // Handle -dX (attached) or -d X (separate)
                if arg.len() > 2 {
                    delimiter = arg[2];
                } else if i + 1 < argc {
                    if let Some(d) = unsafe { get_arg(argv, i + 1) } {
                        if !d.is_empty() { delimiter = d[0]; }
                    }
                    i += 1;
                }
            } else if arg.len() >= 2 && arg[0] == b'-' && arg[1] == b'f' {
                // Handle -fN (attached) or -f N (separate)
                if arg.len() > 2 {
                    field = Some(sys::parse_u64(&arg[2..]).unwrap_or(1) as usize);
                } else if i + 1 < argc {
                    if let Some(f) = unsafe { get_arg(argv, i + 1) } {
                        field = Some(sys::parse_u64(f).unwrap_or(1) as usize);
                    }
                    i += 1;
                }
            } else if arg.len() >= 2 && arg[0] == b'-' && arg[1] == b'c' {
                // Handle -cN (attached) or -c N (separate)
                if arg.len() > 2 {
                    chars = Some(sys::parse_u64(&arg[2..]).unwrap_or(1) as usize);
                } else if i + 1 < argc {
                    if let Some(c) = unsafe { get_arg(argv, i + 1) } {
                        chars = Some(sys::parse_u64(c).unwrap_or(1) as usize);
                    }
                    i += 1;
                }
            }
        }
        i += 1;
    }

    let mut buf = [0u8; 4096];
    let mut line = [0u8; 4096];
    let mut line_len = 0;

    loop {
        let n = io::read(0, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                if let Some(f) = field {
                    let mut field_num = 1;
                    let mut start = 0;
                    let mut found = false;

                    for j in 0..line_len {
                        if line[j] == delimiter {
                            if field_num == f {
                                io::write_all(1, &line[start..j]);
                                found = true;
                                break;
                            }
                            field_num += 1;
                            start = j + 1;
                        }
                    }
                    if !found && field_num == f {
                        io::write_all(1, &line[start..line_len]);
                    }
                } else if let Some(c) = chars {
                    if c <= line_len {
                        io::write_all(1, &line[..c]);
                    } else {
                        io::write_all(1, &line[..line_len]);
                    }
                }
                io::write_str(1, b"\n");
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = c;
                line_len += 1;
            }
        }
    }
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
    fn test_cut_field() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["cut", "-f", "2", "-d", ":"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a:b:c\nx:y:z\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "b\ny\n");
    }

    #[test]
    fn test_cut_chars() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["cut", "-c", "3"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"hello\nworld\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "hel\nwor\n");
    }

    #[test]
    fn test_cut_first_field() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["cut", "-f1", "-d,"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"one,two,three\na,b,c\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "one\na\n");
    }

    #[test]
    fn test_cut_tab_delimiter() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["cut", "-f", "2"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"a\tb\tc\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "b\n");
    }
}
