//! insmod - insert a module into the Linux kernel
//!
//! Insert a loadable module into the kernel.

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// insmod - insert a module into the Linux kernel
///
/// # Synopsis
/// ```text
/// insmod module.ko
/// ```
///
/// # Description
/// Insert a loadable module into the kernel. This is a stub implementation.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn insmod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let path = unsafe { get_arg(argv, 1).unwrap() };

    let fd = io::open(path, libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(path);
        return 1;
    }

    // Get file size
    let size = unsafe { libc::lseek(fd, 0, libc::SEEK_END) };
    unsafe { libc::lseek(fd, 0, libc::SEEK_SET) };

    // This would need mmap and init_module syscall
    io::close(fd);
    let _ = size;
    io::write_str(2, b"insmod: not fully implemented\n");
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
    fn test_insmod_no_args() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["insmod"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }

    #[test]
    fn test_insmod_nonexistent() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["insmod", "/nonexistent.ko"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
    }
}
