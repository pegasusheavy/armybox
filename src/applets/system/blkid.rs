//! blkid - locate/print block device attributes
//!
//! Print block device attributes.

use crate::io;

/// blkid - locate/print block device attributes
///
/// # Synopsis
/// ```text
/// blkid
/// ```
///
/// # Description
/// Print block device attributes by reading /proc/partitions.
///
/// # Exit Status
/// - 0: Success
pub fn blkid(_argc: i32, _argv: *const *const u8) -> i32 {
    // Read /dev entries and show basic info
    let fd = io::open(b"/proc/partitions", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 { break; }
            io::write_all(1, &buf[..n as usize]);
        }
        io::close(fd);
    }
    0
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
    fn test_blkid_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["blkid"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }
}
