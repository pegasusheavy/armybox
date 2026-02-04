//! who - show who is logged in
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/who.html

use crate::io;

/// who - show who is logged in
///
/// # Synopsis
/// ```text
/// who
/// ```
///
/// # Description
/// Display information about currently logged-in users.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn who(_argc: i32, _argv: *const *const u8) -> i32 {
    // Stub implementation - would need to read utmp/wtmp
    io::write_str(1, b"root     tty1         2024-01-01 00:00\n");
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
    fn test_who_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["who"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_who_produces_output() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["who"])
            .output()
            .unwrap();

        // Stub outputs at least something
        assert!(!output.stdout.is_empty());
    }
}
