//! lsmod - show the status of modules in the Linux kernel
//!
//! Show the status of loaded kernel modules.

use crate::io;

/// lsmod - show the status of modules in the Linux kernel
///
/// # Synopsis
/// ```text
/// lsmod
/// ```
///
/// # Description
/// Display information about loaded kernel modules by reading /proc/modules.
///
/// # Exit Status
/// - 0: Success
pub fn lsmod(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Module                  Size  Used by\n");
    let fd = io::open(b"/proc/modules", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 { break; }
            io::write_all(1, &buf[..n as usize]);
        }
        io::close(fd);
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
    fn test_lsmod_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["lsmod"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Module"));
    }
}
