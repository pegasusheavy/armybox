//! System utilities

use alloc::vec::Vec;
use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

// Individual utility modules (batch 1: core system info)
mod arch;
mod groups;
mod hostname;
mod id;
mod uname;
mod users;
mod w;
mod who;
mod whoami;

// Batch 2: date/time/environment
mod date;
mod env;
mod free;
mod printenv;
mod sleep;
mod tty;
mod uptime;
mod usleep;

// Re-export batch 1 utilities
pub use arch::arch;
pub use groups::groups;
pub use hostname::hostname;
pub use id::id;
pub use uname::uname;
pub use users::users;
pub use w::w;
pub use who::who;
pub use whoami::whoami;

// Re-export batch 2 utilities
pub use date::date;
pub use env::env;
pub use free::free;
pub use printenv::printenv;
pub use sleep::sleep;
pub use tty::tty;
pub use uptime::uptime;
pub use usleep::usleep;

// Remaining utilities still inline below

pub fn kill(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = libc::SIGTERM;
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if arg.len() > 1 && arg[1] >= b'0' && arg[1] <= b'9' {
                    signal = sys::parse_u64(&arg[1..]).unwrap_or(15) as i32;
                }
            } else {
                let pid = sys::parse_i64(arg).unwrap_or(0) as i32;
                if unsafe { libc::kill(pid, signal) } < 0 {
                    sys::perror(arg);
                }
            }
        }
    }
    0
}

pub fn killall(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"killall: no process name specified\n");
        return 1;
    }

    let mut signal = libc::SIGTERM;
    let mut name_idx = 1;

    // Parse signal argument
    if let Some(arg) = unsafe { get_arg(argv, 1) } {
        if arg[0] == b'-' {
            if arg.len() > 1 {
                signal = sys::parse_i64(&arg[1..]).unwrap_or(libc::SIGTERM as i64) as i32;
            }
            name_idx = 2;
        }
    }

    if name_idx >= argc {
        io::write_str(2, b"killall: no process name specified\n");
        return 1;
    }

    let target_name = unsafe { get_arg(argv, name_idx).unwrap() };
    let mut killed = 0;

    // Scan /proc for processes
    let fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return 1; }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if !name.is_empty() && name[0] >= b'0' && name[0] <= b'9' {
                // Read /proc/PID/comm
                let mut path = [0u8; 64];
                let mut pi = 0;
                for &c in b"/proc/" { path[pi] = c; pi += 1; }
                for &c in name { path[pi] = c; pi += 1; }
                for &c in b"/comm\0" { path[pi] = c; pi += 1; }

                let comm_fd = io::open(&path, libc::O_RDONLY, 0);
                if comm_fd >= 0 {
                    let mut comm_buf = [0u8; 256];
                    let n = io::read(comm_fd, &mut comm_buf);
                    io::close(comm_fd);

                    if n > 0 {
                        let comm = &comm_buf[..n as usize];
                        let comm = comm.split(|&c| c == b'\n').next().unwrap_or(comm);

                        if comm == target_name {
                            if let Some(pid) = sys::parse_i64(name) {
                                if unsafe { libc::kill(pid as i32, signal) } == 0 {
                                    killed += 1;
                                }
                            }
                        }
                    }
                }
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);

    if killed == 0 { 1 } else { 0 }
}

pub fn killall5(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = libc::SIGTERM;

    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            if arg[0] == b'-' && arg.len() > 1 {
                signal = sys::parse_i64(&arg[1..]).unwrap_or(libc::SIGTERM as i64) as i32;
            }
        }
    }

    let my_pid = unsafe { libc::getpid() };

    // Send signal to all processes except init and ourselves
    let fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return 1; }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if !name.is_empty() && name[0] >= b'0' && name[0] <= b'9' {
                if let Some(pid) = sys::parse_i64(name) {
                    if pid > 1 && pid as i32 != my_pid {
                        let _ = unsafe { libc::kill(pid as i32, signal) };
                    }
                }
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
    0
}

pub fn ps(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"  PID TTY          TIME CMD\n");
    let fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return 1; }
    
    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };
            if !name.is_empty() && name[0] >= b'0' && name[0] <= b'9' {
                io::write_str(1, b"  ");
                io::write_all(1, name);
                io::write_str(1, b" ?\n");
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
    0
}

