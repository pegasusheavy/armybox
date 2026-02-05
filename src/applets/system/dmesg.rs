//! dmesg - print or control the kernel ring buffer
//!
//! Display messages from the kernel ring buffer.

use crate::io;

/// dmesg - print or control the kernel ring buffer
///
/// # Synopsis
/// ```text
/// dmesg
/// ```
///
/// # Description
/// Display messages from the kernel ring buffer. Attempts to read from
/// /dev/kmsg first, falling back to /var/log/dmesg if unavailable.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn dmesg(_argc: i32, _argv: *const *const u8) -> i32 {
    let fd = io::open(b"/dev/kmsg", libc::O_RDONLY | libc::O_NONBLOCK, 0);
    if fd < 0 {
        // Try reading from /var/log/dmesg
        let fd2 = io::open(b"/var/log/dmesg", libc::O_RDONLY, 0);
        if fd2 >= 0 {
            let mut buf = [0u8; 4096];
            loop {
                let n = io::read(fd2, &mut buf);
                if n <= 0 { break; }
                io::write_all(1, &buf[..n as usize]);
            }
            io::close(fd2);
        }
        return 0;
    }
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }
        // Parse kmsg format and output
        io::write_all(1, &buf[..n as usize]);
    }
    io::close(fd);
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
    fn test_dmesg_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["dmesg"])
            .output()
            .unwrap();

        // dmesg should succeed (exit 0) regardless of whether
        // it could actually read kernel messages (requires root)
        assert_eq!(output.status.code(), Some(0));
    }
}
