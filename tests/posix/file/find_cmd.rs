//! POSIX.1-2017 compliance tests for find
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/find.html

use crate::posix::helpers::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

/// POSIX: "The find utility shall recursively descend the directory hierarchy"
#[test]
fn posix_find_basic() {
    let dir = setup_test_env();
    fs::write(dir.path().join("file1"), "a").unwrap();
    fs::create_dir(dir.path().join("subdir")).unwrap();
    fs::write(dir.path().join("subdir/file2"), "b").unwrap();

    let result = run(&["find", dir.path().to_str().unwrap()]);
    assert_success(&result);
    assert!(result.1.contains("file1"));
    assert!(result.1.contains("file2"));
}

/// POSIX: find -name pattern matching
#[test]
fn posix_find_name() {
    let dir = setup_test_env();
    fs::write(dir.path().join("test.txt"), "a").unwrap();
    fs::write(dir.path().join("test.log"), "b").unwrap();

    let result = run(&["find", dir.path().to_str().unwrap(), "-name", "*.txt"]);
    assert_success(&result);
    assert!(result.1.contains("test.txt"));
    assert!(!result.1.contains("test.log"));
}

/// POSIX: find -type f (regular files)
#[test]
fn posix_find_type_file() {
    let dir = setup_test_env();
    fs::write(dir.path().join("file"), "content").unwrap();
    fs::create_dir(dir.path().join("directory")).unwrap();

    let result = run(&["find", dir.path().to_str().unwrap(), "-type", "f"]);
    assert_success(&result);
    assert!(result.1.contains("file"));
    assert!(!result.1.contains("directory") || result.1.lines().all(|l| !l.ends_with("directory")));
}

/// POSIX: find -type d (directories)
#[test]
fn posix_find_type_directory() {
    let dir = setup_test_env();
    fs::write(dir.path().join("file"), "content").unwrap();
    fs::create_dir(dir.path().join("directory")).unwrap();

    let result = run(&["find", dir.path().to_str().unwrap(), "-type", "d"]);
    assert_success(&result);
    assert!(result.1.contains("directory"));
}

/// POSIX: find -exec command
#[test]
fn posix_find_exec() {
    let dir = setup_test_env();
    fs::write(dir.path().join("test.txt"), "content").unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-name",
        "*.txt",
        "-exec",
        "echo",
        "{}",
        ";",
    ]);
    assert_success(&result);
    assert!(result.1.contains("test.txt"));
}

/// POSIX: find -print (explicit)
#[test]
fn posix_find_print() {
    let dir = setup_test_env();
    fs::write(dir.path().join("file"), "x").unwrap();

    let result = run(&["find", dir.path().to_str().unwrap(), "-print"]);
    assert_success(&result);
    assert!(result.1.contains("file"));
}

/// POSIX: find -depth processes directory contents before directory
#[test]
fn posix_find_depth() {
    let dir = setup_test_env();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file"), "x").unwrap();

    let result = run(&["find", dir.path().to_str().unwrap(), "-depth"]);
    assert_success(&result);
    // File should appear before its parent directory
    let file_pos = result.1.find("file").unwrap_or(999);
    let subdir_pos = result.1.rfind("subdir").unwrap_or(0); // Use rfind for last occurrence
    assert!(file_pos < subdir_pos);
}

/// POSIX: find with -maxdepth (extension)
#[test]
fn posix_find_maxdepth() {
    let dir = setup_test_env();
    // depths: dir=0, a=1, a/top=2 & a/b=2, a/b/middle=3, a/b/c=3, a/b/c/deep=4
    fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
    fs::write(dir.path().join("a/top"), "x").unwrap();
    fs::write(dir.path().join("a/b/middle_file"), "x").unwrap();
    fs::write(dir.path().join("a/b/c/deep_file"), "x").unwrap();

    let result = run(&["find", dir.path().to_str().unwrap(), "-maxdepth", "2"]);
    assert_success(&result);
    // Depth <= 2 is included.
    assert!(result.1.contains("top"));
    // Depth > 2 is excluded.
    assert!(!result.1.contains("middle_file"));
    assert!(!result.1.contains("deep_file"));
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_find_exit_success() {
    let dir = setup_test_env();
    let result = run(&["find", dir.path().to_str().unwrap()]);
    assert_eq!(result.0, 0);
}

/// POSIX: Exit status >0 on error
#[test]
fn posix_find_exit_error() {
    let result = run(&["find", "/nonexistent/path"]);
    assert!(result.0 > 0);
}

/// POSIX: find -newer file
#[test]
fn posix_find_newer() {
    let dir = setup_test_env();
    let older = dir.path().join("older");
    fs::write(&older, "x").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let newer = dir.path().join("newer");
    fs::write(&newer, "x").unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-newer",
        older.to_str().unwrap(),
    ]);
    assert_success(&result);
    assert!(result.1.contains("newer"));
}

/// POSIX: find -size
#[test]
fn posix_find_size() {
    let dir = setup_test_env();
    fs::write(dir.path().join("small"), "x").unwrap();
    fs::write(dir.path().join("large"), "x".repeat(10000)).unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "-size",
        "+1k",
    ]);
    assert_success(&result);
    assert!(result.1.contains("large"));
    assert!(!result.1.contains("small"));
}

