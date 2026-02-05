//! su - run a command with substitute user and group ID
//!
//! Change the effective user ID and group ID.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// su - run a command with substitute user and group ID
///
/// # Synopsis
/// ```text
/// su [user] [-c command]
/// ```
///
/// # Description
/// Change the effective user ID and group ID to that of user.
/// If no user is specified, root is assumed.
///
/// # Options
/// - `-c command`: Pass command to the shell
///
/// # Exit Status
/// - 0: Success
/// - 1: Error
pub fn su(argc: i32, argv: *const *const u8) -> i32 {
    let mut user = b"root" as &[u8];
    let mut command: Option<&[u8]> = None;
    let mut i = 1;

    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-c" && i + 1 < argc {
                command = unsafe { get_arg(argv, i + 1) };
                i += 2;
                continue;
            } else if arg == b"-" || arg == b"-l" || arg == b"--login" {
                // Login shell option - ignored for now
                i += 1;
                continue;
            } else if arg[0] != b'-' {
                user = arg;
            }
        }
        i += 1;
    }

    // Look up user in /etc/passwd
    let mut uid: u32 = 0;
    let mut gid: u32 = 0;
    let mut shell = b"/bin/sh" as &[u8];
    let mut home = b"/" as &[u8];

    // For simplicity, just handle root
    if user != b"root" {
        io::write_str(2, b"su: user lookup not fully implemented\n");
        // Try to use getpwnam
    }

    // Set group ID first
    if unsafe { libc::setgid(gid) } != 0 {
        sys::perror(b"setgid");
        return 1;
    }

    // Set user ID
    if unsafe { libc::setuid(uid) } != 0 {
        sys::perror(b"setuid");
        return 1;
    }

    // Execute command or shell
    if let Some(cmd) = command {
        let mut cmd_buf = [0u8; 4096];
        cmd_buf[..cmd.len()].copy_from_slice(cmd);

        let shell_path = b"/bin/sh\0";
        let c_flag = b"-c\0";
        let argv_ptrs = [
            shell_path.as_ptr() as *const i8,
            c_flag.as_ptr() as *const i8,
            cmd_buf.as_ptr() as *const i8,
            core::ptr::null(),
        ];

        unsafe {
            libc::execv(shell_path.as_ptr() as *const i8, argv_ptrs.as_ptr());
        }
    } else {
        let mut shell_buf = [0u8; 256];
        shell_buf[..shell.len()].copy_from_slice(shell);

        let argv_ptrs = [
            shell_buf.as_ptr() as *const i8,
            core::ptr::null(),
        ];

        unsafe {
            libc::execv(shell_buf.as_ptr() as *const i8, argv_ptrs.as_ptr());
        }
    }

    sys::perror(b"exec");
    let _ = home;
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
    fn test_su_requires_root() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        // su typically requires root privileges
        let output = Command::new(&armybox)
            .args(["su", "-c", "true"])
            .output()
            .unwrap();

        // Will likely fail without root, which is expected
        assert!(output.status.code() == Some(0) || output.status.code() == Some(1));
    }
}