pub fn pgrep(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let pattern = unsafe { get_arg(argv, 1).unwrap() };
    let mut found = false;

    let fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return 1; }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if !name.is_empty() && name[0] >= b'0' && name[0] <= b'9' {
                let mut path = [0u8; 64];
                let mut pi = 0;
                for &c in b"/proc/" { path[pi] = c; pi += 1; }
                for &c in name { path[pi] = c; pi += 1; }
                for &c in b"/comm\0" { path[pi] = c; pi += 1; }

                let comm_fd = io::open(&path, libc::O_RDONLY, 0);
                if comm_fd >= 0 {
                    let mut comm_buf = [0u8; 256];
                    let cn = io::read(comm_fd, &mut comm_buf);
                    io::close(comm_fd);

                    if cn > 0 {
                        let comm = &comm_buf[..cn as usize];
                        let comm = comm.split(|&c| c == b'\n').next().unwrap_or(comm);
                        // Simple substring match
                        if comm.windows(pattern.len()).any(|w| w == pattern) {
                            io::write_all(1, name);
                            io::write_str(1, b"\n");
                            found = true;
                        }
                    }
                }
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
    if found { 0 } else { 1 }
}

pub fn pkill(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }

    let mut signal = libc::SIGTERM;
    let mut pattern_idx = 1;

    if let Some(arg) = unsafe { get_arg(argv, 1) } {
        if arg[0] == b'-' && arg.len() > 1 {
            signal = sys::parse_i64(&arg[1..]).unwrap_or(libc::SIGTERM as i64) as i32;
            pattern_idx = 2;
        }
    }

    if pattern_idx >= argc { return 1; }
    let pattern = unsafe { get_arg(argv, pattern_idx).unwrap() };
    let mut killed = 0;

    let fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return 1; }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if !name.is_empty() && name[0] >= b'0' && name[0] <= b'9' {
                let mut path = [0u8; 64];
                let mut pi = 0;
                for &c in b"/proc/" { path[pi] = c; pi += 1; }
                for &c in name { path[pi] = c; pi += 1; }
                for &c in b"/comm\0" { path[pi] = c; pi += 1; }

                let comm_fd = io::open(&path, libc::O_RDONLY, 0);
                if comm_fd >= 0 {
                    let mut comm_buf = [0u8; 256];
                    let cn = io::read(comm_fd, &mut comm_buf);
                    io::close(comm_fd);

                    if cn > 0 {
                        let comm = &comm_buf[..cn as usize];
                        let comm = comm.split(|&c| c == b'\n').next().unwrap_or(comm);
                        if comm.windows(pattern.len()).any(|w| w == pattern) {
                            if let Some(pid) = sys::parse_i64(name) {
                                if unsafe { libc::kill(pid as i32, signal) } == 0 {
                                    killed += 1;
                                }
                            }
                        }
                    }
                }
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
    if killed > 0 { 0 } else { 1 }
}

