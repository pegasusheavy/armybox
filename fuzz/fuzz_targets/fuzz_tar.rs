#![no_main]

use libfuzzer_sys::fuzz_target;
use std::process::{Command, Stdio};
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    // Fuzz tar extraction - attempt to extract arbitrary data as tar
    // This tests that invalid tar archives don't cause crashes

    if data.len() < 512 || data.len() > 1000000 {
        return;
    }

    // Create a file that looks like it might be a tar archive
    let tar_file = tempfile::NamedTempFile::new().unwrap();
    tar_file.as_file().write_all(data).unwrap();
    let tar_path = tar_file.path().to_str().unwrap();

    // Create extraction directory
    let extract_dir = tempfile::TempDir::new().unwrap();
    let extract_path = extract_dir.path().to_str().unwrap();

    // Try to extract - this should fail gracefully on invalid archives
    let _ = Command::new("./target/release/armybox")
        .args(["tar", "-xf", tar_path, "-C", extract_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Also test tar listing
    let _ = Command::new("./target/release/armybox")
        .args(["tar", "-tf", tar_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
});
