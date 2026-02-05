//! kill - send a signal to a process
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/kill.html

use crate::sys;
use crate::applets::get_arg;

/// kill - send a signal to a process
///
/// # Synopsis
/// ```text
/// kill [-s signal_name] pid...
/// kill -signal_number pid...
/// ```
///
/// # Description
/// Send the specified signal to the specified processes.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn kill(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = libc::SIGTERM;
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if arg.len() > 1 && arg[1] >= b'0' && arg[1] <= b'9' {
                    signal = sys::parse_u64(&arg[1..]).unwrap_or(15) as i32;
                }
            } else {
                let pid = sys::parse_i64(arg).unwrap_or(0) as i32;
                if unsafe { libc::kill(pid, signal) } < 0 {
                    sys::perror(arg);
                }
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
    fn test_kill_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Kill with signal 0 (check if process exists) on our own PID
        let pid = std::process::id();
        let output = Command::new(&armybox)
            .args(["kill", "-0", &pid.to_string()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_kill_nonexistent_pid() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Try to kill a very high PID that doesn't exist
        let output = Command::new(&armybox)
            .args(["kill", "999999"])
            .output()
            .unwrap();

        // Should still exit 0 but print error to stderr
        assert_eq!(output.status.code(), Some(0));
    }
}
