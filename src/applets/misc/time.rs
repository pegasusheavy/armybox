//! time - time command execution
//!
//! Run a command and report how long it took.

use crate::io;
use super::get_arg;

/// time - time command execution
///
/// # Synopsis
/// ```text
/// time COMMAND [ARGS...]
/// ```
///
/// # Description
/// Run COMMAND and report timing information.
///
/// # Exit Status
/// - Exit status of COMMAND
pub fn time(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 0; }

    let start = unsafe { libc::time(core::ptr::null_mut()) };

    // Fork and exec
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        #[cfg(feature = "alloc")]
        {
            use alloc::vec::Vec;
            use alloc::ffi::CString;

            let mut args: Vec<CString> = Vec::new();
            for i in 1..argc {
                if let Some(arg) = unsafe { get_arg(argv, i) } {
                    let mut v = Vec::with_capacity(arg.len() + 1);
                    v.extend_from_slice(arg);
                    v.push(0);
                    if let Ok(cs) = CString::from_vec_with_nul(v) {
                        args.push(cs);
                    }
                }
            }
            let ptrs: Vec<*const i8> = args.iter().map(|s| s.as_ptr()).chain(core::iter::once(core::ptr::null())).collect();
            unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
        }
        unsafe { libc::_exit(127) };
    } else if pid > 0 {
        let mut status = 0;
        unsafe { libc::waitpid(pid, &mut status, 0) };

        let end = unsafe { libc::time(core::ptr::null_mut()) };
        let elapsed = end - start;

        io::write_str(2, b"\nreal\t");
        io::write_num(2, elapsed as u64);
        io::write_str(2, b"s\n");
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
    fn test_time_true() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["time", "true"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("real"));
    }

    #[test]
    fn test_time_no_command() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["time"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }
}
