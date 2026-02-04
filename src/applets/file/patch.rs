//! patch - apply a diff file to an original
//!
//! Simplified stub implementation.

use crate::io;
use crate::applets::{get_arg, has_opt};

/// patch - apply a diff file to an original
///
/// # Synopsis
/// ```text
/// patch [-i patchfile] [file]
/// ```
///
/// # Description
/// Apply a patch file to an original.
///
/// # Options
/// - `-i FILE`: Read patch from FILE
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
///
/// Note: This is a stub implementation.
pub fn patch(argc: i32, argv: *const *const u8) -> i32 {
    let mut input: Option<&[u8]> = None;

    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' {
                if has_opt(arg, b'i') && i + 1 < argc {
                    input = unsafe { get_arg(argv, i + 1) };
                    i += 1;
                }
            }
        }
        i += 1;
    }

    let fd = match input {
        Some(p) => {
            let f = io::open(p, libc::O_RDONLY, 0);
            if f < 0 {
                io::write_str(2, b"patch: can't open patch file\n");
                return 1;
            }
            f
        }
        None => 0,
    };

    io::write_str(2, b"patch: stub implementation\n");

    if fd != 0 { io::close(fd); }
    0
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
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() { return release; }
        manifest_dir.join("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_patch_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_patch_stub() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let patch_file = dir.join("test.patch");
        fs::write(&patch_file, "--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new\n").unwrap();

        let output = Command::new(&armybox)
            .args(["patch", "-i", patch_file.to_str().unwrap()])
            .output()
            .unwrap();

        // Stub implementation always returns 0
        assert_eq!(output.status.code(), Some(0));
        cleanup(&dir);
    }

    #[test]
    fn test_patch_nonexistent() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["patch", "-i", "/nonexistent/file.patch"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }
}
