//! POSIX.1-2017 compliance tests for dd
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/dd.html

use crate::posix::helpers::*;
use std::fs;

/// POSIX: "The dd utility shall copy the specified input file to the specified output"
#[test]
fn posix_dd_basic() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "hello world").unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
    ]);
    assert_success(&result);
    assert_eq!(fs::read_to_string(&output).unwrap(), "hello world");
}

/// POSIX: dd bs= block size
#[test]
fn posix_dd_block_size() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "x".repeat(1024)).unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
        "bs=512",
    ]);
    assert_success(&result);
    assert_eq!(fs::metadata(&output).unwrap().len(), 1024);
}

/// POSIX: dd count= number of blocks
#[test]
fn posix_dd_count() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "x".repeat(4096)).unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
        "bs=1024",
        "count=2",
    ]);
    assert_success(&result);
    assert_eq!(fs::metadata(&output).unwrap().len(), 2048);
}

/// POSIX: dd skip= skip input blocks
#[test]
fn posix_dd_skip() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "AAAABBBBcccc").unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
        "bs=4",
        "skip=2",
    ]);
    assert_success(&result);
    assert_eq!(fs::read_to_string(&output).unwrap(), "cccc");
}

/// POSIX: dd seek= seek output blocks
#[test]
fn posix_dd_seek() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "DATA").unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
        "bs=4",
        "seek=2",
    ]);
    assert_success(&result);
    // Output should have 8 bytes of zeros followed by DATA
    let data = fs::read(&output).unwrap();
    assert!(data.len() >= 12);
}

/// POSIX: dd from stdin
#[test]
fn posix_dd_stdin() {
    let dir = setup_test_env();
    let output = dir.path().join("output");

    let result = run_with_stdin(
        &["dd", &format!("of={}", output.to_str().unwrap())],
        b"stdin input",
    );
    assert_success(&result);
    assert_eq!(fs::read_to_string(&output).unwrap(), "stdin input");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_dd_exit_success() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "x").unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
    ]);
    assert_eq!(result.0, 0);
}

/// POSIX: dd conv=lcase
#[test]
fn posix_dd_conv_lcase() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "HELLO WORLD").unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
        "conv=lcase",
    ]);
    assert_success(&result);
    assert_eq!(fs::read_to_string(&output).unwrap(), "hello world");
}

/// POSIX: dd conv=ucase
#[test]
fn posix_dd_conv_ucase() {
    let dir = setup_test_env();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    fs::write(&input, "hello world").unwrap();

    let result = run(&[
        "dd",
        &format!("if={}", input.to_str().unwrap()),
        &format!("of={}", output.to_str().unwrap()),
        "conv=ucase",
    ]);
    assert_success(&result);
    assert_eq!(fs::read_to_string(&output).unwrap(), "HELLO WORLD");
}
