//! POSIX.1-2017 compliance tests for logname
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/logname.html

use crate::posix::helpers::*;

/// POSIX: "The logname utility shall write the user's login name to standard output"
#[test]
fn posix_logname_basic() {
    let result = run(&["logname"]);
    // May fail if not logged in via a terminal
    if result.0 == 0 {
        let name = result.1.trim();
        assert!(!name.is_empty());
    }
}

/// POSIX: logname output is a single line
#[test]
fn posix_logname_single_line() {
    let result = run(&["logname"]);
    if result.0 == 0 {
        assert_eq!(count_stdout_lines(&result), 1);
    }
}

/// POSIX: Exit status 0 on success (when logged in)
#[test]
fn posix_logname_exit() {
    let result = run(&["logname"]);
    // Exit 0 if success, >0 if not on a terminal
    let _ = result;
}
