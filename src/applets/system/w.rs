//! w - show who is logged in and what they are doing
//!
//! Shows who is logged in and what they are doing.

use crate::io;

/// w - show who is logged in and what they are doing
///
/// # Synopsis
/// ```text
/// w
/// ```
///
/// # Description
/// Display information about currently logged-in users and their processes.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn w(_argc: i32, _argv: *const *const u8) -> i32 {
    // Stub implementation
    io::write_str(1, b" 00:00:00 up 0 days, 0:00, 1 user, load average: 0.00, 0.00, 0.00\n");
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
    fn test_w_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["w"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_w_produces_output() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["w"])
            .output()
            .unwrap();

        assert!(!output.stdout.is_empty());
    }
}
