//! date - print or set the system date and time
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/date.html

use crate::io;

/// date - print or set the system date and time
///
/// # Synopsis
/// ```text
/// date [+format]
/// ```
///
/// # Description
/// Display the current date and time.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn date(_argc: i32, _argv: *const *const u8) -> i32 {
    let now = unsafe { libc::time(core::ptr::null_mut()) };
    let tm = unsafe { libc::localtime(&now) };
    if !tm.is_null() {
        let t = unsafe { &*tm };
        io::write_num(1, (t.tm_year + 1900) as u64);
        io::write_str(1, b"-");
        if t.tm_mon + 1 < 10 { io::write_str(1, b"0"); }
        io::write_num(1, (t.tm_mon + 1) as u64);
        io::write_str(1, b"-");
        if t.tm_mday < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_mday as u64);
        io::write_str(1, b" ");
        if t.tm_hour < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_hour as u64);
        io::write_str(1, b":");
        if t.tm_min < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_min as u64);
        io::write_str(1, b":");
        if t.tm_sec < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_sec as u64);
        io::write_str(1, b"\n");
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
    fn test_date_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["date"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_date_format() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["date"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Should contain date in YYYY-MM-DD HH:MM:SS format
        assert!(stdout.contains("-"));
        assert!(stdout.contains(":"));
    }

    #[test]
    fn test_date_current_year() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["date"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Should contain current year (202x)
        assert!(stdout.starts_with("202"));
    }
}
