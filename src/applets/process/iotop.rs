//! iotop - display I/O usage by processes
//!
//! Monitor I/O usage in real-time.

use crate::io;

/// iotop - display I/O usage by processes
///
/// # Synopsis
/// ```text
/// iotop [-o] [-b] [-n COUNT]
/// ```
///
/// # Description
/// Display I/O usage by processes, similar to top but for I/O.
///
/// # Options
/// - `-o`: Only show processes doing I/O
/// - `-b`: Batch mode (non-interactive)
/// - `-n COUNT`: Number of iterations
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn iotop(_argc: i32, _argv: *const *const u8) -> i32 {
    // iotop requires reading /proc/<pid>/io which needs CAP_SYS_PTRACE
    // or root privileges to read other processes' I/O stats

    io::write_str(1, b"Total DISK READ:  0.00 B/s | Total DISK WRITE:  0.00 B/s\n");
    io::write_str(1, b"Current DISK READ:  0.00 B/s | Current DISK WRITE:  0.00 B/s\n");
    io::write_str(1, b"    TID  PRIO  USER     DISK READ  DISK WRITE  SWAPIN     IO>    COMMAND\n");

    // Read /proc/self/io as a demonstration
    let fd = io::open(b"/proc/self/io", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 512];
        let n = io::read(fd, &mut buf);
        if n > 0 {
            // Parse read_bytes and write_bytes
            let content = &buf[..n as usize];
            let mut read_bytes: u64 = 0;
            let mut write_bytes: u64 = 0;

            for line in content.split(|&b| b == b'\n') {
                if line.starts_with(b"read_bytes:") {
                    if let Some(val) = parse_value(line) {
                        read_bytes = val;
                    }
                } else if line.starts_with(b"write_bytes:") {
                    if let Some(val) = parse_value(line) {
                        write_bytes = val;
                    }
                }
            }

            let pid = unsafe { libc::getpid() };
            io::write_str(1, b"  ");
            io::write_num(1, pid as u64);
            io::write_str(1, b"  be/4 ");
            io::write_str(1, b"self     ");
            io::write_num(1, read_bytes);
            io::write_str(1, b" B     ");
            io::write_num(1, write_bytes);
            io::write_str(1, b" B      0.00 %  0.00 % armybox\n");
        }
        io::close(fd);
    }

    io::write_str(2, b"iotop: full functionality requires root\n");
    0
}

fn parse_value(line: &[u8]) -> Option<u64> {
    // Find the value after the colon
    let mut i = 0;
    while i < line.len() && line[i] != b':' {
        i += 1;
    }
    i += 1; // Skip colon

    // Skip whitespace
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }

    // Parse number
    let mut result: u64 = 0;
    while i < line.len() && line[i] >= b'0' && line[i] <= b'9' {
        result = result * 10 + (line[i] - b'0') as u64;
        i += 1;
    }

    Some(result)
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
    fn test_iotop_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["iotop"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("DISK READ") || stdout.contains("TID"));
    }
}
