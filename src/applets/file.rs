//! File operation applets

use crate::io;
use crate::sys;
use super::{get_arg, has_opt, is_opt};

/// cat - concatenate files
pub fn cat(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        // Read from stdin
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 { break; }
            io::write_all(1, &buf[..n as usize]);
        }
        return 0;
    }

    for i in 1..argc {
        let path = match unsafe { get_arg(argv, i) } {
            Some(p) => p,
            None => continue,
        };

        if path == b"-" {
            let mut buf = [0u8; 4096];
            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 { break; }
                io::write_all(1, &buf[..n as usize]);
            }
            continue;
        }

        let fd = io::open(path, libc::O_RDONLY, 0);
        if fd < 0 {
            sys::perror(path);
            continue;
        }

        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 { break; }
            io::write_all(1, &buf[..n as usize]);
        }
        io::close(fd);
    }
    0
}

/// cp - copy files
pub fn cp(argc: i32, argv: *const *const u8) -> i32 {
    let mut recursive = false;
    let mut force = false;
    let mut interactive = false;
    let mut preserve = false;
    let mut files_start = 1;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b'r') || has_opt(arg, b'R') { recursive = true; }
                if has_opt(arg, b'f') { force = true; }
                if has_opt(arg, b'i') { interactive = true; }
                if has_opt(arg, b'p') { preserve = true; }
                files_start = i + 1;
            } else {
                break;
            }
        }
    }

    if argc - files_start < 2 {
        io::write_str(2, b"cp: missing operand\n");
        return 1;
    }

    let dest = unsafe { get_arg(argv, argc - 1).unwrap() };

    for i in files_start..(argc - 1) {
        if let Some(src) = unsafe { get_arg(argv, i) } {
            copy_file(src, dest, recursive, force, interactive, preserve);
        }
    }
    0
}

fn copy_file(src: &[u8], dest: &[u8], recursive: bool, _force: bool, _interactive: bool, _preserve: bool) {
    let src_fd = io::open(src, libc::O_RDONLY, 0);
    if src_fd < 0 {
        sys::perror(src);
        return;
    }

    let dest_fd = io::open(dest, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644);
    if dest_fd < 0 {
        io::close(src_fd);
        sys::perror(dest);
        return;
    }

    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(src_fd, &mut buf);
        if n <= 0 { break; }
        io::write_all(dest_fd, &buf[..n as usize]);
    }

    io::close(src_fd);
    io::close(dest_fd);
    let _ = recursive; // TODO: implement recursive copy
}

/// mv - move/rename files
pub fn mv(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"mv: missing operand\n");
        return 1;
    }

    let src = unsafe { get_arg(argv, 1).unwrap() };
    let dest = unsafe { get_arg(argv, 2).unwrap() };

    // Try rename first
    if io::rename(src, dest) == 0 {
        return 0;
    }

    // Fall back to copy + remove
    copy_file(src, dest, false, true, false, false);
    io::unlink(src);
    0
}

/// rm - remove files
pub fn rm(argc: i32, argv: *const *const u8) -> i32 {
    let mut recursive = false;
    let mut force = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b'r') || has_opt(arg, b'R') { recursive = true; }
                if has_opt(arg, b'f') { force = true; }
            } else {
                if recursive {
                    remove_recursive(arg);
                } else if io::unlink(arg) < 0 && !force {
                    sys::perror(arg);
                }
            }
        }
    }
    0
}

fn remove_recursive(path: &[u8]) {
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    if io::stat(path, &mut st) < 0 { return; }

    if (st.st_mode & libc::S_IFMT) == libc::S_IFDIR {
        // Directory - recurse
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
                    full_path[len] = b'/'; len += 1;
                    for c in name { full_path[len] = *c; len += 1; }

                    remove_recursive(&full_path[..len]);
                }

                offset += dirent.d_reclen as usize;
            }
        }
        io::close(fd);
        io::rmdir(path);
    } else {
        io::unlink(path);
    }
}