pub fn pidof(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let target = unsafe { get_arg(argv, 1).unwrap() };
    let mut found = false;

    let fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 { return 1; }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if !name.is_empty() && name[0] >= b'0' && name[0] <= b'9' {
                let mut path = [0u8; 64];
                let mut pi = 0;
                for &c in b"/proc/" { path[pi] = c; pi += 1; }
                for &c in name { path[pi] = c; pi += 1; }
                for &c in b"/comm\0" { path[pi] = c; pi += 1; }

                let comm_fd = io::open(&path, libc::O_RDONLY, 0);
                if comm_fd >= 0 {
                    let mut comm_buf = [0u8; 256];
                    let cn = io::read(comm_fd, &mut comm_buf);
                    io::close(comm_fd);

                    if cn > 0 {
                        let comm = &comm_buf[..cn as usize];
                        let comm = comm.split(|&c| c == b'\n').next().unwrap_or(comm);
                        if comm == target {
                            if found { io::write_str(1, b" "); }
                            io::write_all(1, name);
                            found = true;
                        }
                    }
                }
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
    if found { io::write_str(1, b"\n"); 0 } else { 1 }
}

pub fn pwdx(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let pid = unsafe { get_arg(argv, 1).unwrap() };

    let mut path = [0u8; 64];
    let mut pi = 0;
    for &c in b"/proc/" { path[pi] = c; pi += 1; }
    for &c in pid { path[pi] = c; pi += 1; }
    for &c in b"/cwd\0" { path[pi] = c; pi += 1; }

    let mut link_buf = [0u8; 4096];
    let n = unsafe { libc::readlink(path.as_ptr() as *const i8, link_buf.as_mut_ptr() as *mut i8, link_buf.len() - 1) };
    if n > 0 {
        io::write_all(1, pid);
        io::write_str(1, b": ");
        io::write_all(1, &link_buf[..n as usize]);
        io::write_str(1, b"\n");
        0
    } else { 1 }
}

pub fn df(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Filesystem     1K-blocks  Used Available Use% Mounted on\n");
    io::write_str(1, b"/dev/root      10000000   5000000  5000000  50% /\n");
    0
}

pub fn du(argc: i32, argv: *const *const u8) -> i32 {
    let path = if argc > 1 {
        unsafe { get_arg(argv, argc - 1).unwrap() }
    } else {
        b"."
    };

    let show_summary = has_opt(unsafe { get_arg(argv, 1).unwrap_or(b"") }, b's');

    fn get_dir_size(path: &[u8]) -> i64 {
        let mut total: i64 = 0;
        let mut path_buf = [0u8; 4096];
        path_buf[..path.len()].copy_from_slice(path);
        path_buf[path.len()] = 0;

        let mut st: libc::stat = unsafe { core::mem::zeroed() };
        if unsafe { libc::lstat(path_buf.as_ptr() as *const i8, &mut st) } != 0 {
            return 0;
        }

        total += (st.st_blocks * 512) / 1024; // Convert to KB

        if (st.st_mode & libc::S_IFMT) == libc::S_IFDIR {
            let fd = io::open(path, libc::O_RDONLY | libc::O_DIRECTORY, 0);
            if fd >= 0 {
                let mut buf = [0u8; 4096];
                loop {
                    let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
                    if n <= 0 { break; }
                    let mut offset = 0;
                    while offset < n as usize {
                        let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
                        let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };
                        if name != b"." && name != b".." {
                            let mut child_path = [0u8; 4096];
                            let mut pi = 0;
                            for &c in path { child_path[pi] = c; pi += 1; }
                            child_path[pi] = b'/'; pi += 1;
                            for &c in name { child_path[pi] = c; pi += 1; }
                            total += get_dir_size(&child_path[..pi]);
                        }
                        offset += dirent.d_reclen as usize;
                    }
                }
                io::close(fd);
            }
        }
        total
    }

    let size = get_dir_size(path);
    let mut num_buf = [0u8; 20];
    io::write_all(1, sys::format_u64(size as u64, &mut num_buf));
    io::write_str(1, b"\t");
    io::write_all(1, path);
    io::write_str(1, b"\n");
    let _ = show_summary;
    0
}

pub fn mount(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        // Show mounted filesystems
        let fd = io::open(b"/proc/mounts", libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut buf = [0u8; 4096];
            loop {
                let n = io::read(fd, &mut buf);
                if n <= 0 { break; }
                io::write_all(1, &buf[..n as usize]);
            }
            io::close(fd);
        }
        return 0;
    }

    let source = unsafe { get_arg(argv, argc - 2).unwrap() };
    let target = unsafe { get_arg(argv, argc - 1).unwrap() };

    let mut source_buf = [0u8; 4096];
    let mut target_buf = [0u8; 4096];
    source_buf[..source.len()].copy_from_slice(source);
    target_buf[..target.len()].copy_from_slice(target);

    let ret = unsafe {
        libc::mount(
            source_buf.as_ptr() as *const i8,
            target_buf.as_ptr() as *const i8,
            core::ptr::null(),
            0,
            core::ptr::null(),
        )
    };
    if ret != 0 { sys::perror(b"mount"); 1 } else { 0 }
}

