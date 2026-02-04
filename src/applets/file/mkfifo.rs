//! mkfifo - make FIFOs (named pipes)
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mkfifo.html

use crate::sys;
use crate::applets::get_arg;

/// mkfifo - make FIFOs (named pipes)
///
/// # Synopsis
/// ```text
/// mkfifo file...
/// ```
///
/// # Description
/// Create FIFO special files (named pipes).
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn mkfifo(argc: i32, argv: *const *const u8) -> i32 {
    let mode = 0o644u32;
    let mut exit_code = 0;

    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path.len() > 0 && path[0] != b'-' {
                if unsafe { libc::mkfifo(path.as_ptr() as *const i8, mode) } < 0 {
                    sys::perror(path);
                    exit_code = 1;
                }
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
    use std::os::unix::fs::FileTypeExt;

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

    fn setup() -> PathBuf {
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_mkfifo_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_mkfifo_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let fifo = dir.join("myfifo");

        let output = Command::new(&armybox)
            .args(["mkfifo", fifo.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(fifo.exists());
        let metadata = fs::metadata(&fifo).unwrap();
        assert!(metadata.file_type().is_fifo());
        cleanup(&dir);
    }

    #[test]
    fn test_mkfifo_multiple() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let fifo1 = dir.join("fifo1");
        let fifo2 = dir.join("fifo2");

        let output = Command::new(&armybox)
            .args(["mkfifo", fifo1.to_str().unwrap(), fifo2.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(fs::metadata(&fifo1).unwrap().file_type().is_fifo());
        assert!(fs::metadata(&fifo2).unwrap().file_type().is_fifo());
        cleanup(&dir);
    }

    #[test]
    fn test_mkfifo_exists() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let fifo = dir.join("myfifo");
        // Create the fifo first
        unsafe { libc::mkfifo(fifo.to_str().unwrap().as_ptr() as *const i8, 0o644) };

        let output = Command::new(&armybox)
            .args(["mkfifo", fifo.to_str().unwrap()])
            .output()
            .unwrap();

        // Should fail if fifo already exists
        assert_ne!(output.status.code(), Some(0));
        cleanup(&dir);
    }

    #[test]
    fn test_mkfifo_no_permission() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Try creating in /root which should fail for non-root
        if std::env::var("USER").map(|u| u == "root").unwrap_or(false) {
            return; // Skip if root
        }

        let output = Command::new(&armybox)
            .args(["mkfifo", "/root/testfifo"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }
}
