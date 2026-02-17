//! find - search for files in a directory hierarchy
//!
//! GNU findutils compatible implementation.

use crate::io;
use crate::applets::get_arg;

/// find - search for files in a directory hierarchy
///
/// # Synopsis
/// ```text
/// find [path...] [expression]
/// ```
///
/// # Description
/// Search for files in a directory hierarchy.
///
/// # Expressions
/// - `-name pattern`: Base of file name matches pattern
/// - `-type c`: File is of type c (f=file, d=directory, l=symlink)
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn find(argc: i32, argv: *const *const u8) -> i32 {
    let start_path = if argc > 1 {
        let first = unsafe { get_arg(argv, 1).unwrap() };
        if first.len() > 0 && first[0] != b'-' { first } else { b"." }
    } else {
        b"."
    };

    let mut name_pattern: Option<&[u8]> = None;
    let mut file_type: Option<u8> = None;

    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-name" && i + 1 < argc {
                name_pattern = unsafe { get_arg(argv, i + 1) };
                i += 1;
            } else if arg == b"-type" && i + 1 < argc {
                if let Some(t) = unsafe { get_arg(argv, i + 1) } {
                    if t.len() > 0 {
                        file_type = Some(t[0]);
                    }
                }
                i += 1;
            }
        }
        i += 1;
    }

    find_recursive(start_path, name_pattern, file_type);
    0
}

fn find_recursive(path: &[u8], name_pattern: Option<&[u8]>, file_type: Option<u8>) {
    let fd = io::open(path, libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return; }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }

        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if name != b"." && name != b".." {
                // Build full path
                let mut full_path = [0u8; 512];
                let mut len = 0;
                for c in path { full_path[len] = *c; len += 1; }
                if path.len() > 0 && path[path.len()-1] != b'/' {
                    full_path[len] = b'/'; len += 1;
                }
                for c in name { full_path[len] = *c; len += 1; }

                // Check type
                let type_ok = match file_type {
                    Some(b'f') => dirent.d_type == libc::DT_REG,
                    Some(b'd') => dirent.d_type == libc::DT_DIR,
                    Some(b'l') => dirent.d_type == libc::DT_LNK,
                    _ => true,
                };

                // Check name pattern (simple glob)
                let name_ok = match name_pattern {
                    Some(p) => {
                        if p.len() >= 2 && p[0] == b'*' {
                            // *.ext pattern
                            name.ends_with(&p[1..])
                        } else {
                            name == p
                        }
                    }
                    None => true,
                };

                if type_ok && name_ok {
                    io::write_all(1, &full_path[..len]);
                    io::write_str(1, b"\n");
                }

                // Recurse into directories
                if dirent.d_type == libc::DT_DIR {
                    find_recursive(&full_path[..len], name_pattern, file_type);
                }
            }

            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    use std::process::Command;
    use std::fs;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() { return release; }
        manifest_dir.join("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("armybox_find_test_{}_{}",  std::process::id(), counter));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_find_all() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("file1.txt"), "").unwrap();
        fs::write(dir.join("file2.txt"), "").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();
        fs::write(dir.join("subdir/file3.txt"), "").unwrap();

        let output = Command::new(&armybox)
            .args(["find", dir.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("file1.txt"));
        assert!(stdout.contains("file2.txt"));
        assert!(stdout.contains("file3.txt"));
        cleanup(&dir);
    }

    #[test]
    fn test_find_by_name() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("file1.txt"), "").unwrap();
        fs::write(dir.join("file2.rs"), "").unwrap();

        let output = Command::new(&armybox)
            .args(["find", dir.to_str().unwrap(), "-name", "*.txt"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("file1.txt"));
        assert!(!stdout.contains("file2.rs"));
        cleanup(&dir);
    }

    #[test]
    fn test_find_by_type() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("file.txt"), "").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();

        let output = Command::new(&armybox)
            .args(["find", dir.to_str().unwrap(), "-type", "d"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("subdir"));
        assert!(!stdout.contains("file.txt"));
        cleanup(&dir);
    }

    #[test]
    fn test_find_default_path() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["find"])
            .output()
            .unwrap();

        // Should search current directory
        assert_eq!(output.status.code(), Some(0));
    }
}
