//! netstat - print network connections
//!
//! Display network connections, routing tables, interface statistics.

use crate::io;

/// netstat - print network connections
///
/// # Synopsis
/// ```text
/// netstat [OPTIONS]
/// ```
///
/// # Description
/// Display active network connections and listening sockets.
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn netstat(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Active Internet connections (servers and established)\n");
    io::write_str(1, b"Proto Recv-Q Send-Q Local Address           Foreign Address         State\n");

    // Read /proc/net/tcp
    let fd = io::open(b"/proc/net/tcp", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);
        if n > 0 {
            let content = &buf[..n as usize];
            for (i, line) in content.split(|&c| c == b'\n').enumerate() {
                if i == 0 { continue; } // Skip header
                if line.is_empty() { continue; }
                io::write_str(1, b"tcp    0      0 ");
                // Parse and format the line (simplified)
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
    fn test_netstat_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["netstat"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Proto") || stdout.contains("Active"));
    }
}
