#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Arbitrary, Debug)]
struct DdArgs {
    bs: u16,
    count: u16,
    skip: u16,
    seek: u16,
    input_data: Vec<u8>,
}

fuzz_target!(|args: DdArgs| {
    if args.input_data.is_empty() || args.input_data.len() > 10000 {
        return;
    }

    // Create input file
    let mut input_file = tempfile::NamedTempFile::new().unwrap();
    input_file.write_all(&args.input_data).unwrap();
    let input_path = input_file.path().to_str().unwrap();

    // Create output file
    let output_file = tempfile::NamedTempFile::new().unwrap();
    let output_path = output_file.path().to_str().unwrap();

    // Build dd arguments
    let bs = (args.bs as u64 % 8192 + 1).to_string();
    let count = (args.count as u64 % 100).to_string();
    let skip = (args.skip as u64 % 10).to_string();
    let seek = (args.seek as u64 % 10).to_string();

    // Run dd - testing it handles various argument combinations
    let _ = Command::new("./target/release/armybox")
        .args([
            "dd",
            &format!("if={}", input_path),
            &format!("of={}", output_path),
            &format!("bs={}", bs),
            &format!("count={}", count),
            &format!("skip={}", skip),
            &format!("seek={}", seek),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
});
