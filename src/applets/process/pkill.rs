//! pkill - signal processes based on name
//!
//! Send a signal to processes matching a pattern.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// pkill - signal processes based on name
///
/// # Synopsis
/// ```text
/// pkill [-SIGNAL] pattern
/// ```
///
/// # Description
/// Send a signal to all processes whose names match the given pattern.
///
/// # Exit Status
/// - 0: At least one process was signaled
/// - 1: No processes matched or an error occurred
pub fn pkill(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }

    let mut signal = libc::SIGTERM;
    let mut pattern_idx = 1;

    if let Some(arg) = unsafe { get_arg(argv, 1) } {
        if arg[0] == b'-' && arg.len() > 1 {
            signal = sys::parse_i64(&arg[1..]).unwrap_or(libc::SIGTERM as i64) as i32;
            pattern_idx = 2;
        }
    }

    if pattern_idx >= argc { return 1; }
    let pattern = unsafe { get_arg(argv, pattern_idx).unwrap() };
    let mut killed = 0;

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
                let mut path = [0u8; 64];
                let mut pi = 0;
                for &c in b"/proc/" { path[pi] = c; pi += 1; }
                for &c in name { path[pi] = c; pi += 1; }
                for &c in b"/comm\0" { path[pi] = c; pi += 1; }

                let comm_fd = io::open(&path, libc::O_RDONLY, 0);
                if comm_fd >= 0 {
                    let mut comm_buf = [0u8; 256];
                    let cn = io::read(comm_fd, &mut comm_buf);
                    io::close(comm_fd);

                    if cn > 0 {
                        let comm = &comm_buf[..cn as usize];
                        let comm = comm.split(|&c| c == b'\n').next().unwrap_or(comm);
                        if comm.windows(pattern.len()).any(|w| w == pattern) {
                            if let Some(pid) = sys::parse_i64(name) {
                                if unsafe { libc::kill(pid as i32, signal) } == 0 {
                                    killed += 1;
                                }
                            }
                        }
                    }
                }
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
    if killed > 0 { 0 } else { 1 }
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
    fn test_pkill_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["pkill"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }

    #[test]
    fn test_pkill_nonexistent() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["pkill", "nonexistent_process_12345"])
            .output()
            .unwrap();

        // No such process, so exit 1
        assert_eq!(output.status.code(), Some(1));
    }

    #[test]
    fn test_pkill_with_signal_0() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Signal 0 just checks if we can signal, doesn't actually send
        let output = Command::new(&armybox)
            .args(["pkill", "-0", "nonexistent_process_12345"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }
}