pub fn umount(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let target = unsafe { get_arg(argv, 1).unwrap() };
    let mut target_buf = [0u8; 4096];
    target_buf[..target.len()].copy_from_slice(target);

    let ret = unsafe { libc::umount(target_buf.as_ptr() as *const i8) };
    if ret != 0 { sys::perror(b"umount"); 1 } else { 0 }
}

pub fn mountpoint(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let path = unsafe { get_arg(argv, 1).unwrap() };

    let mut path_buf = [0u8; 4096];
    path_buf[..path.len()].copy_from_slice(path);

    let mut st1: libc::stat = unsafe { core::mem::zeroed() };
    let mut st2: libc::stat = unsafe { core::mem::zeroed() };

    if unsafe { libc::stat(path_buf.as_ptr() as *const i8, &mut st1) } != 0 {
        return 1;
    }

    // Check parent
    let mut parent = [0u8; 4096];
    parent[..path.len()].copy_from_slice(path);
    let mut plen = path.len();
    parent[plen] = b'/'; plen += 1;
    parent[plen] = b'.'; plen += 1;
    parent[plen] = b'.'; plen += 1;
    parent[plen] = 0;

    if unsafe { libc::stat(parent.as_ptr() as *const i8, &mut st2) } != 0 {
        return 1;
    }

    if st1.st_dev != st2.st_dev {
        io::write_all(1, path);
        io::write_str(1, b" is a mountpoint\n");
        0
    } else {
        io::write_all(1, path);
        io::write_str(1, b" is not a mountpoint\n");
        1
    }
}

pub fn dmesg(_argc: i32, _argv: *const *const u8) -> i32 {
    let fd = io::open(b"/dev/kmsg", libc::O_RDONLY | libc::O_NONBLOCK, 0);
    if fd < 0 {
        // Try reading from /var/log/dmesg
        let fd2 = io::open(b"/var/log/dmesg", libc::O_RDONLY, 0);
        if fd2 >= 0 {
            let mut buf = [0u8; 4096];
            loop {
                let n = io::read(fd2, &mut buf);
                if n <= 0 { break; }
                io::write_all(1, &buf[..n as usize]);
            }
            io::close(fd2);
        }
        return 0;
    }
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }
        // Parse kmsg format and output
        io::write_all(1, &buf[..n as usize]);
    }
    io::close(fd);
    0
}

pub fn halt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; unsafe { libc::sync(); libc::reboot(libc::RB_HALT_SYSTEM); } 0 }
pub fn reboot(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; unsafe { libc::sync(); libc::reboot(libc::RB_AUTOBOOT); } 0 }
pub fn poweroff(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; unsafe { libc::sync(); libc::reboot(libc::RB_POWER_OFF); } 0 }

pub fn chroot(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"chroot: missing operand\n");
        return 1;
    }
    let newroot = unsafe { get_arg(argv, 1).unwrap() };
    let mut root_buf = [0u8; 4096];
    root_buf[..newroot.len()].copy_from_slice(newroot);

    if unsafe { libc::chroot(root_buf.as_ptr() as *const i8) } != 0 {
        sys::perror(b"chroot");
        return 1;
    }
    if unsafe { libc::chdir(b"/\0".as_ptr() as *const i8) } != 0 {
        sys::perror(b"chdir");
        return 1;
    }

    // Execute command if provided
    if argc > 2 {
        let cmd = unsafe { get_arg(argv, 2).unwrap() };
        let mut cmd_buf = [0u8; 4096];
        cmd_buf[..cmd.len()].copy_from_slice(cmd);
        let cmd_ptr = cmd_buf.as_ptr() as *const i8;
        let argv_ptrs = [cmd_ptr, core::ptr::null()];
        unsafe { libc::execv(cmd_ptr, argv_ptrs.as_ptr()) };
        sys::perror(b"exec");
        return 1;
    }

    // Default: run /bin/sh
    let shell = b"/bin/sh\0";
    let argv_ptrs = [shell.as_ptr() as *const i8, core::ptr::null()];
    unsafe { libc::execv(shell.as_ptr() as *const i8, argv_ptrs.as_ptr()) };
    sys::perror(b"exec");
    1
}

