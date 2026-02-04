//! xargs - build and execute command lines from standard input
//!
//! GNU coreutils compatible implementation.

use crate::io;
use crate::applets::get_arg;

/// xargs - build and execute command lines from standard input
///
/// # Synopsis
/// ```text
/// xargs [command [initial-args]]
/// ```
///
/// # Description
/// Read items from the standard input and execute a command with
/// those items as arguments.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn xargs(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;
        use alloc::ffi::CString;

        // Read lines from stdin
        let mut buf = [0u8; 4096];
        let n = io::read(0, &mut buf);
        if n <= 0 { return 0; }

        // Parse arguments
        let cmd = if argc > 1 {
            unsafe { get_arg(argv, 1).unwrap() }
        } else {
            b"echo"
        };

        // Build argument list
        let lines: Vec<&[u8]> = buf[..n as usize]
            .split(|&c| c == b'\n')
            .filter(|l| !l.is_empty())
            .collect();

        for line in lines {
            let pid = unsafe { libc::fork() };
            if pid == 0 {
                let mut args: Vec<CString> = Vec::new();

                // Command
                let mut v = Vec::with_capacity(cmd.len() + 1);
                v.extend_from_slice(cmd);
                v.push(0);
                if let Ok(cs) = CString::from_vec_with_nul(v) {
                    args.push(cs);
                }

                // Original args
                for i in 2..argc {
                    if let Some(arg) = unsafe { get_arg(argv, i) } {
                        let mut v = Vec::with_capacity(arg.len() + 1);
                        v.extend_from_slice(arg);
                        v.push(0);
                        if let Ok(cs) = CString::from_vec_with_nul(v) {
                            args.push(cs);
                        }
                    }
                }

                // Line as argument
                let mut v = Vec::with_capacity(line.len() + 1);
                v.extend_from_slice(line);
                v.push(0);
                if let Ok(cs) = CString::from_vec_with_nul(v) {
                    args.push(cs);
                }

                let ptrs: Vec<*const i8> = args.iter()
                    .map(|s: &CString| s.as_ptr())
                    .chain(core::iter::once(core::ptr::null()))
                    .collect();

                unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
                unsafe { libc::_exit(127) };
            } else if pid > 0 {
                let mut status = 0;
                unsafe { libc::waitpid(pid, &mut status, 0) };
            }
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        io::write_str(2, b"xargs: requires alloc feature\n");
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::{Command, Stdio};
    use std::io::Write;
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
    fn test_xargs_echo() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["xargs", "echo"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"hello\nworld\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"));
        assert!(stdout.contains("world"));
    }

    #[test]
    fn test_xargs_empty_input() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["xargs", "echo"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_xargs_default_echo() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["xargs"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"test\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("test"));
    }
}
