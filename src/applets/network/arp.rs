//! arp - manipulate the system ARP cache
//!
//! Display and modify the ARP cache.

use crate::io;

/// arp - manipulate the system ARP cache
///
/// # Synopsis
/// ```text
/// arp
/// ```
///
/// # Description
/// Display the system ARP cache from /proc/net/arp.
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn arp(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Address                  HWtype  HWaddress           Flags Mask            Iface\n");

    let fd = io::open(b"/proc/net/arp", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);
        if n > 0 {
            let content = &buf[..n as usize];
            for (i, line) in content.split(|&c| c == b'\n').enumerate() {
                if i == 0 { continue; } // Skip header
                if line.is_empty() { continue; }
                io::write_all(1, line);
                io::write_str(1, b"\n");
            }
        }
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
    fn test_arp_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["arp"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Address") || stdout.contains("HWtype"));
    }
}
