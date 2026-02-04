//! hostname - show or set the system hostname
//!
//! Shows or sets the system's hostname.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// hostname - show or set the system hostname
///
/// # Synopsis
/// ```text
/// hostname [name]
/// ```
///
/// # Description
/// With no arguments, print the current hostname.
/// With an argument, set the hostname (requires root).
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn hostname(argc: i32, argv: *const *const u8) -> i32 {
    if argc > 1 {
        if let Some(name) = unsafe { get_arg(argv, 1) } {
            if unsafe { libc::sethostname(name.as_ptr() as *const i8, name.len()) } < 0 {
                sys::perror(b"sethostname");
                return 1;
            }
        }
    } else {
        let mut buf = [0u8; 256];
        if unsafe { libc::gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) } == 0 {
            io::write_all(1, unsafe { io::cstr_to_slice(buf.as_ptr()) });
            io::write_str(1, b"\n");
        }
    }
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
    fn test_hostname_show() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["hostname"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.trim().is_empty());
    }

    #[test]
    fn test_hostname_matches_system() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // Compare with system hostname command
        let system_output = Command::new("hostname")
            .output()
            .unwrap();
        let system_hostname = std::string::String::from_utf8_lossy(&system_output.stdout);

        let output = Command::new(&armybox)
            .args(["hostname"])
            .output()
            .unwrap();

        let armybox_hostname = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(armybox_hostname.trim(), system_hostname.trim());
    }
}
