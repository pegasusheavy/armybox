//! blockdev - call block device ioctls from the command line
//!
//! Get and set block device parameters.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

// Block device ioctls
const BLKGETSIZE64: u64 = 0x80081272;
const BLKFLSBUF: u64 = 0x1261;
const BLKRRPART: u64 = 0x125f;

/// blockdev - call block device ioctls from the command line
///
/// # Synopsis
/// ```text
/// blockdev [options] device
/// ```
///
/// # Description
/// Call block device ioctls from the command line.
///
/// # Options
/// - `--getsize64`: Print device size in bytes
/// - `--flushbufs`: Flush buffers
/// - `--rereadpt`: Reread partition table
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn blockdev(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"blockdev: usage: blockdev [option] device\n");
        return 1;
    }

    let option = unsafe { get_arg(argv, 1).unwrap() };
    let device = unsafe { get_arg(argv, 2).unwrap() };

    let fd = io::open(device, libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(device);
        return 1;
    }

    let result = if option == b"--getsize64" {
        let mut size: u64 = 0;
        if unsafe { libc::ioctl(fd, BLKGETSIZE64, &mut size) } < 0 {
            sys::perror(b"BLKGETSIZE64");
            -1
        } else {
            io::write_num(1, size);
            io::write_str(1, b"\n");
            0
        }
    } else if option == b"--flushbufs" {
        if unsafe { libc::ioctl(fd, BLKFLSBUF) } < 0 {
            sys::perror(b"BLKFLSBUF");
            -1
        } else {
            0
        }
    } else if option == b"--rereadpt" {
        if unsafe { libc::ioctl(fd, BLKRRPART) } < 0 {
            sys::perror(b"BLKRRPART");
            -1
        } else {
            0
        }
    } else {
        io::write_str(2, b"blockdev: unknown option\n");
        -1
    };

    io::close(fd);

    if result < 0 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
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
    fn test_blockdev_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["blockdev"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("usage"));
    }

    #[test]
    fn test_blockdev_unknown_option() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["blockdev", "--unknown", "/dev/null"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }
}