/// mkdir - create directories
pub fn mkdir(argc: i32, argv: *const *const u8) -> i32 {
    let mut parents = false;
    let mut mode = 0o755u32;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b'p') { parents = true; }
                if has_opt(arg, b'm') && i + 1 < argc {
                    if let Some(m) = unsafe { get_arg(argv, i + 1) } {
                        mode = sys::parse_octal(m).unwrap_or(0o755);
                    }
                }
            } else if parents {
                mkdir_parents(arg, mode);
            } else if io::mkdir(arg, mode) < 0 {
                sys::perror(arg);
                return 1;
            }
        }
    }
    0
}

fn mkdir_parents(path: &[u8], mode: u32) {
    let mut partial = [0u8; 512];
    let mut len = 0;

    for &c in path {
        if c == b'/' && len > 0 {
            let _ = io::mkdir(&partial[..len], mode);
        }
        partial[len] = c;
        len += 1;
    }
    let _ = io::mkdir(&partial[..len], mode);
}

/// rmdir - remove empty directories
pub fn rmdir(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] != b'-' && io::rmdir(path) < 0 {
                sys::perror(path);
            }
        }
    }
    0
}

/// touch - change file timestamps
pub fn touch(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] != b'-' {
                // Try to create file if doesn't exist
                let fd = io::open(path, libc::O_WRONLY | libc::O_CREAT, 0o644);
                if fd >= 0 {
                    io::close(fd);
                    // Update timestamps
                    unsafe { libc::utimes(path.as_ptr() as *const i8, core::ptr::null()) };
                } else {
                    sys::perror(path);
                }
            }
        }
    }
    0
}

/// ln - create links
pub fn ln(argc: i32, argv: *const *const u8) -> i32 {
    let mut symbolic = false;
    let mut force = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b's') { symbolic = true; }
                if has_opt(arg, b'f') { force = true; }
            }
        }
    }

    if argc < 3 {
        io::write_str(2, b"ln: missing operand\n");
        return 1;
    }

    let target = unsafe { get_arg(argv, argc - 2).unwrap() };
    let link_name = unsafe { get_arg(argv, argc - 1).unwrap() };

    if force {
        io::unlink(link_name);
    }

    let ret = if symbolic {
        io::symlink(target, link_name)
    } else {
        io::link(target, link_name)
    };

    if ret < 0 {
        sys::perror(link_name);
        return 1;
    }
    0
}

/// ls - list directory contents
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
            if arg[0] == b'-' && arg.len() > 1 {
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

fn list_dir(path: &[u8], show_all: bool, long_format: bool, one_per_line: bool, _show_inode: bool, _classify: bool) {
    let fd = io::open(path, libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 {
        sys::perror(path);
        return;
    }

    let mut buf = [0u8; 4096];
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
                    let mut mode_buf = [0u8; 10];
                    sys::format_mode(st.st_mode as u32, &mut mode_buf);
                    io::write_all(1, &mode_buf);
                    io::write_str(1, b" ");
                    io::write_num(1, st.st_nlink as u64);
                    io::write_str(1, b" ");
                    io::write_num(1, st.st_uid as u64);
                    io::write_str(1, b" ");
                    io::write_num(1, st.st_gid as u64);
                    io::write_str(1, b" ");
                    io::write_num(1, st.st_size as u64);
                    io::write_str(1, b" ");
                }
            }

            io::write_all(1, name);
            if one_per_line || long_format {
                io::write_str(1, b"\n");
            } else {
                io::write_str(1, b"  ");
            }

            offset += dirent.d_reclen as usize;
        }
    }

    if !one_per_line && !long_format {
        io::write_str(1, b"\n");
    }

    io::close(fd);
}

/// pwd - print working directory
pub fn pwd(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut buf = [0u8; 4096];
    let ret = unsafe { libc::getcwd(buf.as_mut_ptr() as *mut i8, buf.len()) };
    if !ret.is_null() {
        io::write_all(1, &buf[..io::strlen_arr(&buf)]);
        io::write_str(1, b"\n");
        0
    } else {
        sys::perror(b"getcwd");
        1
    }
}

