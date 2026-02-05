//! killall5 - send a signal to all processes
//!
//! SystemV killall command - send a signal to all processes except init.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// killall5 - send a signal to all processes
///
/// # Synopsis
/// ```text
/// killall5 [-SIGNAL]
/// ```
///
/// # Description
/// Send a signal to all processes except init (PID 1) and the calling process.
/// This is typically used during system shutdown.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn killall5(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = libc::SIGTERM;

    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            if arg[0] == b'-' && arg.len() > 1 {
                signal = sys::parse_i64(&arg[1..]).unwrap_or(libc::SIGTERM as i64) as i32;
            }
        }
    }

    let my_pid = unsafe { libc::getpid() };

    // Send signal to all processes except init and ourselves
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
                if let Some(pid) = sys::parse_i64(name) {
                    if pid > 1 && pid as i32 != my_pid {
                        let _ = unsafe { libc::kill(pid as i32, signal) };
                    }
                }
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
    fn test_killall5_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Test with signal 0 (check permissions only, no actual signal)
        let output = Command::new(&armybox)
            .args(["killall5", "-0"])
            .output()
            .unwrap();

        // Should exit 0 regardless of whether it could signal all processes
        assert_eq!(output.status.code(), Some(0));
    }
}
