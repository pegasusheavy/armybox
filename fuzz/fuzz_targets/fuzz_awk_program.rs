#![no_main]

use libfuzzer_sys::fuzz_target;
use std::process::{Command, Stdio};
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }

    // Split: first part is program, second is input data
    let split_point = data[0] as usize % (data.len() - 1) + 1;

    let program = match std::str::from_utf8(&data[1..split_point]) {
        Ok(s) => s.to_string(),
        Err(_) => return,
    };

    // Skip very long programs
    if program.len() > 200 {
        return;
    }

    let input_data = &data[split_point..];

    // Create temp file with input data
    let mut temp = tempfile::NamedTempFile::new().unwrap();
    temp.write_all(input_data).unwrap();
    let temp_path = temp.path().to_str().unwrap();

    // Run awk with the program - testing it doesn't crash
    let _ = Command::new("./target/release/armybox")
        .args(["awk", &program, temp_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
});
