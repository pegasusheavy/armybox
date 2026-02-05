//! top - display Linux processes
//!
//! Display a dynamic real-time view of running processes.

use super::ps::ps;

/// top - display Linux processes
///
/// # Synopsis
/// ```text
/// top
/// ```
///
/// # Description
/// Display a dynamic real-time view of running processes.
/// This is a simplified implementation that wraps the ps command.
///
/// # Exit Status
/// - 0: Success
pub fn top(argc: i32, argv: *const *const u8) -> i32 {
    // Simple implementation - just run ps once
    ps(argc, argv)
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
    fn test_top_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["top"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Should contain PID header like ps
        assert!(stdout.contains("PID"));
    }
}
