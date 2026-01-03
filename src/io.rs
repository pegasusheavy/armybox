//! Low-level I/O primitives using libc
//!
//! This module provides basic I/O operations without std dependency.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::ptr;

/// Write all bytes to a file descriptor
pub fn write_all(fd: i32, buf: &[u8]) -> isize {
    let mut written = 0;
    while written < buf.len() {
        let ret = unsafe {
            libc::write(
                fd,
                buf[written..].as_ptr() as *const libc::c_void,
                buf.len() - written,
            )
        };
        if ret < 0 {
            return ret;
        }
        written += ret as usize;
    }
    written as isize
}

/// Write all bytes to fd (alias for write_all returning isize)
pub fn write_all_fd(fd: i32, buf: &[u8]) -> isize {
    write_all(fd, buf)
}

/// Write all bytes and return count written
pub fn write_all_count(fd: i32, buf: &[u8]) -> usize {
    let mut written = 0;
    while written < buf.len() {
        let ret = unsafe {
            libc::write(
                fd,
                buf[written..].as_ptr() as *const libc::c_void,
                buf.len() - written,
            )
        };
        if ret < 0 {
            break;
        }
        written += ret as usize;
    }
    written
}

/// Write a string literal to fd
pub fn write_str(fd: i32, s: &[u8]) -> isize {
    write_all(fd, s)
}

/// Write a number to fd
pub fn write_num(fd: i32, mut n: u64) -> isize {
    if n == 0 {
        return write_str(fd, b"0");
    }

    let mut buf = [0u8; 20];
    let mut i = buf.len();

    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }

    write_all(fd, &buf[i..])
}

/// Write a signed number to fd
pub fn write_signed(fd: i32, n: i64) -> isize {
    if n < 0 {
        write_str(fd, b"-");
        write_num(fd, (-n) as u64)
    } else {
        write_num(fd, n as u64)
    }
}

/// Read from file descriptor into buffer
pub fn read(fd: i32, buf: &mut [u8]) -> isize {
    unsafe {
        libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
    }
}

/// Read entire file into Vec
#[cfg(feature = "alloc")]
pub fn read_all(fd: i32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buf = [0u8; 4096];

    loop {
        let n = read(fd, &mut buf);
        if n <= 0 {
            break;
        }
        result.extend_from_slice(&buf[..n as usize]);
    }

    result
}

/// Open a file
pub fn open(path: &[u8], flags: i32, mode: u32) -> i32 {
    // Ensure null-terminated path
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::open(path_buf.as_ptr() as *const i8, flags, mode) }
}

/// Close a file descriptor
pub fn close(fd: i32) -> i32 {
    unsafe { libc::close(fd) }
}

/// Get file status
pub fn stat(path: &[u8], buf: &mut libc::stat) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::stat(path_buf.as_ptr() as *const i8, buf) }
}

/// Get file status (no follow symlinks)
pub fn lstat(path: &[u8], buf: &mut libc::stat) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::lstat(path_buf.as_ptr() as *const i8, buf) }
}

/// Get file status from fd
pub fn fstat(fd: i32, buf: &mut libc::stat) -> i32 {
    unsafe { libc::fstat(fd, buf) }
}

/// Create a directory
pub fn mkdir(path: &[u8], mode: u32) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::mkdir(path_buf.as_ptr() as *const i8, mode as libc::mode_t) }
}

/// Remove a directory
pub fn rmdir(path: &[u8]) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::rmdir(path_buf.as_ptr() as *const i8) }
}

/// Unlink (remove) a file
pub fn unlink(path: &[u8]) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::unlink(path_buf.as_ptr() as *const i8) }
}

/// Rename a file
pub fn rename(old: &[u8], new: &[u8]) -> i32 {
    let mut old_buf = [0u8; 4096];
    let mut new_buf = [0u8; 4096];

    if old.len() >= old_buf.len() || new.len() >= new_buf.len() {
        return -1;
    }

    old_buf[..old.len()].copy_from_slice(old);
    old_buf[old.len()] = 0;
    new_buf[..new.len()].copy_from_slice(new);
    new_buf[new.len()] = 0;

    unsafe { libc::rename(old_buf.as_ptr() as *const i8, new_buf.as_ptr() as *const i8) }
}

