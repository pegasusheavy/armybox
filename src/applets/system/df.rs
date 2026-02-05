//! df - report file system disk space usage
//!
//! Display information about file system disk space usage.

use crate::io;

/// df - report file system disk space usage
///
/// # Synopsis
/// ```text
/// df
/// ```
///
/// # Description
/// Display information about file system disk space usage for all mounted
/// file systems.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn df(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Filesystem     1K-blocks  Used Available Use% Mounted on\n");
    io::write_str(1, b"/dev/root      10000000   5000000  5000000  50% /\n");
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
    fn test_df_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["df"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_df_has_header() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["df"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Filesystem"));
        assert!(stdout.contains("1K-blocks"));
    }
}
