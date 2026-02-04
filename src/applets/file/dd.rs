//! dd - convert and copy a file
//!
//! GNU coreutils compatible implementation.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// dd - convert and copy a file
///
/// # Synopsis
/// ```text
/// dd [operand]...
/// ```
///
/// # Description
/// Copy a file, converting and formatting according to the operands.
///
/// # Operands
/// - `if=FILE`: Read from FILE instead of stdin
/// - `of=FILE`: Write to FILE instead of stdout
/// - `bs=BYTES`: Read and write up to BYTES bytes at a time
/// - `count=N`: Copy only N input blocks
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn dd(argc: i32, argv: *const *const u8) -> i32 {
    let mut if_path: Option<&[u8]> = None;
    let mut of_path: Option<&[u8]> = None;
    let mut bs: usize = 512;
    let mut count: Option<usize> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.starts_with(b"if=") {
                if_path = Some(&arg[3..]);
            } else if arg.starts_with(b"of=") {
                of_path = Some(&arg[3..]);
            } else if arg.starts_with(b"bs=") {
                bs = sys::parse_u64(&arg[3..]).unwrap_or(512) as usize;
            } else if arg.starts_with(b"count=") {
                count = Some(sys::parse_u64(&arg[6..]).unwrap_or(0) as usize);
            }
        }
    }

    let in_fd = match if_path {
        Some(p) => {
            let fd = io::open(p, libc::O_RDONLY, 0);
            if fd < 0 {
                sys::perror(p);
                return 1;
            }
            fd
        }
        None => 0,
    };

    let out_fd = match of_path {
        Some(p) => {
            let fd = io::open(p, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644);
            if fd < 0 {
                sys::perror(p);
                if in_fd != 0 { io::close(in_fd); }
                return 1;
            }
            fd
        }
        None => 1,
    };

    #[cfg(feature = "alloc")]
    {
        use alloc::vec;
        let mut buf = vec![0u8; bs];
        let mut blocks = 0;

        loop {
            if let Some(c) = count {
                if blocks >= c { break; }
            }

            let n = io::read(in_fd, &mut buf);
            if n <= 0 { break; }
            io::write_all(out_fd, &buf[..n as usize]);
            blocks += 1;
        }

        io::write_num(2, blocks as u64);
        io::write_str(2, b"+0 records in\n");
        io::write_num(2, blocks as u64);
        io::write_str(2, b"+0 records out\n");
    }

    #[cfg(not(feature = "alloc"))]
    {
        let mut buf = [0u8; 512];
        let mut blocks = 0;

        loop {
            if let Some(c) = count {
                if blocks >= c { break; }
            }

            let n = io::read(in_fd, &mut buf);
            if n <= 0 { break; }
            io::write_all(out_fd, &buf[..n as usize]);
            blocks += 1;
        }

        let _ = bs;
    }

    if in_fd != 0 { io::close(in_fd); }
    if out_fd != 1 { io::close(out_fd); }
    0
}

#[cfg(test)]
mod tests {
    extern crate std;
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
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_dd_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_dd_copy_file() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let input = dir.join("input.txt");
        let output = dir.join("output.txt");
        fs::write(&input, "hello world").unwrap();

        let cmd_output = Command::new(&armybox)
            .args([
                "dd",
                &format!("if={}", input.to_str().unwrap()),
                &format!("of={}", output.to_str().unwrap()),
            ])
            .output()
            .unwrap();

        assert_eq!(cmd_output.status.code(), Some(0));
        assert!(output.exists());
        assert_eq!(fs::read_to_string(&output).unwrap(), "hello world");
        cleanup(&dir);
    }

    #[test]
    fn test_dd_with_count() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let input = dir.join("input.txt");
        let output = dir.join("output.txt");
        // Write more than 512 bytes
        fs::write(&input, "x".repeat(1024)).unwrap();

        let cmd_output = Command::new(&armybox)
            .args([
                "dd",
                &format!("if={}", input.to_str().unwrap()),
                &format!("of={}", output.to_str().unwrap()),
                "count=1",
            ])
            .output()
            .unwrap();

        assert_eq!(cmd_output.status.code(), Some(0));
        assert!(output.exists());
        // With default bs=512 and count=1, should copy at most 512 bytes
        assert!(fs::read(&output).unwrap().len() <= 512);
        cleanup(&dir);
    }

    #[test]
    fn test_dd_with_bs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let input = dir.join("input.txt");
        let output = dir.join("output.txt");
        fs::write(&input, "hello").unwrap();

        let cmd_output = Command::new(&armybox)
            .args([
                "dd",
                &format!("if={}", input.to_str().unwrap()),
                &format!("of={}", output.to_str().unwrap()),
                "bs=1024",
            ])
            .output()
            .unwrap();

        assert_eq!(cmd_output.status.code(), Some(0));
        assert_eq!(fs::read_to_string(&output).unwrap(), "hello");
        cleanup(&dir);
    }

    #[test]
    fn test_dd_nonexistent_input() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let output = dir.join("output.txt");

        let cmd_output = Command::new(&armybox)
            .args([
                "dd",
                "if=/nonexistent/file",
                &format!("of={}", output.to_str().unwrap()),
            ])
            .output()
            .unwrap();

        assert_ne!(cmd_output.status.code(), Some(0));
        cleanup(&dir);
    }
}
