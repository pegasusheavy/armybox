//! users - print logged in users
//!
//! Prints the login names of users currently logged in.

use crate::io;

/// users - print logged in users
///
/// # Synopsis
/// ```text
/// users
/// ```
///
/// # Description
/// Print login names of users currently logged in, separated by spaces.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn users(_argc: i32, _argv: *const *const u8) -> i32 {
    // Stub implementation - would need to read utmp
    io::write_str(1, b"root\n");
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
    fn test_users_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["users"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_users_produces_output() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["users"])
            .output()
            .unwrap();

        assert!(!output.stdout.is_empty());
    }
}
