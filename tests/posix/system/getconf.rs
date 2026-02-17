//! POSIX.1-2017 compliance tests for getconf
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/getconf.html

use crate::posix::helpers::*;

/// POSIX: "The getconf utility shall write the value of a configuration variable"
#[test]
fn posix_getconf_page_size() {
    let result = run(&["getconf", "PAGE_SIZE"]);
    assert_success(&result);
    let size: u64 = result.1.trim().parse().unwrap();
    assert!(size >= 512); // Page size at least 512 bytes
}

/// POSIX: getconf PATH_MAX
#[test]
fn posix_getconf_path_max() {
    let result = run(&["getconf", "PATH_MAX", "/"]);
    assert_success(&result);
    let max: i64 = result.1.trim().parse().unwrap();
    assert!(max > 0 || max == -1); // -1 means unlimited
}

/// POSIX: getconf NAME_MAX
#[test]
fn posix_getconf_name_max() {
    let result = run(&["getconf", "NAME_MAX", "/"]);
    assert_success(&result);
    let max: i64 = result.1.trim().parse().unwrap();
    assert!(max > 0);
}

/// POSIX: getconf _POSIX_VERSION
#[test]
fn posix_getconf_posix_version() {
    let result = run(&["getconf", "_POSIX_VERSION"]);
    assert_success(&result);
    // Should be a version like 200809 or 200112
    let version: i64 = result.1.trim().parse().unwrap();
    assert!(version >= 199009);
}

/// POSIX: getconf ARG_MAX
#[test]
fn posix_getconf_arg_max() {
    let result = run(&["getconf", "ARG_MAX"]);
    assert_success(&result);
    let max: i64 = result.1.trim().parse().unwrap();
    assert!(max > 0);
}

/// POSIX: getconf NGROUPS_MAX
#[test]
fn posix_getconf_ngroups_max() {
    let result = run(&["getconf", "NGROUPS_MAX"]);
    assert_success(&result);
    let max: i64 = result.1.trim().parse().unwrap();
    assert!(max > 0);
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_getconf_exit_success() {
    let result = run(&["getconf", "PAGE_SIZE"]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 for invalid variable
#[test]
fn posix_getconf_exit_error() {
    let result = run(&["getconf", "INVALID_VAR_NAME_12345"]);
    assert!(result.0 > 0);
}
