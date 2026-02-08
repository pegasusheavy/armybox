//! POSIX.1-2017 compliance tests for dirname
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/dirname.html

use crate::posix::helpers::*;

/// POSIX: "The dirname utility shall write to standard output the
/// directory component of a pathname."
#[test]
fn posix_dirname_basic() {
    let result = run(&["dirname", "/usr/bin/sort"]);
    assert_success(&result);
    assert_stdout(&result, "/usr/bin\n");
}

/// POSIX: dirname of "/" is "/"
#[test]
fn posix_dirname_root() {
    let result = run(&["dirname", "/"]);
    assert_success(&result);
    assert_stdout(&result, "/\n");
}

/// POSIX: dirname with no slashes returns "."
#[test]
fn posix_dirname_no_slash() {
    let result = run(&["dirname", "filename"]);
    assert_success(&result);
    assert_stdout(&result, ".\n");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_dirname_exit_success() {
    let result = run(&["dirname", "/path/to/file"]);
    assert_eq!(result.0, 0);
}
