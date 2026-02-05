//! hwclock - query or set the hardware clock (RTC)
//!
//! Access the hardware real-time clock.

use super::date::date;

/// hwclock - query or set the hardware clock (RTC)
///
/// # Synopsis
/// ```text
/// hwclock
/// ```
///
/// # Description
/// Query or set the hardware clock. This is a simplified implementation
/// that wraps the date command.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn hwclock(argc: i32, argv: *const *const u8) -> i32 {
    // Simple implementation - just show current time like date
    let _ = argc;
    let _ = argv;
    date(argc, argv)
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
    fn test_hwclock_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["hwclock"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        // Should produce some output (date/time)
        assert!(!output.stdout.is_empty());
    }
}
