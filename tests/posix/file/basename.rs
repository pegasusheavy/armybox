//! POSIX.1-2017 compliance tests for basename
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/basename.html

use crate::posix::helpers::*;

/// POSIX: "The basename utility shall write a string to standard output
/// that is the final component of a pathname."
#[test]
fn posix_basename_basic() {
    let result = run(&["basename", "/usr/bin/sort"]);
    assert_success(&result);
    assert_stdout(&result, "sort\n");
}

/// POSIX: basename with suffix removal
#[test]
fn posix_basename_suffix() {
    let result = run(&["basename", "/usr/include/stdio.h", ".h"]);
    assert_success(&result);
    assert_stdout(&result, "stdio\n");
}

/// POSIX: basename with trailing slashes stripped
#[test]
fn posix_basename_trailing_slash() {
    let result = run(&["basename", "/foo/bar/"]);
    assert_success(&result);
    assert_stdout(&result, "bar\n");
}

/// POSIX: basename "/" returns "/"
#[test]
fn posix_basename_root() {
    let result = run(&["basename", "/"]);
    assert_success(&result);
    assert_stdout(&result, "/\n");
}

/// POSIX: basename with no slashes returns string as-is
#[test]
fn posix_basename_no_slash() {
    let result = run(&["basename", "filename"]);
    assert_success(&result);
    assert_stdout(&result, "filename\n");
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_basename_exit_success() {
    let result = run(&["basename", "/path/to/file"]);
    assert_eq!(result.0, 0);
}