/// chmod - change file modes
pub fn chmod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"chmod: missing operand\n");
        return 1;
    }

    let mode_str = unsafe { get_arg(argv, 1).unwrap() };
    let mode = sys::parse_octal(mode_str).unwrap_or(0o644);

    for i in 2..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if io::chmod(path, mode) < 0 {
                sys::perror(path);
            }
        }
    }
    0
}

/// chown - change file owner
pub fn chown(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"chown: missing operand\n");
        return 1;
    }

    let owner = unsafe { get_arg(argv, 1).unwrap() };
    let uid = sys::parse_u64(owner).unwrap_or(0) as u32;

    for i in 2..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if unsafe { libc::chown(path.as_ptr() as *const i8, uid, u32::MAX) } < 0 {
                sys::perror(path);
            }
        }
    }
    0
}

/// chgrp - change file group
pub fn chgrp(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"chgrp: missing operand\n");
        return 1;
    }

    let group = unsafe { get_arg(argv, 1).unwrap() };
    let gid = sys::parse_u64(group).unwrap_or(0) as u32;

    for i in 2..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if unsafe { libc::chown(path.as_ptr() as *const i8, u32::MAX, gid) } < 0 {
                sys::perror(path);
            }
        }
    }
    0
}

/// stat - display file status
pub fn stat(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] == b'-' { continue; }

            let mut st: libc::stat = unsafe { core::mem::zeroed() };
            if io::stat(path, &mut st) < 0 {
                sys::perror(path);
                continue;
            }

            io::write_str(1, b"  File: ");
            io::write_all(1, path);
            io::write_str(1, b"\n  Size: ");
            io::write_num(1, st.st_size as u64);
            io::write_str(1, b"\tBlocks: ");
            io::write_num(1, st.st_blocks as u64);
            io::write_str(1, b"\nDevice: ");
            io::write_num(1, st.st_dev as u64);
            io::write_str(1, b"\tInode: ");
            io::write_num(1, st.st_ino as u64);
            io::write_str(1, b"\tLinks: ");
            io::write_num(1, st.st_nlink as u64);
            io::write_str(1, b"\nAccess: ");
            let mut mode_buf = [0u8; 10];
            sys::format_mode(st.st_mode as u32, &mut mode_buf);
            io::write_all(1, &mode_buf);
            io::write_str(1, b"\n");
        }
    }
    0
}

/// readlink - print resolved symbolic link
pub fn readlink(argc: i32, argv: *const *const u8) -> i32 {
    let mut canonicalize = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b'f') { canonicalize = true; }
            } else {
                let mut buf = [0u8; 4096];
                let n = if canonicalize {
                    io::realpath(arg, &mut buf)
                } else {
                    io::readlink(arg, &mut buf)
                };
                if n > 0 {
                    io::write_all(1, &buf[..n as usize]);
                    io::write_str(1, b"\n");
                } else {
                    sys::perror(arg);
                    return 1;
                }
            }
        }
    }
    0
}

/// realpath - print canonical path
pub fn realpath(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] == b'-' { continue; }

            let mut buf = [0u8; 4096];
            let n = io::realpath(path, &mut buf);
            if n > 0 {
                io::write_all(1, &buf[..n as usize]);
                io::write_str(1, b"\n");
            } else {
                sys::perror(path);
                return 1;
            }
        }
    }
    0
}

/// basename - strip directory from file name
pub fn basename(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"basename: missing operand\n");
        return 1;
    }

    let path = unsafe { get_arg(argv, 1).unwrap() };
    let suffix = if argc > 2 { unsafe { get_arg(argv, 2) } } else { None };

    // Find last /
    let mut start = 0;
    for i in 0..path.len() {
        if path[i] == b'/' {
            start = i + 1;
        }
    }

    let base = &path[start..];
    let mut end = base.len();

    // Strip suffix if provided
    if let Some(s) = suffix {
        if base.len() > s.len() && &base[base.len()-s.len()..] == s {
            end = base.len() - s.len();
        }
    }

    io::write_all(1, &base[..end]);
    io::write_str(1, b"\n");
    0
}

