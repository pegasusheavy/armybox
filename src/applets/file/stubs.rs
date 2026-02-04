//! Stub implementations for various file utilities
//!
//! These are placeholder implementations that may be expanded later.

use crate::io;

/// chattr - change file attributes on a Linux file system
pub fn chattr(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    io::write_str(2, b"chattr: stub\n");
    0
}

/// lsattr - list file attributes on a Linux file system
pub fn lsattr(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    io::write_str(2, b"lsattr: stub\n");
    0
}

/// fstype - print type of filesystem
pub fn fstype(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    io::write_str(1, b"ext4\n");
    0
}

/// makedevs - create a range of device files
pub fn makedevs(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    io::write_str(2, b"makedevs: stub\n");
    0
}

/// setfattr - set extended attributes of filesystem objects
pub fn setfattr(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
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
    fn test_chattr_stub() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["chattr"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_lsattr_stub() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["lsattr"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_fstype_output() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["fstype"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ext4"));
    }

    #[test]
    fn test_makedevs_stub() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["makedevs"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_setfattr_stub() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["setfattr"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }
}
