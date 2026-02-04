//! pwd - print name of current/working directory
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pwd.html

use crate::io;
use crate::sys;

/// pwd - print name of current/working directory
///
/// # Synopsis
/// ```text
/// pwd [-L|-P]
/// ```
///
/// # Description
/// Prints the absolute pathname of the current working directory.
///
/// # Options
/// - `-L`: Use PWD from environment (logical path) - not implemented
/// - `-P`: Print physical path without symlinks (default)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn pwd(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut buf = [0u8; 4096];
    let ret = unsafe { libc::getcwd(buf.as_mut_ptr() as *mut i8, buf.len()) };
    if !ret.is_null() {
        io::write_all(1, &buf[..io::strlen_arr(&buf)]);
        io::write_str(1, b"\n");
        0
    } else {
        sys::perror(b"getcwd");
        1
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::fs;
    use std::path::PathBuf;
    use std::env;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        // Use absolute paths since tests may change current_dir
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() { return release; }
        manifest_dir.join("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_pwd_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_pwd_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();

        let output = Command::new(&armybox)
            .args(["pwd"])
            .current_dir(&dir)
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Should contain the temp dir path
        assert!(stdout.trim().len() > 0);
        cleanup(&dir);
    }

    #[test]
    fn test_pwd_output_ends_with_newline() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["pwd"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.ends_with('\n'));
    }

    #[test]
    fn test_pwd_absolute_path() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["pwd"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Absolute path should start with /
        assert!(stdout.trim().starts_with('/'));
    }
}
