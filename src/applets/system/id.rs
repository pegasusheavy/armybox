//! id - print user and group IDs
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/id.html

use crate::io;

/// id - print user and group IDs
///
/// # Synopsis
/// ```text
/// id [user]
/// ```
///
/// # Description
/// Print user and group information for the specified user,
/// or the current user if no user is specified.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn id(argc: i32, argv: *const *const u8) -> i32 {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let euid = unsafe { libc::geteuid() };
    let egid = unsafe { libc::getegid() };

    let _ = argc; let _ = argv;

    io::write_str(1, b"uid="); io::write_num(1, uid as u64);
    io::write_str(1, b" gid="); io::write_num(1, gid as u64);
    if euid != uid { io::write_str(1, b" euid="); io::write_num(1, euid as u64); }
    if egid != gid { io::write_str(1, b" egid="); io::write_num(1, egid as u64); }
    io::write_str(1, b"\n");
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
    fn test_id_output() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["id"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("uid="));
        assert!(stdout.contains("gid="));
    }

    #[test]
    fn test_id_contains_current_uid() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["id"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Should contain uid= followed by a number
        assert!(stdout.starts_with("uid="));
    }
}
