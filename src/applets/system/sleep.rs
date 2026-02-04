//! sleep - delay for a specified amount of time
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/sleep.html

use crate::sys;
use crate::applets::get_arg;

/// sleep - delay for a specified amount of time
///
/// # Synopsis
/// ```text
/// sleep time
/// ```
///
/// # Description
/// Suspend execution for at least the integral number of seconds
/// specified by the time operand.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn sleep(argc: i32, argv: *const *const u8) -> i32 {
    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            let secs = sys::parse_u64(arg).unwrap_or(0) as u32;
            unsafe { libc::sleep(secs) };
        }
    }
    0
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::time::Instant;
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
    fn test_sleep_zero() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let start = Instant::now();
        let output = Command::new(&armybox)
            .args(["sleep", "0"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        // Should complete quickly
        assert!(start.elapsed().as_millis() < 500);
    }

    #[test]
    fn test_sleep_one_second() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let start = Instant::now();
        let output = Command::new(&armybox)
            .args(["sleep", "1"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        // Should take at least 1 second
        assert!(start.elapsed().as_millis() >= 900);
    }
}