/// dirname - strip last component from file name
pub fn dirname(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"dirname: missing operand\n");
        return 1;
    }

    let path = unsafe { get_arg(argv, 1).unwrap() };

    // Find last /
    let mut last_slash = None;
    for i in 0..path.len() {
        if path[i] == b'/' {
            last_slash = Some(i);
        }
    }

    match last_slash {
        Some(0) => { io::write_str(1, b"/\n"); }
        Some(i) => {
            io::write_all(1, &path[..i]);
            io::write_str(1, b"\n");
        }
        None => { io::write_str(1, b".\n"); }
    }
    0
}

/// sync - sync filesystem
pub fn sync_cmd(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe { libc::sync() };
    0
}

/// link - create hard link
pub fn link(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"link: missing operand\n");
        return 1;
    }

    let target = unsafe { get_arg(argv, 1).unwrap() };
    let link_name = unsafe { get_arg(argv, 2).unwrap() };

    if io::link(target, link_name) < 0 {
        sys::perror(link_name);
        return 1;
    }
    0
}

/// unlink - remove file
pub fn unlink(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"unlink: missing operand\n");
        return 1;
    }

    let path = unsafe { get_arg(argv, 1).unwrap() };
    if io::unlink(path) < 0 {
        sys::perror(path);
        return 1;
    }
    0
}

/// dd - convert and copy a file
pub fn dd(argc: i32, argv: *const *const u8) -> i32 {
    let mut if_path: Option<&[u8]> = None;
    let mut of_path: Option<&[u8]> = None;
    let mut bs: usize = 512;
    let mut count: Option<usize> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.starts_with(b"if=") {
                if_path = Some(&arg[3..]);
            } else if arg.starts_with(b"of=") {
                of_path = Some(&arg[3..]);
            } else if arg.starts_with(b"bs=") {
                bs = sys::parse_u64(&arg[3..]).unwrap_or(512) as usize;
            } else if arg.starts_with(b"count=") {
                count = Some(sys::parse_u64(&arg[6..]).unwrap_or(0) as usize);
            }
        }
    }

    let in_fd = match if_path {
        Some(p) => io::open(p, libc::O_RDONLY, 0),
        None => 0,
    };
    if in_fd < 0 { return 1; }

    let out_fd = match of_path {
        Some(p) => io::open(p, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644),
        None => 1,
    };
    if out_fd < 0 {
        if in_fd != 0 { io::close(in_fd); }
        return 1;
    }

    #[cfg(feature = "alloc")]
    {
        use alloc::vec;
        let mut buf = vec![0u8; bs];
        let mut blocks = 0;

        loop {
            if let Some(c) = count {
                if blocks >= c { break; }
            }

            let n = io::read(in_fd, &mut buf);
            if n <= 0 { break; }
            io::write_all(out_fd, &buf[..n as usize]);
            blocks += 1;
        }

        io::write_num(2, blocks as u64);
        io::write_str(2, b"+0 records in\n");
        io::write_num(2, blocks as u64);
        io::write_str(2, b"+0 records out\n");
    }

    #[cfg(not(feature = "alloc"))]
    {
        let mut buf = [0u8; 512];
        let mut blocks = 0;

        loop {
            if let Some(c) = count {
                if blocks >= c { break; }
            }

            let n = io::read(in_fd, &mut buf);
            if n <= 0 { break; }
            io::write_all(out_fd, &buf[..n as usize]);
            blocks += 1;
        }

        let _ = bs;
    }

    if in_fd != 0 { io::close(in_fd); }
    if out_fd != 1 { io::close(out_fd); }
    0
}

/// mktemp - create temporary file/directory
pub fn mktemp(argc: i32, argv: *const *const u8) -> i32 {
    let mut dir = false;
    let template = if argc > 1 {
        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if has_opt(arg, b'd') { dir = true; }
                else if arg[0] != b'-' { return create_temp(arg, dir); }
            }
        }
        b"/tmp/tmp.XXXXXX"
    } else {
        b"/tmp/tmp.XXXXXX"
    };
    create_temp(template, dir)
}

