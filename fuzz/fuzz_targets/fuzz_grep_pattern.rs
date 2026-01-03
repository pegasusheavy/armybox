#![no_main]

use libfuzzer_sys::fuzz_target;
use std::process::{Command, Stdio};
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    // Convert input to pattern and test data
    if data.len() < 2 {
        return;
    }

    // Split input: first byte determines pattern length, rest is pattern + test data
    let pattern_len = (data[0] as usize % 64).min(data.len() - 1);
    if pattern_len == 0 {
        return;
    }

    let pattern = match std::str::from_utf8(&data[1..1 + pattern_len]) {
        Ok(s) => s.to_string(),
        Err(_) => return, // Skip invalid UTF-8
    };

    // Skip patterns that would cause regex issues (too complex)
    if pattern.len() > 100 {
        return;
    }

    let test_data = &data[1 + pattern_len..];
    if test_data.is_empty() {
        return;
    }

    // Create temp file with test data
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    temp.write_all(test_data).unwrap();
    let temp_path = temp.path().to_str().unwrap();

    // Run grep with the pattern
    // We're testing that grep doesn't crash on arbitrary patterns
    let _ = Command::new("./target/release/armybox")
        .args(["grep", "-E", &pattern, temp_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
});
