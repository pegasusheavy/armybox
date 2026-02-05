//! cmp - compare two files byte by byte
//!
//! Compare two files and report the first difference.

use crate::io;
use crate::sys;
use super::get_arg;

/// cmp - compare two files
///
/// # Synopsis
/// ```text
/// cmp FILE1 FILE2
/// ```
///
/// # Description
/// Compare two files byte by byte.
///
/// # Exit Status
/// - 0: Files are identical
/// - 1: Files differ
/// - 2: Error
pub fn cmp(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 { return 2; }
    let path1 = match unsafe { get_arg(argv, 1) } { Some(p) => p, None => return 2 };
    let path2 = match unsafe { get_arg(argv, 2) } { Some(p) => p, None => return 2 };

    let fd1 = io::open(path1, libc::O_RDONLY, 0);
    let fd2 = io::open(path2, libc::O_RDONLY, 0);
    if fd1 < 0 || fd2 < 0 {
        if fd1 >= 0 { io::close(fd1); }
        if fd2 >= 0 { io::close(fd2); }
        return 2;
    }

    let mut buf1 = [0u8; 4096];
    let mut buf2 = [0u8; 4096];
    let mut byte_num = 0u64;
    let mut line_num = 1u64;
    let mut result = 0;

    'outer: loop {
        let n1 = io::read(fd1, &mut buf1);
        let n2 = io::read(fd2, &mut buf2);

        if n1 <= 0 && n2 <= 0 { break; }
        if n1 != n2 || n1 <= 0 || n2 <= 0 { result = 1; break; }

        for i in 0..n1 as usize {
            byte_num += 1;
            if buf1[i] == b'\n' { line_num += 1; }
            if buf1[i] != buf2[i] {
                let mut num_buf = [0u8; 20];
                io::write_all(1, path1);
                io::write_str(1, b" ");
                io::write_all(1, path2);
                io::write_str(1, b" differ: byte ");
                io::write_all(1, sys::format_u64(byte_num, &mut num_buf));
                io::write_str(1, b", line ");
                io::write_all(1, sys::format_u64(line_num, &mut num_buf));
                io::write_str(1, b"\n");
                result = 1;
                break 'outer;
            }
        }
    }

    io::close(fd1);
    io::close(fd2);
    result
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::path::PathBuf;
    use std::fs;

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
    fn test_cmp_identical() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = std::env::temp_dir().join("armybox_test_cmp");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("file1.txt"), "hello world").unwrap();
        fs::write(dir.join("file2.txt"), "hello world").unwrap();

        let output = Command::new(&armybox)
            .args(["cmp",
                dir.join("file1.txt").to_str().unwrap(),
                dir.join("file2.txt").to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cmp_different() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = std::env::temp_dir().join("armybox_test_cmp2");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("file1.txt"), "hello").unwrap();
        fs::write(dir.join("file2.txt"), "world").unwrap();

        let output = Command::new(&armybox)
            .args(["cmp",
                dir.join("file1.txt").to_str().unwrap(),
                dir.join("file2.txt").to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("differ"));
        let _ = fs::remove_dir_all(&dir);
    }
}