/// Create a symlink
pub fn symlink(target: &[u8], linkpath: &[u8]) -> i32 {
    let mut target_buf = [0u8; 4096];
    let mut link_buf = [0u8; 4096];

    if target.len() >= target_buf.len() || linkpath.len() >= link_buf.len() {
        return -1;
    }

    target_buf[..target.len()].copy_from_slice(target);
    target_buf[target.len()] = 0;
    link_buf[..linkpath.len()].copy_from_slice(linkpath);
    link_buf[linkpath.len()] = 0;

    unsafe { libc::symlink(target_buf.as_ptr() as *const i8, link_buf.as_ptr() as *const i8) }
}

/// Create a hard link
pub fn link(old: &[u8], new: &[u8]) -> i32 {
    let mut old_buf = [0u8; 4096];
    let mut new_buf = [0u8; 4096];

    if old.len() >= old_buf.len() || new.len() >= new_buf.len() {
        return -1;
    }

    old_buf[..old.len()].copy_from_slice(old);
    old_buf[old.len()] = 0;
    new_buf[..new.len()].copy_from_slice(new);
    new_buf[new.len()] = 0;

    unsafe { libc::link(old_buf.as_ptr() as *const i8, new_buf.as_ptr() as *const i8) }
}

/// Change file permissions
pub fn chmod(path: &[u8], mode: u32) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::chmod(path_buf.as_ptr() as *const i8, mode as libc::mode_t) }
}

/// Change working directory
pub fn chdir(path: &[u8]) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::chdir(path_buf.as_ptr() as *const i8) }
}

/// Get current working directory
#[cfg(feature = "alloc")]
pub fn getcwd() -> Option<Vec<u8>> {
    let mut buf = [0u8; 4096];
    let ret = unsafe { libc::getcwd(buf.as_mut_ptr() as *mut i8, buf.len()) };

    if ret.is_null() {
        None
    } else {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(buf[..len].to_vec())
    }
}

/// Get current working directory (no alloc version)
#[cfg(not(feature = "alloc"))]
pub fn getcwd(buf: &mut [u8]) -> bool {
    let ret = unsafe { libc::getcwd(buf.as_mut_ptr() as *mut i8, buf.len()) };
    !ret.is_null()
}

/// Read symbolic link
pub fn readlink(path: &[u8], buf: &mut [u8]) -> isize {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() { return -1; }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::readlink(path_buf.as_ptr() as *const i8, buf.as_mut_ptr() as *mut i8, buf.len()) }
}

/// Get canonical path
pub fn realpath(path: &[u8], buf: &mut [u8]) -> isize {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() { return -1; }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    let ret = unsafe { libc::realpath(path_buf.as_ptr() as *const i8, buf.as_mut_ptr() as *mut i8) };
    if ret.is_null() {
        -1
    } else {
        strlen_arr(buf) as isize
    }
}

/// Open directory for reading
pub fn opendir(path: &[u8]) -> *mut libc::DIR {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return ptr::null_mut();
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::opendir(path_buf.as_ptr() as *const i8) }
}

/// Read directory entry
pub fn readdir(dir: *mut libc::DIR) -> *mut libc::dirent {
    unsafe { libc::readdir(dir) }
}

/// Close directory
pub fn closedir(dir: *mut libc::DIR) -> i32 {
    unsafe { libc::closedir(dir) }
}

/// Get user ID
pub fn getuid() -> u32 {
    unsafe { libc::getuid() }
}

/// Get effective user ID
pub fn geteuid() -> u32 {
    unsafe { libc::geteuid() }
}

/// Get group ID
pub fn getgid() -> u32 {
    unsafe { libc::getgid() }
}

/// Get effective group ID
pub fn getegid() -> u32 {
    unsafe { libc::getegid() }
}

/// Check file access permissions (POSIX access())
pub fn access(path: &[u8], mode: i32) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::access(path_buf.as_ptr() as *const i8, mode) }
}

/// Get process ID
pub fn getpid() -> i32 {
    unsafe { libc::getpid() }
}

/// Get parent process ID
pub fn getppid() -> i32 {
    unsafe { libc::getppid() }
}

/// Get hostname
#[cfg(feature = "alloc")]
pub fn gethostname() -> Option<Vec<u8>> {
    let mut buf = [0u8; 256];
    let ret = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) };

    if ret != 0 {
        None
    } else {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(buf[..len].to_vec())
    }
}

