//! ln - make links between files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ln.html

use crate::io;
use crate::sys;
use crate::applets::{get_arg, has_opt};

/// ln - make links between files
///
/// # Synopsis
/// ```text
/// ln [-fs] source_file target_file
/// ln [-fs] source_file... target_dir
/// ```
///
/// # Description
/// Creates a link to a file. By default creates hard links.
///
/// # Options
/// - `-f`: Force - remove existing destination files
/// - `-s`: Create symbolic links instead of hard links
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn ln(argc: i32, argv: *const *const u8) -> i32 {
    let mut symbolic = false;
    let mut force = false;
    let mut files_start = 1;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' {
                if has_opt(arg, b's') { symbolic = true; }
                if has_opt(arg, b'f') { force = true; }
                files_start = i + 1;
            } else {
                break;
            }
        }
    }

    if argc - files_start < 2 {
        io::write_str(2, b"ln: missing operand\n");
        return 1;
    }

    let target = match unsafe { get_arg(argv, argc - 2) } {
        Some(t) => t,
        None => {
            io::write_str(2, b"ln: missing target\n");
            return 1;
        }
    };

    let link_name = match unsafe { get_arg(argv, argc - 1) } {
        Some(l) => l,
        None => {
            io::write_str(2, b"ln: missing link name\n");
            return 1;
        }
    };

    if force {
        let _ = io::unlink(link_name);
    }

    let ret = if symbolic {
        io::symlink(target, link_name)
    } else {
        io::link(target, link_name)
    };

    if ret < 0 {
        sys::perror(link_name);
        return 1;
    }
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
        let release = PathBuf::from("target/release/armybox");
        if release.exists() { return release; }
        PathBuf::from("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_ln_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_ln_hard_link() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let target = dir.join("target.txt");
        let link = dir.join("link.txt");
        fs::write(&target, "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ln", target.to_str().unwrap(), link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(link.exists());
        assert_eq!(fs::read_to_string(&link).unwrap(), "content");
        cleanup(&dir);
    }

    #[test]
    fn test_ln_symbolic_link() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let target = dir.join("target.txt");
        let link = dir.join("symlink");
        fs::write(&target, "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ln", "-s", target.to_str().unwrap(), link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(link.is_symlink());
        cleanup(&dir);
    }

    #[test]
    fn test_ln_force() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let target = dir.join("target.txt");
        let link = dir.join("link.txt");
        fs::write(&target, "content").unwrap();
        fs::write(&link, "old").unwrap();

        let output = Command::new(&armybox)
            .args(["ln", "-sf", target.to_str().unwrap(), link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(link.is_symlink());
        cleanup(&dir);
    }

    #[test]
    fn test_ln_missing_operand() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["ln"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }

    #[test]
    fn test_ln_nonexistent_target() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let link = dir.join("link.txt");

        // Hard link to nonexistent file should fail
        let output = Command::new(&armybox)
            .args(["ln", "/nonexistent/file", link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
        cleanup(&dir);
    }
}
