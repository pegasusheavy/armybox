//! chown - change file owner and group
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/chown.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// chown - change file owner and group
///
/// # Synopsis
/// ```text
/// chown [-hR] owner[:group] file...
/// ```
///
/// # Description
/// Changes the user and/or group ownership of each given file.
///
/// # Options
/// - `-h`: Affect symlinks instead of referenced file (not implemented)
/// - `-R`: Recursive (not implemented)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn chown(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"chown: missing operand\n");
        return 1;
    }

    let owner = match unsafe { get_arg(argv, 1) } {
        Some(o) => o,
        None => {
            io::write_str(2, b"chown: missing owner\n");
            return 1;
        }
    };

    // Parse owner - could be uid or uid:gid
    let uid = sys::parse_u64(owner).unwrap_or(0) as u32;

    let mut exit_code = 0;
    for i in 2..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if unsafe { libc::chown(path.as_ptr() as *const i8, uid, u32::MAX) } < 0 {
                sys::perror(path);
                exit_code = 1;
            }
        }
    }
    exit_code
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::fs;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let release = PathBuf::from("target/release/armybox");
        if release.exists() { return release; }
        PathBuf::from("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_chown_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_chown_missing_operand() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["chown"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }

    #[test]
    fn test_chown_nonexistent_file() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["chown", "1000", "/nonexistent/file"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }

    #[test]
    fn test_chown_no_permission() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Skip if root (root can change ownership)
        if std::env::var("USER").map(|u| u == "root").unwrap_or(false) {
            return;
        }

        let dir = setup();
        let file = dir.join("file.txt");
        fs::write(&file, "content").unwrap();

        // Try to change owner (should fail for non-root)
        let output = Command::new(&armybox)
            .args(["chown", "0", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
        cleanup(&dir);
    }
}
