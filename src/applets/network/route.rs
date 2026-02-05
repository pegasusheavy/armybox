//! route - show/manipulate IP routing table
//!
//! Display or modify the IP routing table.

use crate::io;

/// route - show/manipulate IP routing table
///
/// # Synopsis
/// ```text
/// route
/// ```
///
/// # Description
/// Display the kernel IP routing table.
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn route(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Kernel IP routing table\n");
    io::write_str(1, b"Destination     Gateway         Genmask         Flags Metric Ref    Use Iface\n");

    let fd = io::open(b"/proc/net/route", libc::O_RDONLY, 0);
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
    fn test_route_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["route"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Destination") || stdout.contains("routing"));
    }
}