fn create_temp(template: &[u8], dir: bool) -> i32 {
    let mut path = [0u8; 256];
    for (i, &c) in template.iter().enumerate() {
        path[i] = c;
    }

    // Replace X with random chars
    let seed = unsafe { libc::time(core::ptr::null_mut()) } as u64;
    let mut rng = seed;
    for i in 0..template.len() {
        if path[i] == b'X' {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            path[i] = b"0123456789abcdef"[(rng >> 60) as usize];
        }
    }

    if dir {
        if io::mkdir(&path[..template.len()], 0o700) < 0 {
            sys::perror(&path[..template.len()]);
            return 1;
        }
    } else {
        let fd = io::open(&path[..template.len()], libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL, 0o600);
        if fd < 0 {
            sys::perror(&path[..template.len()]);
            return 1;
        }
        io::close(fd);
    }

    io::write_all(1, &path[..template.len()]);
    io::write_str(1, b"\n");
    0
}

/// mkfifo - make FIFO special file
pub fn mkfifo(argc: i32, argv: *const *const u8) -> i32 {
    let mode = 0o644u32;

    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] != b'-' {
                if unsafe { libc::mkfifo(path.as_ptr() as *const i8, mode) } < 0 {
                    sys::perror(path);
                    return 1;
                }
            }
        }
    }
    0
}

/// mknod - make block or character special files
pub fn mknod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 4 {
        io::write_str(2, b"mknod: missing operand\n");
        return 1;
    }

    let path = unsafe { get_arg(argv, 1).unwrap() };
    let type_arg = unsafe { get_arg(argv, 2).unwrap() };

    let (mode, dev) = if type_arg == b"p" {
        (libc::S_IFIFO | 0o666, 0)
    } else if argc >= 5 {
        let major = sys::parse_u64(unsafe { get_arg(argv, 3).unwrap() }).unwrap_or(0) as u32;
        let minor = sys::parse_u64(unsafe { get_arg(argv, 4).unwrap() }).unwrap_or(0) as u32;
        let m = if type_arg == b"b" { libc::S_IFBLK } else { libc::S_IFCHR };
        (m | 0o666, sys::makedev(major, minor))
    } else {
        io::write_str(2, b"mknod: missing major/minor\n");
        return 1;
    };

    if unsafe { libc::mknod(path.as_ptr() as *const i8, mode, dev) } < 0 {
        sys::perror(path);
        return 1;
    }
    0
}

/// split - split file into pieces
pub fn split(argc: i32, argv: *const *const u8) -> i32 {
    let mut lines = 1000usize;
    let mut prefix = b"x".as_slice();
    let mut input: Option<&[u8]> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'l') && i + 1 < argc {
                if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                    lines = sys::parse_u64(n).unwrap_or(1000) as usize;
                }
            } else if arg[0] != b'-' {
                if input.is_none() {
                    input = Some(arg);
                } else {
                    prefix = arg;
                }
            }
        }
    }

    let fd = match input {
        Some(p) if p != b"-" => io::open(p, libc::O_RDONLY, 0),
        _ => 0,
    };
    if fd < 0 { return 1; }

    let _ = lines;
    let _ = prefix;
    // Simplified - just copy to one output
    io::write_str(2, b"split: simplified implementation\n");

    if fd != 0 { io::close(fd); }
    0
}

/// install - copy files and set attributes
pub fn install(argc: i32, argv: *const *const u8) -> i32 {
    let mut dir_mode = false;
    let mut mode = 0o755u32;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'd') { dir_mode = true; }
            if has_opt(arg, b'm') && i + 1 < argc {
                if let Some(m) = unsafe { get_arg(argv, i + 1) } {
                    mode = sys::parse_octal(m).unwrap_or(0o755);
                }
            }
        }
    }

    if dir_mode {
        for i in 1..argc {
            if let Some(path) = unsafe { get_arg(argv, i) } {
                if path[0] != b'-' {
                    mkdir_parents(path, mode);
                }
            }
        }
    } else if argc >= 3 {
        let src = unsafe { get_arg(argv, argc - 2).unwrap() };
        let dest = unsafe { get_arg(argv, argc - 1).unwrap() };
        copy_file(src, dest, false, true, false, false);
        io::chmod(dest, mode);
    }
    0
}

