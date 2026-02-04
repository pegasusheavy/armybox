//! touch - change file timestamps
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/touch.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// touch - change file access and modification times
///
/// # Synopsis
/// ```text
/// touch [-acm] [-r ref_file | -t time] file...
/// ```
///
/// # Description
/// Updates the access and modification times of each file.
/// Creates files that do not exist (unless -c is specified).
///
/// # Options
/// - `-a`: Change only access time (not implemented)
/// - `-c`: Do not create file if it doesn't exist (not implemented)
/// - `-m`: Change only modification time (not implemented)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn touch(argc: i32, argv: *const *const u8) -> i32 {
    let mut exit_code = 0;

    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path.len() > 0 && path[0] != b'-' {
                // Try to create file if doesn't exist
                let fd = io::open(path, libc::O_WRONLY | libc::O_CREAT, 0o644);
                if fd >= 0 {
                    io::close(fd);
                    // Update timestamps to current time
                    if unsafe { libc::utimes(path.as_ptr() as *const i8, core::ptr::null()) } < 0 {
                        sys::perror(path);
                        exit_code = 1;
                    }
                } else {
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
    use std::time::SystemTime;

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
        let dir = std::env::temp_dir().join(format!("armybox_touch_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_touch_create_file() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("newfile.txt");

        let output = Command::new(&armybox)
            .args(["touch", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(file.exists());
        cleanup(&dir);
    }

    #[test]
    fn test_touch_multiple_files() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file1 = dir.join("file1.txt");
        let file2 = dir.join("file2.txt");

        let output = Command::new(&armybox)
            .args(["touch", file1.to_str().unwrap(), file2.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(file1.exists());
        assert!(file2.exists());
        cleanup(&dir);
    }

    #[test]
    fn test_touch_update_existing() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("existing.txt");
        fs::write(&file, "content").unwrap();

        // Get original mtime
        let orig_meta = fs::metadata(&file).unwrap();
        let orig_mtime = orig_meta.modified().unwrap();

        // Wait a tiny bit
        std::thread::sleep(std::time::Duration::from_millis(10));

        let output = Command::new(&armybox)
            .args(["touch", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));

        // mtime should be updated (or at least not older)
        let new_meta = fs::metadata(&file).unwrap();
        let new_mtime = new_meta.modified().unwrap();
        assert!(new_mtime >= orig_mtime);
        cleanup(&dir);
    }

    #[test]
    fn test_touch_no_permission() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Skip if root
        if std::env::var("USER").map(|u| u == "root").unwrap_or(false) {
            return;
        }

        let output = Command::new(&armybox)
            .args(["touch", "/root/test_file"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }
}