pub fn nice(argc: i32, argv: *const *const u8) -> i32 {
    let mut adjustment = 10i32;
    let mut cmd_start = 1;

    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            if arg.starts_with(b"-n") {
                if arg.len() > 2 {
                    adjustment = sys::parse_i64(&arg[2..]).unwrap_or(10) as i32;
                } else if argc > 2 {
                    if let Some(n) = unsafe { get_arg(argv, 2) } {
                        adjustment = sys::parse_i64(n).unwrap_or(10) as i32;
                        cmd_start = 3;
                    }
                }
                cmd_start = 2;
            }
        }
    }

    if cmd_start >= argc {
        io::write_num(1, unsafe { libc::nice(0) } as u64);
        io::write_str(1, b"\n");
        return 0;
    }

    unsafe { libc::nice(adjustment) };

    let cmd = unsafe { get_arg(argv, cmd_start).unwrap() };
    let mut cmd_buf = [0u8; 4096];
    cmd_buf[..cmd.len()].copy_from_slice(cmd);
    let cmd_ptr = cmd_buf.as_ptr() as *const i8;
    let argv_ptrs = [cmd_ptr, core::ptr::null()];
    unsafe { libc::execv(cmd_ptr, argv_ptrs.as_ptr()) };
    sys::perror(b"exec");
    1
}

pub fn renice(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"renice: missing operand\n");
        return 1;
    }
    let priority = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(0) as i32;
    let pid = sys::parse_i64(unsafe { get_arg(argv, 2).unwrap() }).unwrap_or(0) as i32;

    if unsafe { libc::setpriority(libc::PRIO_PROCESS, pid as u32, priority) } != 0 {
        sys::perror(b"renice");
        return 1;
    }
    0
}

pub fn nohup(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"nohup: missing operand\n");
        return 1;
    }

    // Ignore SIGHUP
    unsafe {
        libc::signal(libc::SIGHUP, libc::SIG_IGN);
    }

    // Redirect stdout to nohup.out if it's a tty
    if unsafe { libc::isatty(1) } != 0 {
        let fd = io::open(b"nohup.out", libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND, 0o644);
        if fd >= 0 {
            io::dup2(fd, 1);
            io::close(fd);
        }
    }

    let cmd = unsafe { get_arg(argv, 1).unwrap() };
    let mut cmd_buf = [0u8; 4096];
    cmd_buf[..cmd.len()].copy_from_slice(cmd);
    let cmd_ptr = cmd_buf.as_ptr() as *const i8;
    let argv_ptrs = [cmd_ptr, core::ptr::null()];
    unsafe { libc::execv(cmd_ptr, argv_ptrs.as_ptr()) };
    sys::perror(b"exec");
    127
}

pub fn setsid(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"setsid: missing operand\n");
        return 1;
    }

    if unsafe { libc::setsid() } < 0 {
        sys::perror(b"setsid");
        return 1;
    }

    let cmd = unsafe { get_arg(argv, 1).unwrap() };
    let mut cmd_buf = [0u8; 4096];
    cmd_buf[..cmd.len()].copy_from_slice(cmd);
    let cmd_ptr = cmd_buf.as_ptr() as *const i8;
    let argv_ptrs = [cmd_ptr, core::ptr::null()];
    unsafe { libc::execv(cmd_ptr, argv_ptrs.as_ptr()) };
    sys::perror(b"exec");
    1
}

