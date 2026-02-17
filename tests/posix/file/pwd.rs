//! POSIX.1-2017 compliance tests for pwd
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pwd.html

use crate::posix::helpers::*;

/// POSIX: "The pwd utility shall write to standard output an absolute
/// pathname of the current working directory"
#[test]
fn posix_pwd_basic() {
    let dir = setup_test_env();
    let result = run_in_dir(&["pwd"], dir.path());
    assert_success(&result);
    // Output should be an absolute path ending with newline
    assert!(result.1.starts_with('/'));
    assert!(result.1.ends_with('\n'));
}

/// POSIX: pwd output shall be absolute pathname
#[test]
fn posix_pwd_absolute() {
    let result = run(&["pwd"]);
    assert_success(&result);
    let path = result.1.trim();
    assert!(
        path.starts_with('/'),
        "pwd output should be absolute path, got: {}",
        path
    );
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_pwd_exit_success() {
    let result = run(&["pwd"]);
    assert_eq!(result.0, 0);
}

/// POSIX: -L option (logical pathname, default behavior)
#[test]
fn posix_pwd_logical() {
    let result = run(&["pwd", "-L"]);
    assert_success(&result);
    assert!(result.1.trim().starts_with('/'));
}

/// POSIX: -P option (physical pathname, no symlinks)
#[test]
fn posix_pwd_physical() {
    let result = run(&["pwd", "-P"]);
    assert_success(&result);
    assert!(result.1.trim().starts_with('/'));
}
