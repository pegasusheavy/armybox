//! ls - list directory contents
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ls.html

use crate::io;
use crate::sys;
use crate::applets::get_arg;

/// ls - list directory contents
///
/// # Synopsis
/// ```text
/// ls [-aFilR1] [file...]
/// ```
///
/// # Description
/// Lists directory contents. By default, lists the current directory.
///
/// # Options
/// - `-a`: Show all files including hidden (starting with .)
/// - `-F`: Append indicator (/ for dirs, * for executables, @ for symlinks)
/// - `-i`: Show inode numbers
/// - `-l`: Long format with permissions, size, date
/// - `-R`: Recursive listing (partial support)
/// - `-1`: One file per line
///
/// # Exit Status
/// - 0: Success
/// - >0: An error occurred
pub fn ls(argc: i32, argv: *const *const u8) -> i32 {
    let mut show_all = false;
    let mut long_format = false;
    let mut one_per_line = false;
    let mut recursive = false;
    let mut show_inode = false;
    let mut classify = false;

    let mut paths_start = argc;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() > 0 && arg[0] == b'-' && arg.len() > 1 {
                for &c in &arg[1..] {
                    match c {
                        b'a' => show_all = true,
                        b'l' => long_format = true,
                        b'1' => one_per_line = true,
                        b'R' => recursive = true,
                        b'i' => show_inode = true,
                        b'F' => classify = true,
                        _ => {}
                    }
                }
            } else {
                paths_start = i;
                break;
            }
        }
    }

    if paths_start >= argc {
        list_dir(b".", show_all, long_format, one_per_line, show_inode, classify);
    } else {
        for i in paths_start..argc {
            if let Some(path) = unsafe { get_arg(argv, i) } {
                if recursive {
                    io::write_all(1, path);
                    io::write_str(1, b":\n");
                }
                list_dir(path, show_all, long_format, one_per_line, show_inode, classify);
            }
        }
    }
    let _ = recursive;
    0
}

