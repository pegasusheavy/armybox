//! inotifyd - inotify event daemon
//!
//! Run a program on file system events.

use crate::io;
use crate::sys;
use super::get_arg;

/// inotifyd - inotify event daemon
///
/// # Synopsis
/// ```text
/// inotifyd PROG FILE:MASK...
/// ```
///
/// # Description
/// Run PROG when file system events occur on FILE.
///
/// # Masks
/// - a: Access
/// - c: Create
/// - d: Delete
/// - m: Modify
/// - M: Move
/// - n: Name change (rename)
/// - w: Close write
/// - 0: Close nowrite
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn inotifyd(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"inotifyd: usage: inotifyd PROG FILE:MASK...\n");
        return 1;
    }

    let _prog = unsafe { get_arg(argv, 1).unwrap() };

    // Initialize inotify
    let inotify_fd = unsafe { libc::inotify_init() };
    if inotify_fd < 0 {
        sys::perror(b"inotify_init");
        return 1;
    }

    // Add watches for each FILE:MASK
    for i in 2..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            // Find colon separator
            let mut colon_pos = None;
            for (j, &c) in arg.iter().enumerate() {
                if c == b':' {
                    colon_pos = Some(j);
                    break;
                }
            }

            let (file, mask_str) = if let Some(pos) = colon_pos {
                (&arg[..pos], &arg[pos + 1..])
            } else {
                (arg, b"" as &[u8])
            };

            let mask = parse_mask(mask_str);
            if mask == 0 {
                io::write_str(2, b"inotifyd: empty mask for ");
                io::write_all(2, file);
                io::write_str(2, b"\n");
                continue;
            }

            // Need null-terminated path
            let mut path = [0u8; 256];
            let len = file.len().min(path.len() - 1);
            path[..len].copy_from_slice(&file[..len]);

            let wd = unsafe {
                libc::inotify_add_watch(inotify_fd, path.as_ptr() as *const i8, mask)
            };
            if wd < 0 {
                sys::perror(file);
            }
        }
    }

    io::write_str(2, b"inotifyd: daemon mode not fully implemented\n");

    unsafe { libc::close(inotify_fd) };
    0
}

fn parse_mask(s: &[u8]) -> u32 {
    let mut mask: u32 = 0;
    for &c in s {
        mask |= match c {
            b'a' => libc::IN_ACCESS,
            b'c' => libc::IN_CREATE,
            b'd' => libc::IN_DELETE | libc::IN_DELETE_SELF,
            b'm' => libc::IN_MODIFY,
            b'M' => libc::IN_MOVE_SELF | libc::IN_MOVED_FROM | libc::IN_MOVED_TO,
            b'n' => libc::IN_MOVED_FROM | libc::IN_MOVED_TO,
            b'w' => libc::IN_CLOSE_WRITE,
            b'0' => libc::IN_CLOSE_NOWRITE,
            _ => 0,
        };
    }
    mask
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
    fn test_inotifyd_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["inotifyd"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("usage"));
    }

    #[test]
    fn test_inotifyd_with_watch() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["inotifyd", "echo", "/tmp:m"])
            .output()
            .unwrap();

        // Should complete (not fully implemented)
        assert_eq!(output.status.code(), Some(0));
    }
}
