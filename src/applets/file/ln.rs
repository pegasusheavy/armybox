//! ln - make links between files
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ln.html

use crate::io;
use crate::sys;
use crate::applets::{get_arg, has_opt};

/// ln - make links between files
///
/// # Synopsis
/// ```text
/// ln [-fs] source_file target_file
/// ln [-fs] source_file... target_dir
/// ```
///
/// # Description
/// Creates a link to a file. By default creates hard links. If the final
/// operand names an existing directory, a link is created inside it for
/// each preceding source, named after that source's basename. Otherwise
/// exactly two operands are required: source_file and target_file.
///
/// # Options
/// - `-f`: Force - remove existing destination files (only after the link
///   would otherwise fail because the destination exists)
/// - `-s`: Create symbolic links instead of hard links
/// - `-n`: Do not dereference a destination that is a symbolic link to a
///   directory; treat it as a normal file target instead
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn ln(argc: i32, argv: *const *const u8) -> i32 {
    let mut symbolic = false;
    let mut force = false;
    let mut no_deref = false;
    let mut files_start = 1;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' {
                if has_opt(arg, b's') { symbolic = true; }
                if has_opt(arg, b'f') { force = true; }
                if has_opt(arg, b'n') { no_deref = true; }
                files_start = i + 1;
            } else {
                break;
            }
        }
    }

    let num_operands = argc - files_start;
    if num_operands < 2 {
        io::write_str(2, b"ln: missing operand\n");
        return 1;
    }

    let last = match unsafe { get_arg(argv, argc - 1) } {
        Some(l) => l,
        None => {
            io::write_str(2, b"ln: missing operand\n");
            return 1;
        }
    };

    // Determine whether the last operand is a directory. When -n is given,
    // a symbolic link to a directory is treated as a normal file instead
    // of being dereferenced.
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    let stat_ret = if no_deref { io::lstat(last, &mut st) } else { io::stat(last, &mut st) };
    let last_is_dir = stat_ret == 0 && (st.st_mode & libc::S_IFMT) == libc::S_IFDIR;

    if last_is_dir {
        // ln [-fs] source... target_dir
        let mut exit_code = 0;
        for i in files_start..(argc - 1) {
            if let Some(src) = unsafe { get_arg(argv, i) } {
                let mut dest_path = [0u8; 4096];
                let dest_len = build_dest_path(last, src, &mut dest_path);
                if make_link(src, &dest_path[..dest_len], symbolic, force) != 0 {
                    exit_code = 1;
                }
            }
        }
        exit_code
    } else if num_operands == 2 {
        // ln [-fs] source_file target_file
        let source = match unsafe { get_arg(argv, files_start) } {
            Some(s) => s,
            None => {
                io::write_str(2, b"ln: missing operand\n");
                return 1;
            }
        };
        make_link(source, last, symbolic, force)
    } else {
        io::write_str(2, b"ln: target '");
        io::write_all(2, last);
        io::write_str(2, b"' is not a directory\n");
        1
    }
}

/// Build destination path by appending basename of source to dest directory
fn build_dest_path(dest_dir: &[u8], src: &[u8], buf: &mut [u8]) -> usize {
    let mut len = 0;

    for &c in dest_dir {
        if len < buf.len() - 1 {
            buf[len] = c;
            len += 1;
        }
    }

    if len > 0 && buf[len - 1] != b'/' {
        if len < buf.len() - 1 {
            buf[len] = b'/';
            len += 1;
        }
    }

    let basename_start = src.iter().rposition(|&c| c == b'/').map(|p| p + 1).unwrap_or(0);

    for &c in &src[basename_start..] {
        if len < buf.len() - 1 {
            buf[len] = c;
            len += 1;
        }
    }

    len
}

/// Create a single (hard or symbolic) link, validating before removing any
/// pre-existing destination. The link is attempted first; only if it fails
/// because the destination already exists, and `-f` was given, is the
/// destination removed and the link retried.
fn make_link(source: &[u8], link_name: &[u8], symbolic: bool, force: bool) -> i32 {
    sys::clear_errno();
    let mut ret = if symbolic {
        io::symlink(source, link_name)
    } else {
        io::link(source, link_name)
    };

    if ret < 0 && force && sys::errno() == libc::EEXIST {
        let _ = io::unlink(link_name);
        sys::clear_errno();
        ret = if symbolic {
            io::symlink(source, link_name)
        } else {
            io::link(source, link_name)
        };
    }

    if ret < 0 {
        sys::perror(link_name);
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    use std::process::Command;
    use std::fs;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let release = PathBuf::from("target/release/armybox");
        if release.exists() { return release; }
        PathBuf::from("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("armybox_ln_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_ln_hard_link() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let target = dir.join("target.txt");
        let link = dir.join("link.txt");
        fs::write(&target, "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ln", target.to_str().unwrap(), link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(link.exists());
        assert_eq!(fs::read_to_string(&link).unwrap(), "content");
        cleanup(&dir);
    }

    #[test]
    fn test_ln_symbolic_link() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let target = dir.join("target.txt");
        let link = dir.join("symlink");
        fs::write(&target, "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ln", "-s", target.to_str().unwrap(), link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(link.is_symlink());
        cleanup(&dir);
    }

    #[test]
    fn test_ln_force() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let target = dir.join("target.txt");
        let link = dir.join("link.txt");
        fs::write(&target, "content").unwrap();
        fs::write(&link, "old").unwrap();

        let output = Command::new(&armybox)
            .args(["ln", "-sf", target.to_str().unwrap(), link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert!(link.is_symlink());
        cleanup(&dir);
    }

    #[test]
    fn test_ln_missing_operand() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["ln"])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
    }

    #[test]
    fn test_ln_nonexistent_target() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        let link = dir.join("link.txt");

        // Hard link to nonexistent file should fail
        let output = Command::new(&armybox)
            .args(["ln", "/nonexistent/file", link.to_str().unwrap()])
            .output()
            .unwrap();

        assert_ne!(output.status.code(), Some(0));
        cleanup(&dir);
    }
}