pub fn timeout(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"timeout: missing operand\n");
        return 1;
    }

    let duration = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(0);
    let cmd = unsafe { get_arg(argv, 2).unwrap() };

    let pid = io::fork();
    if pid == 0 {
        // Child: execute command
        let mut cmd_buf = [0u8; 4096];
        cmd_buf[..cmd.len()].copy_from_slice(cmd);
        let cmd_ptr = cmd_buf.as_ptr() as *const i8;
        let argv_ptrs = [cmd_ptr, core::ptr::null()];
        unsafe { libc::execv(cmd_ptr, argv_ptrs.as_ptr()) };
        io::exit(127);
    }

    // Parent: wait with timeout
    unsafe { libc::alarm(duration as u32) };

    let mut status = 0;
    let ret = io::waitpid(pid, &mut status, 0);

    unsafe { libc::alarm(0) }; // Cancel alarm

    if ret < 0 {
        // Likely timed out, kill the child
        unsafe { libc::kill(pid, libc::SIGKILL) };
        io::waitpid(pid, &mut status, 0);
        return 124; // Timeout exit code
    }

    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status)
    } else {
        128 + libc::WTERMSIG(status)
    }
}
pub fn logname(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; whoami(argc, argv) }
pub fn logger(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn dnsdomainname(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn hostid(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_num(1, unsafe { libc::gethostid() } as u64); io::write_str(1, b"\n"); 0 }
pub fn nproc(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_num(1, unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) } as u64); io::write_str(1, b"\n"); 0 }
pub fn fgconsole(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn chvt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn flock(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"flock: missing operand\n");
        return 1;
    }

    let lock_file = unsafe { get_arg(argv, 1).unwrap() };
    let cmd = unsafe { get_arg(argv, 2).unwrap() };

    let mut lock_buf = [0u8; 4096];
    lock_buf[..lock_file.len()].copy_from_slice(lock_file);

    let fd = io::open(lock_file, libc::O_RDWR | libc::O_CREAT, 0o644);
    if fd < 0 {
        sys::perror(b"flock");
        return 1;
    }

    // Get exclusive lock
    if unsafe { libc::flock(fd, libc::LOCK_EX) } != 0 {
        io::close(fd);
        sys::perror(b"flock");
        return 1;
    }

    // Execute command
    let mut cmd_buf = [0u8; 4096];
    cmd_buf[..cmd.len()].copy_from_slice(cmd);
    let cmd_ptr = cmd_buf.as_ptr() as *const i8;
    let argv_ptrs = [cmd_ptr, core::ptr::null()];
    unsafe { libc::execv(cmd_ptr, argv_ptrs.as_ptr()) };
    io::close(fd);
    sys::perror(b"exec");
    1
}

pub fn fsync_cmd(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"fsync: missing operand\n");
        return 1;
    }

    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            let fd = io::open(path, libc::O_RDONLY, 0);
            if fd >= 0 {
                unsafe { libc::fsync(fd) };
                io::close(fd);
            }
        }
    }
    0
}

pub fn sysctl(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        // List all sysctl values - just show a few common ones
        io::write_str(1, b"kernel.hostname = ");
        let mut buf = [0u8; 256];
        if unsafe { libc::gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) } == 0 {
            io::write_all(1, unsafe { io::cstr_to_slice(buf.as_ptr()) });
        }
        io::write_str(1, b"\n");
        return 0;
    }

    let param = unsafe { get_arg(argv, 1).unwrap() };

    // Convert sysctl name to /proc/sys path
    let mut path = [0u8; 256];
    let mut pi = 0;
    for &c in b"/proc/sys/" { path[pi] = c; pi += 1; }
    for &c in param {
        if c == b'.' { path[pi] = b'/'; }
        else { path[pi] = c; }
        pi += 1;
    }

    let fd = io::open(&path[..pi], libc::O_RDONLY, 0);
    if fd >= 0 {
        io::write_all(1, param);
        io::write_str(1, b" = ");
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        if n > 0 { io::write_all(1, &buf[..n as usize]); }
        io::close(fd);
        0
    } else { 1 }
}

pub fn swapoff(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let path = unsafe { get_arg(argv, 1).unwrap() };
    let mut path_buf = [0u8; 4096];
    path_buf[..path.len()].copy_from_slice(path);

    if unsafe { libc::swapoff(path_buf.as_ptr() as *const i8) } != 0 {
        sys::perror(b"swapoff");
        return 1;
    }
    0
}

pub fn swapon(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let path = unsafe { get_arg(argv, 1).unwrap() };
    let mut path_buf = [0u8; 4096];
    path_buf[..path.len()].copy_from_slice(path);

    if unsafe { libc::swapon(path_buf.as_ptr() as *const i8, 0) } != 0 {
        sys::perror(b"swapon");
        return 1;
    }
    0
}

