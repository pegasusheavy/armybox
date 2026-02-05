//! base64 - base64 encode/decode
//!
//! Encode or decode data in base64.

use crate::io;
use super::has_opt;

/// base64 - base64 encode/decode
///
/// # Synopsis
/// ```text
/// base64 [-d] [FILE]
/// ```
///
/// # Description
/// Encode or decode base64 data.
///
/// # Options
/// - `-d`: Decode
///
/// # Exit Status
/// - 0: Success
pub fn base64(argc: i32, argv: *const *const u8) -> i32 {
    let mut decode = false;
    for i in 1..argc {
        if let Some(arg) = unsafe { super::get_arg(argv, i) } {
            if has_opt(arg, b'd') { decode = true; }
        }
    }

    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    if decode {
        // Decode
        let mut buf = [0u8; 4096];
        let n = io::read(0, &mut buf);
        if n > 0 {
            let mut i = 0;
            while i + 4 <= n as usize {
                let a = ALPHABET.iter().position(|&c| c == buf[i]).unwrap_or(0);
                let b = ALPHABET.iter().position(|&c| c == buf[i+1]).unwrap_or(0);
                let c = if buf[i+2] != b'=' { ALPHABET.iter().position(|&x| x == buf[i+2]).unwrap_or(0) } else { 0 };
                let d = if buf[i+3] != b'=' { ALPHABET.iter().position(|&x| x == buf[i+3]).unwrap_or(0) } else { 0 };

                io::write_all(1, &[((a << 2) | (b >> 4)) as u8]);
                if buf[i+2] != b'=' { io::write_all(1, &[(((b & 0xf) << 4) | (c >> 2)) as u8]); }
                if buf[i+3] != b'=' { io::write_all(1, &[(((c & 0x3) << 6) | d) as u8]); }
                i += 4;
            }
        }
    } else {
        // Encode
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 { break; }

            let mut i = 0;
            while i + 3 <= n as usize {
                let a = buf[i];
                let b = buf[i+1];
                let c = buf[i+2];
                io::write_all(1, &[ALPHABET[(a >> 2) as usize]]);
                io::write_all(1, &[ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize]]);
                io::write_all(1, &[ALPHABET[(((b & 0xf) << 2) | (c >> 6)) as usize]]);
                io::write_all(1, &[ALPHABET[(c & 0x3f) as usize]]);
                i += 3;
            }

            if i < n as usize {
                let a = buf[i];
                let b = if i + 1 < n as usize { buf[i+1] } else { 0 };
                io::write_all(1, &[ALPHABET[(a >> 2) as usize]]);
                io::write_all(1, &[ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize]]);
                if i + 1 < n as usize {
                    io::write_all(1, &[ALPHABET[((b & 0xf) << 2) as usize]]);
                    io::write_str(1, b"=");
                } else {
                    io::write_str(1, b"==");
                }
            }
        }
        io::write_str(1, b"\n");
    }
    0
}

/// base32 - base32 encode/decode
///
/// Usage: base32 [-d] [FILE]
/// RFC 4648 Base32 encoding/decoding.
#[cfg(feature = "alloc")]
pub fn base32(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    let mut decode = false;
    let mut input_file: Option<&[u8]> = None;
    let mut i = 1;

    // Parse arguments
    while i < argc as usize {
        let arg = match unsafe { super::get_arg(argv, i as i32) } {
            Some(a) => a,
            None => break,
        };

        if arg == b"-d" || arg == b"--decode" {
            decode = true;
        } else if arg == b"-w" || arg == b"--wrap" {
            i += 1; // Skip wrap column argument
        } else if !arg.starts_with(b"-") {
            input_file = Some(arg);
        }
        i += 1;
    }

    // Read input
    let fd = match input_file {
        Some(path) if path != b"-" => {
            let fd = io::open(path, libc::O_RDONLY, 0);
            if fd < 0 {
                io::write_str(2, b"base32: ");
                io::write_all(2, path);
                io::write_str(2, b": No such file\n");
                return 1;
            }
            fd
        }
        _ => 0,
    };

    let content = io::read_all(fd);
    if fd > 0 {
        io::close(fd);
    }

    if decode {
        // Decode
        let mut output: Vec<u8> = Vec::new();
        let mut buffer: u64 = 0;
        let mut bits = 0;

        for &c in &content {
            let val = match c {
                b'A'..=b'Z' => c - b'A',
                b'a'..=b'z' => c - b'a',
                b'2'..=b'7' => c - b'2' + 26,
                b'=' | b'\n' | b'\r' | b' ' | b'\t' => continue,
                _ => continue,
            };

            buffer = (buffer << 5) | (val as u64);
            bits += 5;

            if bits >= 8 {
                bits -= 8;
                output.push(((buffer >> bits) & 0xFF) as u8);
            }
        }

        io::write_all(1, &output);
    } else {
        // Encode
        let mut i = 0;
        let mut col = 0;

        while i < content.len() {
            // Take up to 5 bytes (40 bits) -> 8 base32 chars
            let mut buf: u64 = 0;
            let mut bytes = 0;

            for j in 0..5 {
                if i + j < content.len() {
                    buf = (buf << 8) | (content[i + j] as u64);
                    bytes += 1;
                } else {
                    buf <<= 8;
                }
            }

            let chars = (bytes * 8 + 4) / 5; // Round up

            for j in 0..8 {
                if j < chars {
                    let idx = ((buf >> (35 - j * 5)) & 0x1F) as usize;
                    io::write_all(1, &[BASE32_ALPHABET[idx]]);
                } else {
                    io::write_str(1, b"=");
                }
                col += 1;
                if col >= 76 {
                    io::write_str(1, b"\n");
                    col = 0;
                }
            }

            i += 5;
        }

        if col > 0 {
            io::write_str(1, b"\n");
        }
    }

    0
}

#[cfg(not(feature = "alloc"))]
pub fn base32(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"base32: requires alloc feature\n");
    1
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
    fn test_base64_encode() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["base64"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        child.stdin.as_mut().unwrap().write_all(b"Hello").unwrap();
        drop(child.stdin.take());

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "SGVsbG8=");
    }

    #[test]
    fn test_base64_decode() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["base64", "-d"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        child.stdin.as_mut().unwrap().write_all(b"SGVsbG8=").unwrap();
        drop(child.stdin.take());

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(&output.stdout, b"Hello");
    }
}
