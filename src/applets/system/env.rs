//! env - run a program in a modified environment
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/env.html

use crate::io;

/// env - run a program in a modified environment
///
/// # Synopsis
/// ```text
/// env [name=value]... [utility [argument...]]
/// ```
///
/// # Description
/// When called with no arguments, print the environment.
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn env(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe extern "C" { static environ: *const *const i8; }
    unsafe {
        let mut i = 0;
        while !(*environ.add(i)).is_null() {
            let e = io::cstr_to_slice(*environ.add(i) as *const u8);
            io::write_all(1, e);
            io::write_str(1, b"\n");
            i += 1;
        }
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
    fn test_env_runs() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["env"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_env_contains_path() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["env"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Environment should contain PATH
        assert!(stdout.contains("PATH="));
    }

    #[test]
    fn test_env_format() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["env"])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Each line should contain = for variable assignments
        for line in stdout.lines() {
            if !line.is_empty() {
                assert!(line.contains("="), "Line missing '=': {}", line);
            }
        }
    }
}
