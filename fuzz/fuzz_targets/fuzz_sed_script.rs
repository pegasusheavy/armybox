#![no_main]

use libfuzzer_sys::fuzz_target;
use std::process::{Command, Stdio};
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }

    // Extract sed script and input data
    let script_len = (data[0] as usize % 100).min(data.len() - 1);
    if script_len < 3 {
        return;
    }

    let script = match std::str::from_utf8(&data[1..1 + script_len]) {
        Ok(s) => s.to_string(),
        Err(_) => return,
    };

    let input_data = &data[1 + script_len..];
    if input_data.is_empty() {
        return;
    }

    // Create temp file
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    temp.write_all(input_data).unwrap();
    let temp_path = temp.path().to_str().unwrap();

    // Test sed doesn't crash on arbitrary scripts
    let _ = Command::new("./target/release/armybox")
        .args(["sed", &script, temp_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
});