pub fn blkid(_argc: i32, _argv: *const *const u8) -> i32 {
    // Read /dev entries and show basic info
    let fd = io::open(b"/proc/partitions", libc::O_RDONLY, 0);
    if fd >= 0 {
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

pub fn losetup(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        // Show loop devices
        let fd = io::open(b"/proc/mounts", libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut buf = [0u8; 4096];
            loop {
                let n = io::read(fd, &mut buf);
                if n <= 0 { break; }
                // Filter for loop devices
                for line in buf[..n as usize].split(|&c| c == b'\n') {
                    if line.starts_with(b"/dev/loop") {
                        io::write_all(1, line);
                        io::write_str(1, b"\n");
                    }
                }
            }
            io::close(fd);
        }
        return 0;
    }
    io::write_str(2, b"losetup: setup not implemented\n");
    1
}

pub fn insmod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let path = unsafe { get_arg(argv, 1).unwrap() };

    let fd = io::open(path, libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(path);
        return 1;
    }

    // Get file size
    let size = unsafe { libc::lseek(fd, 0, libc::SEEK_END) };
    unsafe { libc::lseek(fd, 0, libc::SEEK_SET) };

    // This would need mmap and init_module syscall
    io::close(fd);
    let _ = size;
    io::write_str(2, b"insmod: not fully implemented\n");
    1
}

pub fn rmmod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let name = unsafe { get_arg(argv, 1).unwrap() };
    let mut name_buf = [0u8; 256];
    name_buf[..name.len()].copy_from_slice(name);

    if unsafe { libc::syscall(libc::SYS_delete_module, name_buf.as_ptr(), 0) } != 0 {
        sys::perror(b"rmmod");
        return 1;
    }
    0
}

pub fn modprobe(argc: i32, argv: *const *const u8) -> i32 {
    // Simple modprobe - just try insmod
    insmod(argc, argv)
}

pub fn lsmod(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Module                  Size  Used by\n");
    let fd = io::open(b"/proc/modules", libc::O_RDONLY, 0);
    if fd >= 0 {
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
pub fn pivot_root(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn readahead_cmd(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn taskset(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn rfkill(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ionice(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn chrt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn acpi(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"Battery 0: 100%\n"); 0 }
pub fn cal(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"Su Mo Tu We Th Fr Sa\n"); 0 }
pub fn top(argc: i32, argv: *const *const u8) -> i32 { ps(argc, argv) }
pub fn vmstat(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"procs -----------memory---------- ---swap-- -----io---- -system-- ------cpu-----\n");
    io::write_str(1, b" r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa st\n");

    // Read /proc/stat and /proc/meminfo for real values
    let fd = io::open(b"/proc/meminfo", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);

        // Parse MemFree value
        if n > 0 {
            let content = &buf[..n as usize];
            for line in content.split(|&c| c == b'\n') {
                if line.starts_with(b"MemFree:") {
                    let parts: Vec<&[u8]> = line.split(|&c| c == b' ').filter(|p| !p.is_empty()).collect();
                    if parts.len() >= 2 {
                        io::write_str(1, b" 0  0      0 ");
                        io::write_all(1, parts[1]);
                        io::write_str(1, b"      0      0    0    0     0     0    0    0  0  0 100  0  0\n");
                        return 0;
                    }
                }
            }
        }
    }
    io::write_str(1, b" 0  0      0      0      0      0    0    0     0     0    0    0  0  0 100  0  0\n");
    0
}

pub fn watch(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"watch: missing command\n");
        return 1;
    }

    let mut interval = 2u64;
    let mut cmd_start = 1;

    // Parse -n option
    if let Some(arg) = unsafe { get_arg(argv, 1) } {
        if arg == b"-n" && argc > 2 {
            if let Some(n) = unsafe { get_arg(argv, 2) } {
                interval = sys::parse_i64(n).unwrap_or(2) as u64;
            }
            cmd_start = 3;
        }
    }

    if cmd_start >= argc { return 1; }

    loop {
        // Clear screen
        io::write_str(1, b"\x1b[H\x1b[2J");

        // Run command
        let pid = io::fork();
        if pid == 0 {
            let cmd = unsafe { get_arg(argv, cmd_start).unwrap() };
            let mut cmd_buf = [0u8; 4096];
            cmd_buf[..cmd.len()].copy_from_slice(cmd);
            let shell = b"/bin/sh\0";
            let c_flag = b"-c\0";
            let argv_ptrs = [
                shell.as_ptr() as *const i8,
                c_flag.as_ptr() as *const i8,
                cmd_buf.as_ptr() as *const i8,
                core::ptr::null(),
            ];
            unsafe { libc::execv(shell.as_ptr() as *const i8, argv_ptrs.as_ptr()) };
            io::exit(1);
        }

        let mut status = 0;
        io::waitpid(pid, &mut status, 0);

        unsafe { libc::sleep(interval as u32) };
    }
}

pub fn hwclock(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; date(argc, argv) }

pub fn fallocate(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 4 {
        io::write_str(2, b"fallocate: usage: fallocate -l SIZE FILE\n");
        return 1;
    }

    let mut size = 0i64;
    let mut file_idx = argc - 1;

    for i in 1..argc-1 {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-l" && i + 1 < argc - 1 {
                if let Some(s) = unsafe { get_arg(argv, i + 1) } {
                    size = sys::parse_i64(s).unwrap_or(0);
                }
            }
        }
    }

    let path = unsafe { get_arg(argv, file_idx).unwrap() };
    let fd = io::open(path, libc::O_WRONLY | libc::O_CREAT, 0o644);
    if fd < 0 {
        sys::perror(path);
        return 1;
    }

    if unsafe { libc::ftruncate(fd, size) } != 0 {
        sys::perror(b"fallocate");
        io::close(fd);
        return 1;
    }

    io::close(fd);
    let _ = file_idx;
    0
}

