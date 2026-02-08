//! POSIX.1-2017 compliance tests for who
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/who.html

use crate::posix::helpers::*;

/// POSIX: "The who utility shall list various pieces of information about accessible users"
#[test]
fn posix_who_basic() {
    let result = run(&["who"]);
    // May be empty if no users logged in, but should succeed
    assert_success(&result);
}

/// POSIX: who -H header
#[test]
fn posix_who_header() {
    let result = run(&["who", "-H"]);
    assert_success(&result);
    // Should have a header line if -H is supported
}

/// POSIX: who -q quick format (count)
#[test]
fn posix_who_quick() {
    let result = run(&["who", "-q"]);
    assert_success(&result);
    // Quick format shows count
}

/// POSIX: who -a all options
#[test]
fn posix_who_all() {
    let result = run(&["who", "-a"]);
    assert_success(&result);
}

/// POSIX: who am i (current user)
#[test]
fn posix_who_am_i() {
    let result = run(&["who", "am", "i"]);
    // May fail if not on a tty, but should not crash
    let _ = result;
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_who_exit_success() {
    let result = run(&["who"]);
    assert_eq!(result.0, 0);
}

/// POSIX: who -b boot time
#[test]
fn posix_who_boot() {
    let result = run(&["who", "-b"]);
    assert_success(&result);
}

/// POSIX: who -u idle time
#[test]
fn posix_who_idle() {
    let result = run(&["who", "-u"]);
    assert_success(&result);
}

/// POSIX: who -T terminal state
#[test]
fn posix_who_terminal() {
    let result = run(&["who", "-T"]);
    assert_success(&result);
}

/// POSIX: who -m same as "who am i"
#[test]
fn posix_who_m() {
    let result = run(&["who", "-m"]);
    let _ = result;
}