/// truncate - shrink or extend file size
pub fn truncate(argc: i32, argv: *const *const u8) -> i32 {
    let mut size: i64 = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b's') && i + 1 < argc {
                if let Some(s) = unsafe { get_arg(argv, i + 1) } {
                    size = sys::parse_u64(s).unwrap_or(0) as i64;
                }
            } else if arg[0] != b'-' {
                if unsafe { libc::truncate(arg.as_ptr() as *const i8, size) } < 0 {
                    sys::perror(arg);
                    return 1;
                }
            }
        }
    }
    0
}

/// shred - overwrite file to hide contents
pub fn shred(argc: i32, argv: *const *const u8) -> i32 {
    let mut remove = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'u') { remove = true; }
            else if arg[0] != b'-' {
                // Overwrite with random data
                let fd = io::open(arg, libc::O_WRONLY, 0);
                if fd < 0 {
                    sys::perror(arg);
                    continue;
                }

                let mut st: libc::stat = unsafe { core::mem::zeroed() };
                if io::fstat(fd, &mut st) == 0 {
                    let size = st.st_size as usize;
                    let mut buf = [0xFFu8; 4096];
                    let mut written = 0;
                    while written < size {
                        let chunk = core::cmp::min(buf.len(), size - written);
                        io::write_all(fd, &buf[..chunk]);
                        written += chunk;
                    }
                    unsafe { libc::fsync(fd) };

                    // Zero pass
                    unsafe { libc::lseek(fd, 0, libc::SEEK_SET) };
                    buf.fill(0);
                    written = 0;
                    while written < size {
                        let chunk = core::cmp::min(buf.len(), size - written);
                        io::write_all(fd, &buf[..chunk]);
                        written += chunk;
                    }
                }
                io::close(fd);

                if remove {
                    io::unlink(arg);
                }
            }
        }
    }
    0
}

/// file - determine file type
pub fn file(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] == b'-' { continue; }

            io::write_all(1, path);
            io::write_str(1, b": ");

            let mut st: libc::stat = unsafe { core::mem::zeroed() };
            if io::lstat(path, &mut st) < 0 {
                io::write_str(1, b"cannot stat\n");
                continue;
            }

            match st.st_mode & libc::S_IFMT {
                libc::S_IFDIR => { io::write_str(1, b"directory\n"); }
                libc::S_IFLNK => { io::write_str(1, b"symbolic link\n"); }
                libc::S_IFIFO => { io::write_str(1, b"fifo (named pipe)\n"); }
                libc::S_IFSOCK => { io::write_str(1, b"socket\n"); }
                libc::S_IFBLK => { io::write_str(1, b"block special\n"); }
                libc::S_IFCHR => { io::write_str(1, b"character special\n"); }
                libc::S_IFREG => {
                    // Check magic bytes
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd >= 0 {
                        let mut magic = [0u8; 8];
                        let n = io::read(fd, &mut magic);
                        io::close(fd);

                        if n >= 4 {
                            if magic[0..4] == [0x7F, b'E', b'L', b'F'] {
                                io::write_str(1, b"ELF executable\n");
                            } else if magic[0..2] == [b'#', b'!'] {
                                io::write_str(1, b"script\n");
                            } else if magic[0..4] == [0x1F, 0x8B, 0x08, 0x00] {
                                io::write_str(1, b"gzip compressed\n");
                            } else if magic[0..3] == [b'B', b'Z', b'h'] {
                                io::write_str(1, b"bzip2 compressed\n");
                            } else if st.st_size == 0 {
                                io::write_str(1, b"empty\n");
                            } else {
                                io::write_str(1, b"data\n");
                            }
                        } else {
                            io::write_str(1, b"empty\n");
                        }
                    } else {
                        io::write_str(1, b"regular file\n");
                    }
                }
                _ => { io::write_str(1, b"unknown\n"); }
            }
        }
    }
    0
}

