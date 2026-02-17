//! renice - alter priority of running processes
//!
//! Change the scheduling priority of running processes.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// renice - alter priority of running processes
///
/// # Synopsis
/// ```text
/// renice priority pid
/// ```
///
/// # Description
/// Alter the scheduling priority of one or more running processes.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn renice(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"renice: missing operand\n");
        return 1;
    }
    let priority = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(0) as i32;
    let pid = sys::parse_i64(unsafe { get_arg(argv, 2).unwrap() }).unwrap_or(0) as i32;

    if unsafe { libc::setpriority(libc::PRIO_PROCESS, pid as u32, priority) } != 0 {
        sys::perror(b"renice");
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
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
    fn test_renice_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["renice"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("missing operand"));
    }

    #[test]
    fn test_renice_self() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let pid = std::process::id();
        let output = Command::new(&armybox)
            .args(["renice", "0", &pid.to_string()])
            .output()
            .unwrap();

        // Should succeed (can always set own priority to same or lower)
        assert_eq!(output.status.code(), Some(0));
    }
}