/// POSIX: find -perm mode
#[test]
fn posix_find_perm() {
    let dir = setup_test_env();
    let exec = dir.path().join("executable");
    let noexec = dir.path().join("noexec");
    fs::write(&exec, "x").unwrap();
    fs::write(&noexec, "x").unwrap();
    fs::set_permissions(&exec, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&noexec, fs::Permissions::from_mode(0o644)).unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "-perm",
        "-111",
    ]);
    // -perm -111 matches entries with all execute bits set.
    assert_success(&result);
    assert!(result.1.contains("executable"));
    assert!(!result.1.contains("noexec"));
}

/// POSIX: find logical operators -a (and)
#[test]
fn posix_find_and() {
    let dir = setup_test_env();
    fs::write(dir.path().join("test.txt"), "content").unwrap();
    fs::write(dir.path().join("test.log"), "x").unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "-a",
        "-name",
        "*.txt",
    ]);
    assert_success(&result);
    assert!(result.1.contains("test.txt"));
    assert!(!result.1.contains("test.log"));
}

/// POSIX: find logical operators -o (or)
#[test]
fn posix_find_or() {
    let dir = setup_test_env();
    fs::write(dir.path().join("test.txt"), "x").unwrap();
    fs::write(dir.path().join("test.log"), "x").unwrap();
    fs::write(dir.path().join("other.dat"), "x").unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-name",
        "*.txt",
        "-o",
        "-name",
        "*.log",
    ]);
    assert_success(&result);
    assert!(result.1.contains("test.txt"));
    assert!(result.1.contains("test.log"));
}

/// POSIX: find -prune stops descent into matched directories
#[test]
fn posix_find_prune() {
    let dir = setup_test_env();
    fs::write(dir.path().join("keep"), "x").unwrap();
    fs::create_dir(dir.path().join("skip")).unwrap();
    fs::write(dir.path().join("skip/pruned_file"), "x").unwrap();

    // `-name skip -prune -o -print`: the skip directory matches and is pruned
    // (so -print is not reached for it), while everything else is printed.
    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-name",
        "skip",
        "-prune",
        "-o",
        "-print",
    ]);
    assert_success(&result);
    assert!(result.1.contains("keep"));
    // The pruned directory is never descended into.
    assert!(!result.1.contains("pruned_file"));
    // The pruned directory itself is not printed.
    assert!(!result.1.lines().any(|l| l.ends_with("skip")));
}

/// POSIX: find ! (negation)
#[test]
fn posix_find_not() {
    let dir = setup_test_env();
    fs::write(dir.path().join("keep.txt"), "x").unwrap();
    fs::write(dir.path().join("drop.log"), "x").unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "!",
        "-name",
        "*.log",
    ]);
    assert_success(&result);
    assert!(result.1.contains("keep.txt"));
    assert!(!result.1.contains("drop.log"));
}

/// POSIX: find ( ) grouping overrides -a/-o precedence
#[test]
fn posix_find_group_precedence() {
    let dir = setup_test_env();
    fs::write(dir.path().join("a.txt"), "x").unwrap();
    fs::write(dir.path().join("b.dat"), "x").unwrap();
    // A directory whose name matches *.txt; -type f must still exclude it.
    fs::create_dir(dir.path().join("c.txt")).unwrap();

    // ( -name '*.txt' -o -name '*.dat' ) -type f
    // The parentheses group the OR so the trailing -type f applies to the whole
    // group; without them, -a would bind tighter than -o.
    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "(",
        "-name",
        "*.txt",
        "-o",
        "-name",
        "*.dat",
        ")",
        "-type",
        "f",
    ]);
    assert_success(&result);
    assert!(result.1.contains("a.txt"));
    assert!(result.1.contains("b.dat"));
    // c.txt is a directory: matched by name but excluded by -type f.
    assert!(!result.1.lines().any(|l| l.ends_with("c.txt")));
}

/// POSIX: find -size with unit suffixes (c bytes, k KiB)
#[test]
fn posix_find_size_suffixes() {
    let dir = setup_test_env();
    fs::write(dir.path().join("tiny"), "abc").unwrap(); // 3 bytes
    fs::write(dir.path().join("big"), "x".repeat(5000)).unwrap(); // ~5 KiB

    // -size -3k : rounds up to KiB units; tiny -> 1 unit (< 3), big -> 5 (not < 3)
    let r1 = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "-size",
        "-3k",
    ]);
    assert_success(&r1);
    assert!(r1.1.contains("tiny"));
    assert!(!r1.1.contains("big"));

    // -size +4k : big -> 5 units (> 4), tiny -> 1 (not > 4)
    let r2 = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "-size",
        "+4k",
    ]);
    assert_success(&r2);
    assert!(r2.1.contains("big"));
    assert!(!r2.1.contains("tiny"));

    // -size +2c : byte units; both files exceed 2 bytes.
    let r3 = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "-size",
        "+2c",
    ]);
    assert_success(&r3);
    assert!(r3.1.contains("tiny"));
    assert!(r3.1.contains("big"));
}

/// POSIX: `-exec false ;` fails per match, so find exits nonzero.
#[test]
fn posix_find_exec_false_exit() {
    let dir = setup_test_env();
    fs::write(dir.path().join("f"), "x").unwrap();

    let result = run(&[
        "find",
        dir.path().to_str().unwrap(),
        "-type",
        "f",
        "-exec",
        "false",
        ";",
    ]);
    // `false` exits 1 for the matched file, which makes find exit 1.
    assert_eq!(result.0, 1);
}
