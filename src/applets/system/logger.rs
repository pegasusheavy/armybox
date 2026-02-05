//! logger - enter messages into the system log
//!
//! Make entries in the system log.

/// logger - enter messages into the system log
///
/// # Synopsis
/// ```text
/// logger [message...]
/// ```
///
/// # Description
/// Make entries in the system log. This is a minimal stub implementation.
///
/// # Exit Status
/// - 0: Success
pub fn logger(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    // Stub implementation - just succeed
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
    fn test_logger_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["logger", "test message"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_logger_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["logger"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }
}
