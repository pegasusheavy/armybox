//! rfkill - tool for enabling and disabling wireless devices
//!
//! Query and change the state of rfkill switches.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// rfkill - tool for enabling and disabling wireless devices
///
/// # Synopsis
/// ```text
/// rfkill [list|block|unblock] [type]
/// ```
///
/// # Description
/// Tool for enabling and disabling wireless devices.
///
/// # Commands
/// - list: List the current state of rfkill switches
/// - block: Disable a type of wireless device
/// - unblock: Enable a type of wireless device
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn rfkill(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        // Default to list
        return rfkill_list();
    }

    let cmd = unsafe { get_arg(argv, 1).unwrap() };

    if cmd == b"list" {
        rfkill_list()
    } else if cmd == b"block" || cmd == b"unblock" {
        if argc < 3 {
            io::write_str(2, b"rfkill: missing type\n");
            return 1;
        }
        let type_arg = unsafe { get_arg(argv, 2).unwrap() };
        let block = cmd == b"block";
        rfkill_set(type_arg, block)
    } else {
        io::write_str(2, b"rfkill: unknown command\n");
        1
    }
}

fn rfkill_list() -> i32 {
    // Read from /sys/class/rfkill
    let dir = io::opendir(b"/sys/class/rfkill");
    if dir.is_null() {
        io::write_str(1, b"No rfkill devices found\n");
        return 0;
    }

    io::write_str(1, b"ID TYPE      DEVICE     SOFT      HARD\n");

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let d_name = unsafe { &(*entry).d_name };
        let name_len = d_name.iter().position(|&c| c == 0).unwrap_or(d_name.len());
        let name = &d_name[..name_len];
        let name_u8: &[u8] = unsafe { core::mem::transmute(name) };

        // Skip . and ..
        if name_u8 == b"." || name_u8 == b".." {
            continue;
        }

        // Read type
        let mut type_path = [0u8; 256];
        let mut pi = 0;
        for &c in b"/sys/class/rfkill/" {
            type_path[pi] = c;
            pi += 1;
        }
        for &c in name_u8 {
            type_path[pi] = c;
            pi += 1;
        }
        for &c in b"/type" {
            type_path[pi] = c;
            pi += 1;
        }

        let type_fd = io::open(&type_path[..pi], libc::O_RDONLY, 0);
        let mut type_buf = [0u8; 32];
        let type_str = if type_fd >= 0 {
            let n = io::read(type_fd, &mut type_buf);
            io::close(type_fd);
            let len = if n > 0 && type_buf[n as usize - 1] == b'\n' {
                n as usize - 1
            } else {
                n as usize
            };
            &type_buf[..len]
        } else {
            b"unknown" as &[u8]
        };

        // Print entry
        io::write_all(1, name_u8);
        io::write_str(1, b" ");
        io::write_all(1, type_str);
        io::write_str(1, b"\n");
    }

    io::closedir(dir);
    0
}

fn rfkill_set(type_arg: &[u8], block: bool) -> i32 {
    let _ = type_arg;
    let _ = block;
    io::write_str(2, b"rfkill: block/unblock not fully implemented\n");
    1
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
    fn test_rfkill_list() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["rfkill", "list"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_rfkill_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["rfkill"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }
}
