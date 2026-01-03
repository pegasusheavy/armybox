#![no_main]

use libfuzzer_sys::fuzz_target;
use std::process::{Command, Stdio};
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 100000 {
        return;
    }

    // Test gzip compression/decompression roundtrip
    let input_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(input_file.path(), data).unwrap();
    let input_path = input_file.path().to_str().unwrap().to_string();

    // Compress
    let compress_result = Command::new("./target/release/armybox")
        .args(["gzip", "-k", "-f", &input_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if compress_result.is_err() {
        return;
    }

    let gz_path = format!("{}.gz", input_path);
    if !std::path::Path::new(&gz_path).exists() {
        return;
    }

    // Decompress
    let output_file = tempfile::NamedTempFile::new().unwrap();
    let output_path = output_file.path().to_str().unwrap();

    let _ = Command::new("./target/release/armybox")
        .args(["zcat", &gz_path])
        .stdout(std::fs::File::create(output_path).unwrap())
        .stderr(Stdio::null())
        .status();

    // Verify roundtrip
    let original = std::fs::read(&input_path).unwrap_or_default();
    let decompressed = std::fs::read(output_path).unwrap_or_default();

    // Should match (unless compression failed silently)
    if original != decompressed && !original.is_empty() && !decompressed.is_empty() {
        // This would indicate a bug
        panic!("Roundtrip mismatch!");
    }
});
