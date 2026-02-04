//! POSIX.1-2017 compliance tests
//!
//! Tests verify behavior matches POSIX specifications.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/
//!
//! **Important:** These tests require a pre-built armybox binary.
//! Run: `RUSTFLAGS="-C linker=gcc -C link-arg=-lc" cargo build --release`
//! before running these tests.

use std::process::Command;
use std::path::PathBuf;

/// Helper to get the path to the armybox binary
///
/// Checks ARMYBOX_PATH env var first, then target/release/armybox, then target/debug/armybox.
/// Panics with helpful message if no binary found.
pub fn get_armybox_path() -> PathBuf {
    // Check environment variable first (for CI or custom setups)
    if let Ok(path) = std::env::var("ARMYBOX_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return p;
        }
        panic!("ARMYBOX_PATH set to '{}' but file does not exist", path);
    }

    // Try release first, then debug
    let release = PathBuf::from("target/release/armybox");
    if release.exists() {
        return release;
    }

    let debug = PathBuf::from("target/debug/armybox");
    if debug.exists() {
        return debug;
    }

    panic!(
        "armybox binary not found!\n\
         \n\
         Please build it first with:\n\
         \n\
         RUSTFLAGS=\"-C linker=gcc -C link-arg=-lc\" cargo build --release\n\
         \n\
         Or set ARMYBOX_PATH environment variable to the binary location."
    );
}

/// Helper to run armybox utility and capture output
pub fn run_armybox(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(get_armybox_path())
        .args(args)
        .output()
        .expect("Failed to execute armybox");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    (code, stdout, stderr)
}

/// Helper to create test environment
pub fn setup_test_env() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

mod file;