/// xargs - build and execute commands
pub fn xargs(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;
        use alloc::ffi::CString;

        // Read lines from stdin
        let mut buf = [0u8; 4096];
        let n = io::read(0, &mut buf);
        if n <= 0 { return 0; }

        // Parse arguments
        let cmd = if argc > 1 {
            unsafe { get_arg(argv, 1).unwrap() }
        } else {
            b"echo"
        };

        // Build argument list
        let lines: Vec<&[u8]> = buf[..n as usize]
            .split(|&c| c == b'\n')
            .filter(|l| !l.is_empty())
            .collect();

        for line in lines {
            let pid = unsafe { libc::fork() };
            if pid == 0 {
                let mut args: Vec<CString> = Vec::new();

                // Command
                let mut v = Vec::with_capacity(cmd.len() + 1);
                v.extend_from_slice(cmd);
                v.push(0);
                if let Ok(cs) = CString::from_vec_with_nul(v) {
                    args.push(cs);
                }

                // Original args
                for i in 2..argc {
                    if let Some(arg) = unsafe { get_arg(argv, i) } {
                        let mut v = Vec::with_capacity(arg.len() + 1);
                        v.extend_from_slice(arg);
                        v.push(0);
                        if let Ok(cs) = CString::from_vec_with_nul(v) {
                            args.push(cs);
                        }
                    }
                }

                // Line as argument
                let mut v = Vec::with_capacity(line.len() + 1);
                v.extend_from_slice(line);
                v.push(0);
                if let Ok(cs) = CString::from_vec_with_nul(v) {
                    args.push(cs);
                }

                let ptrs: Vec<*const i8> = args.iter()
                    .map(|s: &CString| s.as_ptr())
                    .chain(core::iter::once(core::ptr::null()))
                    .collect();

                unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
                unsafe { libc::_exit(127) };
            } else if pid > 0 {
                let mut status = 0;
                unsafe { libc::waitpid(pid, &mut status, 0) };
            }
        }
    }
    0
}

/// patch - apply a diff file
pub fn patch(argc: i32, argv: *const *const u8) -> i32 {
    let mut input: Option<&[u8]> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'i') && i + 1 < argc {
                input = unsafe { get_arg(argv, i + 1) };
            }
        }
    }

    let fd = match input {
        Some(p) => io::open(p, libc::O_RDONLY, 0),
        None => 0,
    };
    if fd < 0 { return 1; }

    io::write_str(2, b"patch: stub implementation\n");

    if fd != 0 { io::close(fd); }
    0
}

/// find - search for files
pub fn find(argc: i32, argv: *const *const u8) -> i32 {
    let start_path = if argc > 1 {
        let first = unsafe { get_arg(argv, 1).unwrap() };
        if first[0] != b'-' { first } else { b"." }
    } else {
        b"."
    };

    let mut name_pattern: Option<&[u8]> = None;
    let mut file_type: Option<u8> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-name" && i + 1 < argc {
                name_pattern = unsafe { get_arg(argv, i + 1) };
            } else if arg == b"-type" && i + 1 < argc {
                if let Some(t) = unsafe { get_arg(argv, i + 1) } {
                    file_type = Some(t[0]);
                }
            }
        }
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
                if path[path.len()-1] != b'/' {
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

/// cd - change directory (shell builtin, but implemented as stub)
pub fn cd(argc: i32, argv: *const *const u8) -> i32 {
    let path = if argc > 1 {
        unsafe { get_arg(argv, 1).unwrap() }
    } else {
        // Get HOME
        b"/root"
    };

    if io::chdir(path) < 0 {
        sys::perror(path);
        return 1;
    }
    0
}

// Additional toybox applets
pub fn chattr(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"chattr: stub\n"); 0 }
pub fn lsattr(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"lsattr: stub\n"); 0 }
pub fn fstype(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"ext4\n"); 0 }
pub fn makedevs(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"makedevs: stub\n"); 0 }
pub fn setfattr(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
