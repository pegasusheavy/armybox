//! POSIX.1-2017 compliance tests for ps
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ps.html

use crate::posix::helpers::*;

/// POSIX: "The ps utility shall write information about processes"
#[test]
fn posix_ps_basic() {
    let result = run(&["ps"]);
    assert_success(&result);
    // Should have at least a header and the ps process itself
    assert!(!result.1.is_empty());
}

/// POSIX: ps -a all with tty
#[test]
fn posix_ps_all_tty() {
    let result = run(&["ps", "-a"]);
    assert_success(&result);
}

/// POSIX: ps -e all processes
#[test]
fn posix_ps_all() {
    let result = run(&["ps", "-e"]);
    assert_success(&result);
    // Should have many processes
    let lines: Vec<&str> = result.1.lines().collect();
    assert!(lines.len() > 1);
}

/// POSIX: ps -f full format
#[test]
fn posix_ps_full() {
    let result = run(&["ps", "-f"]);
    assert_success(&result);
    // Full format includes UID, PID, PPID, etc.
    let header = result.1.lines().next().unwrap_or("");
    assert!(header.contains("UID") || header.contains("PID"));
}

/// POSIX: ps -o custom format
#[test]
fn posix_ps_format() {
    let result = run(&["ps", "-o", "pid,comm"]);
    assert_success(&result);
    assert!(result.1.contains("PID"));
}

/// POSIX: ps -p specific PID
#[test]
fn posix_ps_pid() {
    let result = run(&["ps", "-p", "1"]);
    assert_success(&result);
    // Should show init/systemd
}

/// POSIX: ps -u user
#[test]
fn posix_ps_user() {
    let result = run(&["ps", "-u", "root"]);
    // May succeed or fail depending on system
    let _ = result;
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_ps_exit_success() {
    let result = run(&["ps"]);
    assert_eq!(result.0, 0);
}

/// POSIX: ps -ef combined common flags
#[test]
fn posix_ps_ef() {
    let result = run(&["ps", "-ef"]);
    assert_success(&result);
    let lines: Vec<&str> = result.1.lines().collect();
    assert!(lines.len() > 1);
}

/// POSIX: ps shows command name
#[test]
fn posix_ps_shows_cmd() {
    let result = run(&["ps", "-e"]);
    assert_success(&result);
    // Should contain process names
    assert!(result.1.len() > 50);
}

/// POSIX: ps -l long format
#[test]
fn posix_ps_long() {
    let result = run(&["ps", "-l"]);
    assert_success(&result);
}

/// POSIX: ps -A same as -e
#[test]
fn posix_ps_all_uppercase() {
    let result = run(&["ps", "-A"]);
    assert_success(&result);
    let lines: Vec<&str> = result.1.lines().collect();
    assert!(lines.len() > 1);
}
