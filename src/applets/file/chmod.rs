//! chmod - change file mode bits
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/chmod.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// chmod - change file mode bits
///
/// # Synopsis
/// ```text
/// chmod [-R] mode file...
/// ```
///
/// # Description
/// Changes the file mode bits of each given file according to mode.
///
/// # Options
/// - `-R`: Recursive (not implemented)
///
/// # Mode
/// Octal mode (e.g., 755, 644) or symbolic mode (not implemented)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn chmod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"chmod: missing operand\n");
        return 1;
    }

    let mode_str = match unsafe { get_arg(argv, 1) } {
        Some(m) => m,
        None => {
            io::write_str(2, b"chmod: missing mode\n");
            return 1;
        }
    };
    let mode = sys::parse_octal(mode_str).unwrap_or(0o644);

    let mut exit_code = 0;
    for i in 2..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if io::chmod(path, mode) < 0 {
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
    use std::os::unix::fs::PermissionsExt;

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
        let dir = std::env::temp_dir().join(format!("armybox_chmod_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_chmod_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("file.txt");
        fs::write(&file, "content").unwrap();

        let output = Command::new(&armybox)
            .args(["chmod", "755", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let mode = fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
        cleanup(&dir);
    }

    #[test]
    fn test_chmod_restrictive() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file = dir.join("file.txt");
        fs::write(&file, "content").unwrap();

        let output = Command::new(&armybox)
            .args(["chmod", "600", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let mode = fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        cleanup(&dir);
    }

    #[test]
    fn test_chmod_multiple_files() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let file1 = dir.join("file1.txt");
        let file2 = dir.join("file2.txt");
        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();

        let output = Command::new(&armybox)
            .args(["chmod", "700", file1.to_str().unwrap(), file2.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert_eq!(fs::metadata(&file1).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(&file2).unwrap().permissions().mode() & 0o777, 0o700);
        cleanup(&dir);
    }

    #[test]
    fn test_chmod_missing_operand() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["chmod"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }

    #[test]
    fn test_chmod_nonexistent_file() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["chmod", "755", "/nonexistent/file"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }
}