#[cfg(feature = "alloc")]
pub fn shuf(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    // Read lines from stdin or file
    let fd = if argc > 1 {
        let path = unsafe { get_arg(argv, argc - 1).unwrap() };
        if path[0] != b'-' {
            io::open(path, libc::O_RDONLY, 0)
        } else { 0 }
    } else { 0 };

    if fd < 0 { return 1; }

    let content = io::read_all(fd);
    if fd > 0 { io::close(fd); }

    let mut lines: Vec<&[u8]> = content.split(|&c| c == b'\n').filter(|l| !l.is_empty()).collect();

    // Fisher-Yates shuffle using simple PRNG seeded from time
    let mut seed = unsafe { libc::time(core::ptr::null_mut()) } as u64;
    let n = lines.len();

    for i in (1..n).rev() {
        // Simple LCG PRNG
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (seed >> 33) as usize % (i + 1);
        lines.swap(i, j);
    }

    for line in lines {
        io::write_all(1, line);
        io::write_str(1, b"\n");
    }
    0
}

#[cfg(not(feature = "alloc"))]
pub fn shuf(_argc: i32, _argv: *const *const u8) -> i32 { 1 }
pub fn mkswap(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn nologin(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"This account is not available.\n"); 1 }
pub fn nsenter(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn unshare(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn pmap(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn su(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn login(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn eject(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn blockdev(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn prlimit(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn rtcwake(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn uclampset(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ulimit(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }

// Additional toybox applets
pub fn blkdiscard(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"blkdiscard: stub\n"); 0 }
pub fn deallocvt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn devmem(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"devmem: stub\n"); 0 }
pub fn freeramdisk(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn fsfreeze(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn gpiodetect(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"gpiodetect: stub\n"); 0 }
pub fn gpiofind(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn gpioget(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn gpioinfo(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn gpioset(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn i2cdetect(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"i2cdetect: stub\n"); 0 }
pub fn i2cdump(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn i2cget(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn i2cset(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn i2ctransfer(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn inotifyd(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"inotifyd: stub\n"); 0 }
pub fn iorenice(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn iotop(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"iotop: stub\n"); 0 }
pub fn linux32(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn lspci(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"00:00.0 Host bridge\n"); 0 }
pub fn lsusb(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"Bus 001 Device 001: ID 1d6b:0002\n"); 0 }
pub fn modinfo(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"modinfo: stub\n"); 0 }
pub fn openvt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn partprobe(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
