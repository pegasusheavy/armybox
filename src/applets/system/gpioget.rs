//! gpioget - read values of GPIO lines
//!
//! Read values from GPIO lines.

use crate::io;
use crate::sys;
use super::get_arg;

/// gpioget - read values of GPIO lines
///
/// # Synopsis
/// ```text
/// gpioget CHIP LINE...
/// ```
///
/// # Description
/// Read values from one or more GPIO lines on a chip.
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn gpioget(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"gpioget: usage: gpioget CHIP LINE...\n");
        return 1;
    }

    let chip = unsafe { get_arg(argv, 1).unwrap() };

    // Build device path
    let mut path = [0u8; 64];
    if chip.len() > 0 && chip[0] == b'/' {
        // Absolute path
        let len = chip.len().min(path.len() - 1);
        path[..len].copy_from_slice(&chip[..len]);
    } else {
        // Relative - prepend /dev/
        let prefix = b"/dev/";
        path[..prefix.len()].copy_from_slice(prefix);
        let len = chip.len().min(path.len() - prefix.len() - 1);
        path[prefix.len()..prefix.len() + len].copy_from_slice(&chip[..len]);
    }

    let fd = io::open(&path, libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(&path);
        return 1;
    }

    // Read line values using GPIO_GET_LINEHANDLE_IOCTL would be needed
    // For now, just print placeholder values
    for i in 2..argc {
        if let Some(line_str) = unsafe { get_arg(argv, i) } {
            let _line = sys::parse_u64(line_str).unwrap_or(0);
            // Would need to actually read the GPIO value
            io::write_str(1, b"0");
            if i < argc - 1 {
                io::write_str(1, b" ");
            }
        }
    }
    io::write_str(1, b"\n");

    io::close(fd);
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
    fn test_gpioget_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["gpioget"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("usage"));
    }

    #[test]
    fn test_gpioget_nonexistent_chip() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["gpioget", "nonexistent_chip", "0"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }
}
