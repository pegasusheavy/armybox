//! getconf - get configuration values
//!
//! Query system configuration variables.

use crate::io;
use super::get_arg;

/// getconf - get configuration values
///
/// # Synopsis
/// ```text
/// getconf NAME
/// ```
///
/// # Description
/// Get values for system configuration variables.
///
/// # Exit Status
/// - 0: Success
/// - 1: Unknown variable
pub fn getconf(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let name = unsafe { get_arg(argv, 1).unwrap() };

    let val = match name {
        b"PAGE_SIZE" | b"PAGESIZE" => unsafe { libc::sysconf(libc::_SC_PAGESIZE) },
        b"NPROCESSORS_ONLN" => unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) },
        b"NPROCESSORS_CONF" => unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) },
        _ => -1,
    };

    if val >= 0 {
        io::write_num(1, val as u64);
        io::write_str(1, b"\n");
        0
    } else {
        1
    }
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
    fn test_getconf_page_size() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["getconf", "PAGE_SIZE"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let val: u64 = stdout.trim().parse().unwrap_or(0);
        assert!(val >= 4096); // Page size is at least 4KB
    }

    #[test]
    fn test_getconf_nprocessors() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["getconf", "NPROCESSORS_ONLN"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let val: u64 = stdout.trim().parse().unwrap_or(0);
        assert!(val >= 1);
    }

    #[test]
    fn test_getconf_unknown() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["getconf", "UNKNOWN_VAR"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }
}
