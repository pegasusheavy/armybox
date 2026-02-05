//! ps - report process status
//!
//! Display information about running processes.

use crate::io;

/// ps - report process status
///
/// # Synopsis
/// ```text
/// ps
/// ```
///
/// # Description
/// Display information about running processes. This is a minimal
/// implementation that lists PIDs from /proc.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn ps(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"  PID TTY          TIME CMD\n");
    let fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return 1; }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };
            if !name.is_empty() && name[0] >= b'0' && name[0] <= b'9' {
                io::write_str(1, b"  ");
                io::write_all(1, name);
                io::write_str(1, b" ?\n");
            }
            offset += dirent.d_reclen as usize;
        }
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
    fn test_ps_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["ps"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_ps_has_header() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["ps"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("PID"));
        assert!(stdout.contains("TTY"));
    }

    #[test]
    fn test_ps_lists_processes() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["ps"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Should have at least the header plus one process
        let lines: Vec<&str> = stdout.lines().collect();
        assert!(lines.len() > 1);
    }
}