/// Sleep for seconds
pub fn sleep(secs: u32) {
    unsafe { libc::sleep(secs); }
}

/// Sleep for microseconds
pub fn usleep(usecs: u32) {
    unsafe { libc::usleep(usecs); }
}

/// Duplicate file descriptor
pub fn dup(fd: i32) -> i32 {
    unsafe { libc::dup(fd) }
}

/// Duplicate file descriptor to specific fd
pub fn dup2(old: i32, new: i32) -> i32 {
    unsafe { libc::dup2(old, new) }
}

/// Seek in file
pub fn lseek(fd: i32, offset: i64, whence: i32) -> i64 {
    unsafe { libc::lseek(fd, offset as libc::off_t, whence) as i64 }
}

/// Truncate file
pub fn ftruncate(fd: i32, length: i64) -> i32 {
    unsafe { libc::ftruncate(fd, length as libc::off_t) }
}

/// Sync filesystem
pub fn sync() {
    unsafe { libc::sync(); }
}

/// Check if fd is a tty
pub fn isatty(fd: i32) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}

/// Get terminal name
#[cfg(feature = "alloc")]
pub fn ttyname(fd: i32) -> Option<Vec<u8>> {
    let ptr = unsafe { libc::ttyname(fd) };
    if ptr.is_null() {
        None
    } else {
        let mut len = 0;
        while unsafe { *ptr.add(len) } != 0 {
            len += 1;
        }
        let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
        Some(slice.to_vec())
    }
}

/// Get environment variable
pub fn getenv(name: &[u8]) -> Option<&'static [u8]> {
    let mut name_buf = [0u8; 256];
    if name.len() >= name_buf.len() {
        return None;
    }
    name_buf[..name.len()].copy_from_slice(name);
    name_buf[name.len()] = 0;

    let ptr = unsafe { libc::getenv(name_buf.as_ptr() as *const i8) };
    if ptr.is_null() {
        None
    } else {
        let mut len = 0;
        while unsafe { *ptr.add(len) } != 0 {
            len += 1;
        }
        Some(unsafe { core::slice::from_raw_parts(ptr as *const u8, len) })
    }
}

/// Send signal to process
pub fn kill(pid: i32, sig: i32) -> i32 {
    unsafe { libc::kill(pid, sig) }
}

/// Execute program
pub fn execve(path: &[u8], argv: *const *const i8, envp: *const *const i8) -> i32 {
    let mut path_buf = [0u8; 4096];
    if path.len() >= path_buf.len() {
        return -1;
    }
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;

    unsafe { libc::execve(path_buf.as_ptr() as *const i8, argv, envp) }
}

/// Fork process
pub fn fork() -> i32 {
    unsafe { libc::fork() }
}

/// Wait for child process
pub fn wait(status: *mut i32) -> i32 {
    unsafe { libc::wait(status) }
}

/// Wait for specific process
pub fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32 {
    unsafe { libc::waitpid(pid, status, options) }
}

/// Get uname info
pub fn uname(buf: &mut libc::utsname) -> i32 {
    unsafe { libc::uname(buf) }
}

/// Exit process
pub fn exit(code: i32) -> ! {
    unsafe { libc::_exit(code); }
}

// ============================================================================
// Helper functions for argument parsing
// ============================================================================

/// Get C string length
pub fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

/// String length for array
pub fn strlen_arr(s: &[u8]) -> usize {
    s.iter().position(|&c| c == 0).unwrap_or(s.len())
}

/// Convert C string pointer to slice
pub unsafe fn cstr_to_slice(s: *const u8) -> &'static [u8] {
    let len = strlen(s);
    unsafe { core::slice::from_raw_parts(s, len) }
}

/// Compare byte slices
pub fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y)
}

/// Check if slice starts with prefix
pub fn starts_with(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && &s[..prefix.len()] == prefix
}

/// Get dirent name as u8 slice
/// On Linux, d_name is [i8; 256], we need to convert to u8
pub unsafe fn dirent_name(entry: *const libc::dirent) -> (&'static [u8], usize) {
    unsafe {
        let name_ptr = (*entry).d_name.as_ptr();
        let mut len = 0;
        while *name_ptr.add(len) != 0 && len < 255 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(name_ptr as *const u8, len);
        (slice, len)
    }
}