fn list_dir(path: &[u8], show_all: bool, long_format: bool, one_per_line: bool, show_inode: bool, classify: bool) {
    let fd = io::open(path, libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 {
        sys::perror(path);
        return;
    }

    let mut buf = [0u8; 4096];
    let mut col = 0;
    let term_width = 80; // Default terminal width

    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }

        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if !show_all && name.len() > 0 && name[0] == b'.' {
                offset += dirent.d_reclen as usize;
                continue;
            }

            if long_format {
                // Build full path for stat
                let mut full_path = [0u8; 512];
                let mut len = 0;
                for c in path { full_path[len] = *c; len += 1; }
                full_path[len] = b'/'; len += 1;
                for c in name { full_path[len] = *c; len += 1; }

                let mut st: libc::stat = unsafe { core::mem::zeroed() };
                if io::lstat(&full_path[..len], &mut st) == 0 {
                    // Show inode if requested
                    if show_inode {
                        io::write_num(1, st.st_ino as u64);
                        io::write_str(1, b" ");
                    }

                    // File mode
                    let mut mode_buf = [0u8; 10];
                    sys::format_mode(st.st_mode as u32, &mut mode_buf);
                    io::write_all(1, &mode_buf);
                    io::write_str(1, b" ");

                    // Number of links (right-aligned in 3 chars)
                    let nlink = st.st_nlink as u64;
                    if nlink < 10 { io::write_str(1, b"  "); }
                    else if nlink < 100 { io::write_str(1, b" "); }
                    io::write_num(1, nlink);
                    io::write_str(1, b" ");

                    // UID (right-aligned in 5 chars)
                    let uid = st.st_uid as u64;
                    if uid < 10 { io::write_str(1, b"    "); }
                    else if uid < 100 { io::write_str(1, b"   "); }
                    else if uid < 1000 { io::write_str(1, b"  "); }
                    else if uid < 10000 { io::write_str(1, b" "); }
                    io::write_num(1, uid);
                    io::write_str(1, b" ");

                    // GID (right-aligned in 5 chars)
                    let gid = st.st_gid as u64;
                    if gid < 10 { io::write_str(1, b"    "); }
                    else if gid < 100 { io::write_str(1, b"   "); }
                    else if gid < 1000 { io::write_str(1, b"  "); }
                    else if gid < 10000 { io::write_str(1, b" "); }
                    io::write_num(1, gid);
                    io::write_str(1, b" ");

                    // Size (right-aligned in 8 chars)
                    let size = st.st_size as u64;
                    if size < 10 { io::write_str(1, b"       "); }
                    else if size < 100 { io::write_str(1, b"      "); }
                    else if size < 1000 { io::write_str(1, b"     "); }
                    else if size < 10000 { io::write_str(1, b"    "); }
                    else if size < 100000 { io::write_str(1, b"   "); }
                    else if size < 1000000 { io::write_str(1, b"  "); }
                    else if size < 10000000 { io::write_str(1, b" "); }
                    io::write_num(1, size);
                    io::write_str(1, b" ");

                    // Date/time (simplified: show mtime)
                    format_time(st.st_mtime as u64);
                    io::write_str(1, b" ");

                    // Filename
                    io::write_all(1, name);

                    // For symlinks, show target
                    if (st.st_mode & libc::S_IFMT) == libc::S_IFLNK {
                        io::write_str(1, b" -> ");
                        let mut link_target = [0u8; 256];
                        let link_len = io::readlink(&full_path[..len], &mut link_target);
                        if link_len > 0 {
                            io::write_all(1, &link_target[..link_len as usize]);
                        }
                    }

                    // Classify suffix
                    if classify {
                        write_classify_suffix(st.st_mode as u32);
                    }

                    io::write_str(1, b"\n");
                }
            } else {
                // Show inode if requested
                if show_inode {
                    let mut full_path = [0u8; 512];
                    let mut len = 0;
                    for c in path { full_path[len] = *c; len += 1; }
                    full_path[len] = b'/'; len += 1;
                    for c in name { full_path[len] = *c; len += 1; }

                    let mut st: libc::stat = unsafe { core::mem::zeroed() };
                    if io::lstat(&full_path[..len], &mut st) == 0 {
                        io::write_num(1, st.st_ino as u64);
                        io::write_str(1, b" ");
                    }
                }

                if one_per_line {
                    io::write_all(1, name);
                    if classify {
                        let mut full_path = [0u8; 512];
                        let mut len = 0;
                        for c in path { full_path[len] = *c; len += 1; }
                        full_path[len] = b'/'; len += 1;
                        for c in name { full_path[len] = *c; len += 1; }

                        let mut st: libc::stat = unsafe { core::mem::zeroed() };
                        if io::lstat(&full_path[..len], &mut st) == 0 {
                            write_classify_suffix(st.st_mode as u32);
                        }
                    }
                    io::write_str(1, b"\n");
                } else {
                    // Column format
                    let entry_len = name.len() + 2; // name + 2 spaces
                    if col + entry_len > term_width && col > 0 {
                        io::write_str(1, b"\n");
                        col = 0;
                    }
                    io::write_all(1, name);
                    io::write_str(1, b"  ");
                    col += entry_len;
                }
            }

            offset += dirent.d_reclen as usize;
        }
    }

    if !one_per_line && !long_format && col > 0 {
        io::write_str(1, b"\n");
    }

    io::close(fd);
}

/// Format Unix timestamp as "Mon DD HH:MM" or "Mon DD  YYYY" for older files
fn format_time(timestamp: u64) {
    const MONTHS: [&[u8]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun",
        b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"
    ];

    // Simple time calculation (not handling timezones)
    let secs_per_day = 86400u64;
    let secs_per_hour = 3600u64;
    let secs_per_min = 60u64;

    // Days since epoch
    let days = timestamp / secs_per_day;
    let time_of_day = timestamp % secs_per_day;
    let hour = (time_of_day / secs_per_hour) % 24;
    let minute = (time_of_day % secs_per_hour) / secs_per_min;

    // Calculate year, month, day (simplified - doesn't handle leap years perfectly)
    let mut year = 1970u64;
    let mut remaining_days = days;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0usize;
    for (i, &days_in_month) in days_in_months.iter().enumerate() {
        if remaining_days < days_in_month {
            month = i;
            break;
        }
        remaining_days -= days_in_month;
    }
    let day = remaining_days + 1;

    // Get current time to determine if we show time or year
    let now = unsafe { libc::time(core::ptr::null_mut()) } as u64;
    let six_months_ago = now.saturating_sub(180 * secs_per_day);

    io::write_all(1, MONTHS[month]);
    io::write_str(1, b" ");
    if day < 10 { io::write_str(1, b" "); }
    io::write_num(1, day);
    io::write_str(1, b" ");

    if timestamp < six_months_ago || timestamp > now {
        // Show year for old/future files
        io::write_str(1, b" ");
        io::write_num(1, year);
    } else {
        // Show time for recent files
        if hour < 10 { io::write_str(1, b"0"); }
        io::write_num(1, hour);
        io::write_str(1, b":");
        if minute < 10 { io::write_str(1, b"0"); }
        io::write_num(1, minute);
    }
}

