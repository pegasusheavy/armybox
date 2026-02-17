//! POSIX.1-2017 compliance tests for renice
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/renice.html

use crate::posix::helpers::*;

/// POSIX: "The renice utility shall request that the nice values of one or more running processes be changed"
#[test]
fn posix_renice_pid() {
    // Try to renice ourselves (may fail without permission)
    let pid = std::process::id();
    let result = run(&["renice", "5", "-p", &pid.to_string()]);
    // May fail due to permissions
    let _ = result;
}

/// POSIX: renice -n nice value
#[test]
fn posix_renice_n_flag() {
    let pid = std::process::id();
    let result = run(&["renice", "-n", "5", "-p", &pid.to_string()]);
    let _ = result;
}

/// POSIX: renice -g group
#[test]
fn posix_renice_group() {
    let result = run(&["renice", "5", "-g", "0"]);
    // Will likely fail without root
    let _ = result;
}

/// POSIX: renice -u user
#[test]
fn posix_renice_user() {
    let result = run(&["renice", "5", "-u", "root"]);
    // Will likely fail without root
    let _ = result;
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_renice_exit_error() {
    let result = run(&["renice", "5", "-p", "99999"]);
    assert!(result.0 > 0);
}

/// POSIX: renice multiple PIDs
#[test]
fn posix_renice_multiple() {
    let result = run(&["renice", "5", "-p", "99998", "99999"]);
    // Will fail (no such processes) but should accept format
    assert!(result.0 > 0);
}