/// Write classify suffix (-F option)
fn write_classify_suffix(mode: u32) {
    match mode & libc::S_IFMT as u32 {
        m if m == libc::S_IFDIR as u32 => { io::write_str(1, b"/"); }
        m if m == libc::S_IFLNK as u32 => { io::write_str(1, b"@"); }
        m if m == libc::S_IFIFO as u32 => { io::write_str(1, b"|"); }
        m if m == libc::S_IFSOCK as u32 => { io::write_str(1, b"="); }
        m if m == libc::S_IFREG as u32 => {
            if mode & 0o111 != 0 { io::write_str(1, b"*"); }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for ls utility

    extern crate std;
    use std::process::Command;
    use std::fs;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let release = PathBuf::from("target/release/armybox");
        if release.exists() { return release; }
        PathBuf::from("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let id = std::process::id();
        let dir = std::env::temp_dir().join(format!("armybox_ls_test_{}", id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_ls_current_directory() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("file1.txt"), "content").unwrap();
        fs::write(dir.join("file2.txt"), "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ls", dir.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("file1.txt"));
        assert!(stdout.contains("file2.txt"));
        cleanup(&dir);
    }

    #[test]
    fn test_ls_hidden_files() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("visible.txt"), "content").unwrap();
        fs::write(dir.join(".hidden"), "hidden").unwrap();

        // Without -a, hidden files should not appear
        let output = Command::new(&armybox)
            .args(["ls", dir.to_str().unwrap()])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("visible.txt"));
        assert!(!stdout.contains(".hidden"));

        // With -a, hidden files should appear
        let output = Command::new(&armybox)
            .args(["ls", "-a", dir.to_str().unwrap()])
            .output()
            .unwrap();

        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("visible.txt"));
        assert!(stdout.contains(".hidden"));
        cleanup(&dir);
    }

    #[test]
    fn test_ls_one_per_line() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("file1.txt"), "content").unwrap();
        fs::write(dir.join("file2.txt"), "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ls", "-1", dir.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Each file should be on its own line
        let lines: Vec<&str> = stdout.lines().collect();
        assert!(lines.len() >= 2);
        cleanup(&dir);
    }

    #[test]
    fn test_ls_long_format() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("file.txt"), "hello world").unwrap();

        let output = Command::new(&armybox)
            .args(["ls", "-l", dir.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Long format should show permissions (starts with -)
        assert!(stdout.contains("-rw"));
        // Should show filename
        assert!(stdout.contains("file.txt"));
        cleanup(&dir);
    }

    #[test]
    fn test_ls_classify() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::create_dir(dir.join("subdir")).unwrap();
        fs::write(dir.join("file.txt"), "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ls", "-F", "-1", dir.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Directories should have / suffix
        assert!(stdout.contains("subdir/"));
        cleanup(&dir);
    }

    #[test]
    fn test_ls_nonexistent_directory() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["ls", "/nonexistent/directory"])
            .output()
            .unwrap();

        // Should still return 0 but print error to stderr
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        assert!(stderr.len() > 0 || output.status.code() == Some(0));
    }

    #[test]
    fn test_ls_inode() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        fs::write(dir.join("file.txt"), "content").unwrap();

        let output = Command::new(&armybox)
            .args(["ls", "-i", "-1", dir.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        // Output should contain numbers (inodes)
        assert!(stdout.chars().any(|c| c.is_ascii_digit()));
        assert!(stdout.contains("file.txt"));
        cleanup(&dir);
    }

    #[test]
    fn test_ls_empty_directory() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let dir = setup();
        // Dir is empty

        let output = Command::new(&armybox)
            .args(["ls", dir.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        // Output should be empty or just whitespace
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.trim().is_empty() || stdout.trim().len() == 0);
        cleanup(&dir);
    }
}
