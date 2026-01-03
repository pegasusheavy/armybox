//! System information and management applets

use crate::io;
use crate::sys;
use super::super::{get_arg, has_opt, is_opt, is_option};

/// uname - print system information
pub fn uname(argc: i32, argv: *const *const u8) -> i32 {
    let mut show_all = false;
    let mut show_sysname = false;
    let mut show_nodename = false;
    let mut show_release = false;
    let mut show_version = false;
    let mut show_machine = false;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if arg[0] != b'-' {
            break;
        }

        if has_opt(arg, b'a') { show_all = true; }
        if has_opt(arg, b's') { show_sysname = true; }
        if has_opt(arg, b'n') { show_nodename = true; }
        if has_opt(arg, b'r') { show_release = true; }
        if has_opt(arg, b'v') { show_version = true; }
        if has_opt(arg, b'm') { show_machine = true; }
    }

    // Default to sysname only
    if !show_all && !show_sysname && !show_nodename && !show_release && !show_version && !show_machine {
        show_sysname = true;
    }

    if show_all {
        show_sysname = true;
        show_nodename = true;
        show_release = true;
        show_version = true;
        show_machine = true;
    }

    let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
    if io::uname(&mut uts) != 0 {
        sys::perror(b"uname");
        return 1;
    }

    let mut first = true;

    if show_sysname {
        let len = uts.sysname.iter().position(|&c| c == 0).unwrap_or(uts.sysname.len());
        io::write_all(1, unsafe { core::slice::from_raw_parts(uts.sysname.as_ptr() as *const u8, len) });
        first = false;
    }

    if show_nodename {
        if !first { io::write_str(1, b" "); }
        let len = uts.nodename.iter().position(|&c| c == 0).unwrap_or(uts.nodename.len());
        io::write_all(1, unsafe { core::slice::from_raw_parts(uts.nodename.as_ptr() as *const u8, len) });
        first = false;
    }

    if show_release {
        if !first { io::write_str(1, b" "); }
        let len = uts.release.iter().position(|&c| c == 0).unwrap_or(uts.release.len());
        io::write_all(1, unsafe { core::slice::from_raw_parts(uts.release.as_ptr() as *const u8, len) });
        first = false;
    }

    if show_version {
        if !first { io::write_str(1, b" "); }
        let len = uts.version.iter().position(|&c| c == 0).unwrap_or(uts.version.len());
        io::write_all(1, unsafe { core::slice::from_raw_parts(uts.version.as_ptr() as *const u8, len) });
        first = false;
    }

    if show_machine {
        if !first { io::write_str(1, b" "); }
        let len = uts.machine.iter().position(|&c| c == 0).unwrap_or(uts.machine.len());
        io::write_all(1, unsafe { core::slice::from_raw_parts(uts.machine.as_ptr() as *const u8, len) });
    }

    io::write_str(1, b"\n");
    0
}

/// hostname - print system hostname
pub fn hostname(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        if let Some(name) = io::gethostname() {
            io::write_all(1, &name);
            io::write_str(1, b"\n");
            return 0;
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        let mut buf = [0u8; 256];
        let ret = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) };
        if ret == 0 {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
            io::write_all(1, &buf[..len]);
            io::write_str(1, b"\n");
            return 0;
        }
    }

    sys::perror(b"hostname");
    1
}

/// whoami - print effective user name
pub fn whoami(_argc: i32, _argv: *const *const u8) -> i32 {
    let uid = io::geteuid();

    // Read /etc/passwd to find username
    let fd = io::open(b"/etc/passwd", libc::O_RDONLY, 0);
    if fd < 0 {
        // Fallback to printing uid
        io::write_num(1, uid as u64);
        io::write_str(1, b"\n");
        return 0;
    }

    let mut buf = [0u8; 4096];
    let mut line = [0u8; 256];
    let mut line_len = 0;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 {
            break;
        }

        for i in 0..n as usize {
            if buf[i] == b'\n' {
                // Parse line: username:x:uid:gid:...
                if let Some(username) = parse_passwd_line(&line[..line_len], uid) {
                    io::write_all(1, username);
                    io::write_str(1, b"\n");
                    io::close(fd);
                    return 0;
                }
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = buf[i];
                line_len += 1;
            }
        }
    }

    io::close(fd);

    // Fallback
    io::write_num(1, uid as u64);
    io::write_str(1, b"\n");
    0
}

fn parse_passwd_line(line: &[u8], target_uid: u32) -> Option<&[u8]> {
    let mut fields = line.split(|&c| c == b':');
    let username = fields.next()?;
    let _ = fields.next()?; // password
    let uid_str = fields.next()?;

    let uid = sys::parse_u64(uid_str)? as u32;
    if uid == target_uid {
        Some(username)
    } else {
        None
    }
}

/// id - print user and group IDs
pub fn id(_argc: i32, _argv: *const *const u8) -> i32 {
    let uid = io::getuid();
    let euid = io::geteuid();
    let gid = io::getgid();

    io::write_str(1, b"uid=");
    io::write_num(1, uid as u64);

    // Try to get username
    let fd = io::open(b"/etc/passwd", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);

        if n > 0 {
            // Simple search for uid
            // In production, we'd parse properly
        }
    }

    io::write_str(1, b" gid=");
    io::write_num(1, gid as u64);

    if uid != euid {
        io::write_str(1, b" euid=");
        io::write_num(1, euid as u64);
    }

    io::write_str(1, b"\n");
    0
}

/// groups - print group memberships
pub fn groups(_argc: i32, _argv: *const *const u8) -> i32 {
    let gid = io::getgid();
    io::write_num(1, gid as u64);
    io::write_str(1, b"\n");
    0
}

/// logname - print login name
pub fn logname(_argc: i32, _argv: *const *const u8) -> i32 {
    if let Some(name) = io::getenv(b"LOGNAME") {
        io::write_all(1, name);
        io::write_str(1, b"\n");
        return 0;
    }

    if let Some(name) = io::getenv(b"USER") {
        io::write_all(1, name);
        io::write_str(1, b"\n");
        return 0;
    }

    io::write_str(2, b"logname: no login name\n");
    1
}

/// arch - print machine architecture
pub fn arch(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
    if io::uname(&mut uts) != 0 {
        sys::perror(b"arch");
        return 1;
    }

    let len = uts.machine.iter().position(|&c| c == 0).unwrap_or(uts.machine.len());
    io::write_all(1, unsafe { core::slice::from_raw_parts(uts.machine.as_ptr() as *const u8, len) });
    io::write_str(1, b"\n");
    0
}

/// nproc - print number of processing units
pub fn nproc(_argc: i32, _argv: *const *const u8) -> i32 {
    let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    if n > 0 {
        io::write_num(1, n as u64);
        io::write_str(1, b"\n");
        0
    } else {
        io::write_str(1, b"1\n");
        0
    }
}

/// uptime - print system uptime
pub fn uptime(_argc: i32, _argv: *const *const u8) -> i32 {
    // Read /proc/uptime
    let fd = io::open(b"/proc/uptime", libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(b"/proc/uptime");
        return 1;
    }

    let mut buf = [0u8; 64];
    let n = io::read(fd, &mut buf);
    io::close(fd);

    if n <= 0 {
        return 1;
    }

    // Parse first number (seconds)
    let space_pos = buf[..n as usize].iter().position(|&c| c == b' ' || c == b'.').unwrap_or(n as usize);
    let secs = sys::parse_u64(&buf[..space_pos]).unwrap_or(0);

    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    io::write_str(1, b"up ");
    if days > 0 {
        io::write_num(1, days);
        io::write_str(1, if days == 1 { b" day, " } else { b" days, " });
    }
    io::write_num(1, hours);
    io::write_str(1, b":");
    if mins < 10 { io::write_str(1, b"0"); }
    io::write_num(1, mins);
    io::write_str(1, b"\n");

    0
}

/// date - print or set date
pub fn date(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut tv: libc::timeval = unsafe { core::mem::zeroed() };
    unsafe { libc::gettimeofday(&mut tv, core::ptr::null_mut()) };

    let mut tm: libc::tm = unsafe { core::mem::zeroed() };
    unsafe { libc::localtime_r(&tv.tv_sec, &mut tm) };

    static MONTHS: &[&[u8]] = &[
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun",
        b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"
    ];

    static DAYS: &[&[u8]] = &[
        b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"
    ];

    // Format: Day Mon DD HH:MM:SS YYYY
    io::write_all(1, DAYS[tm.tm_wday as usize % 7]);
    io::write_str(1, b" ");
    io::write_all(1, MONTHS[tm.tm_mon as usize % 12]);
    io::write_str(1, b" ");

    if tm.tm_mday < 10 { io::write_str(1, b" "); }
    io::write_num(1, tm.tm_mday as u64);
    io::write_str(1, b" ");

    if tm.tm_hour < 10 { io::write_str(1, b"0"); }
    io::write_num(1, tm.tm_hour as u64);
    io::write_str(1, b":");
    if tm.tm_min < 10 { io::write_str(1, b"0"); }
    io::write_num(1, tm.tm_min as u64);
    io::write_str(1, b":");
    if tm.tm_sec < 10 { io::write_str(1, b"0"); }
    io::write_num(1, tm.tm_sec as u64);
    io::write_str(1, b" ");

    io::write_num(1, (tm.tm_year + 1900) as u64);
    io::write_str(1, b"\n");

    0
}

/// env - print environment
pub fn env(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe extern "C" {
        static environ: *const *const i8;
    }

    unsafe {
        let mut ptr = environ;
        while !(*ptr).is_null() {
            let s = *ptr;
            let len = io::strlen(s as *const u8);
            io::write_all(1, core::slice::from_raw_parts(s as *const u8, len));
            io::write_str(1, b"\n");
            ptr = ptr.add(1);
        }
    }

    0
}

/// printenv - print environment variable
pub fn printenv(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        return env(argc, argv);
    }

    for i in 1..argc {
        let name = match unsafe { get_arg(argv, i) } {
            Some(n) => n,
            None => continue,
        };

        if let Some(value) = io::getenv(name) {
            io::write_all(1, value);
            io::write_str(1, b"\n");
        }
    }

    0
}

/// tty - print terminal name
pub fn tty(_argc: i32, _argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        if let Some(name) = io::ttyname(0) {
            io::write_all(1, &name);
            io::write_str(1, b"\n");
            return 0;
        }
    }

    if io::isatty(0) {
        io::write_str(1, b"/dev/tty\n");
        0
    } else {
        io::write_str(1, b"not a tty\n");
        1
    }
}

/// df - report filesystem disk space usage
pub fn df(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Filesystem     1K-blocks      Used Available Use% Mounted on\n");

    // Read /proc/mounts
    let fd = io::open(b"/proc/mounts", libc::O_RDONLY, 0);
    if fd < 0 {
        return 1;
    }

    let mut buf = [0u8; 8192];
    let n = io::read(fd, &mut buf);
    io::close(fd);

    if n <= 0 {
        return 1;
    }

    // Parse each line
    let mut line_start = 0;
    for i in 0..n as usize {
        if buf[i] == b'\n' {
            df_line(&buf[line_start..i]);
            line_start = i + 1;
        }
    }

    0
}

fn df_line(line: &[u8]) {
    // Parse: device mountpoint fstype options ...
    let mut fields = line.split(|&c| c == b' ');

    let device = match fields.next() {
        Some(d) => d,
        None => return,
    };

    let mountpoint = match fields.next() {
        Some(m) => m,
        None => return,
    };

    // Skip virtual filesystems
    if device.starts_with(b"none") || device.starts_with(b"proc") ||
       device.starts_with(b"sys") || device.starts_with(b"devpts") ||
       device.starts_with(b"tmpfs") && !mountpoint.starts_with(b"/run") {
        return;
    }

    // Get stats using statfs
    let mut statfs: libc::statfs = unsafe { core::mem::zeroed() };

    let mut mp_buf = [0u8; 256];
    if mountpoint.len() >= mp_buf.len() {
        return;
    }
    mp_buf[..mountpoint.len()].copy_from_slice(mountpoint);
    mp_buf[mountpoint.len()] = 0;

    if unsafe { libc::statfs(mp_buf.as_ptr() as *const i8, &mut statfs) } != 0 {
        return;
    }

    let block_size = statfs.f_bsize as u64;
    let total = statfs.f_blocks as u64 * block_size / 1024;
    let free = statfs.f_bfree as u64 * block_size / 1024;
    let available = statfs.f_bavail as u64 * block_size / 1024;
    let used = total.saturating_sub(free);
    let use_pct = if total > 0 { (used * 100) / total } else { 0 };

    // Print
    io::write_all(1, device);
    // Padding
    for _ in device.len()..15 { io::write_str(1, b" "); }

    let mut num_buf = [0u8; 20];
    let s = sys::format_u64(total, &mut num_buf);
    for _ in s.len()..11 { io::write_str(1, b" "); }
    io::write_all(1, s);

    let s = sys::format_u64(used, &mut num_buf);
    for _ in s.len()..11 { io::write_str(1, b" "); }
    io::write_all(1, s);

    let s = sys::format_u64(available, &mut num_buf);
    for _ in s.len()..10 { io::write_str(1, b" "); }
    io::write_all(1, s);

    let s = sys::format_u64(use_pct, &mut num_buf);
    for _ in s.len()..4 { io::write_str(1, b" "); }
    io::write_all(1, s);
    io::write_str(1, b"% ");

    io::write_all(1, mountpoint);
    io::write_str(1, b"\n");
}

/// du - estimate file space usage
pub fn du(argc: i32, argv: *const *const u8) -> i32 {
    let mut summarize = false;
    let mut human = false;
    let mut paths_start = 1;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if arg[0] != b'-' {
            break;
        }

        paths_start = i + 1;

        if has_opt(arg, b's') { summarize = true; }
        if has_opt(arg, b'h') { human = true; }
    }

    if paths_start >= argc {
        return du_path(b".", summarize, human);
    }

    let mut exit_code = 0;

    for i in paths_start..argc {
        let path = match unsafe { get_arg(argv, i) } {
            Some(p) => p,
            None => continue,
        };

        if du_path(path, summarize, human) != 0 {
            exit_code = 1;
        }
    }

    exit_code
}

fn du_path(path: &[u8], summarize: bool, human: bool) -> i32 {
    let total = du_recursive(path, summarize, human);

    if summarize {
        print_du_size(total, human);
        io::write_str(1, b"\t");
        io::write_all(1, path);
        io::write_str(1, b"\n");
    }

    0
}

fn du_recursive(path: &[u8], summarize: bool, human: bool) -> u64 {
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    if io::lstat(path, &mut st) != 0 {
        return 0;
    }

    let size = (st.st_blocks as u64) * 512 / 1024; // Convert to KB

    if (st.st_mode & libc::S_IFMT) != libc::S_IFDIR {
        if !summarize {
            print_du_size(size, human);
            io::write_str(1, b"\t");
            io::write_all(1, path);
            io::write_str(1, b"\n");
        }
        return size;
    }

    let dir = io::opendir(path);
    if dir.is_null() {
        return size;
    }

    let mut total = size;

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let (name_slice, name_len) = unsafe { io::dirent_name(entry) };

        // Skip . and ..
        if (name_len == 1 && name_slice[0] == b'.') ||
           (name_len == 2 && name_slice[0] == b'.' && name_slice[1] == b'.') {
            continue;
        }

        // Build child path
        let mut child = [0u8; 4096];
        let mut len = 0;
        for &c in path { if len < child.len() - 1 { child[len] = c; len += 1; } }
        if len > 0 && child[len - 1] != b'/' { child[len] = b'/'; len += 1; }
        for &c in name_slice { if len < child.len() - 1 { child[len] = c; len += 1; } }

        total += du_recursive(&child[..len], summarize, human);
    }

    io::closedir(dir);

    if !summarize {
        print_du_size(total, human);
        io::write_str(1, b"\t");
        io::write_all(1, path);
        io::write_str(1, b"\n");
    }

    total
}

fn print_du_size(kb: u64, human: bool) {
    let mut num_buf = [0u8; 20];

    if human {
        let (size, suffix) = if kb >= 1024 * 1024 {
            (kb / (1024 * 1024), b'G')
        } else if kb >= 1024 {
            (kb / 1024, b'M')
        } else {
            (kb, b'K')
        };

        let s = sys::format_u64(size, &mut num_buf);
        io::write_all(1, s);
        io::write_all(1, &[suffix]);
    } else {
        let s = sys::format_u64(kb, &mut num_buf);
        io::write_all(1, s);
    }
}

/// ps - process status
pub fn ps(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"  PID TTY          TIME CMD\n");

    let dir = io::opendir(b"/proc");
    if dir.is_null() {
        sys::perror(b"/proc");
        return 1;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let (name_slice, name_len) = unsafe { io::dirent_name(entry) };

        // Check if it's a PID directory (all digits)
        if !name_slice.iter().all(|&c| c >= b'0' && c <= b'9') {
            continue;
        }

        // Read /proc/PID/comm
        let mut comm_path = [0u8; 64];
        let mut len = 0;
        for &c in b"/proc/" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in name_slice { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in b"/comm" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }

        let fd = io::open(&comm_path[..len], libc::O_RDONLY, 0);
        if fd < 0 {
            continue;
        }

        let mut comm = [0u8; 256];
        let n = io::read(fd, &mut comm);
        io::close(fd);

        if n <= 0 {
            continue;
        }

        // Print PID
        for _ in name_len..5 { io::write_str(1, b" "); }
        io::write_all(1, name_slice);
        io::write_str(1, b" ?        00:00:00 ");

        // Print command (strip newline)
        let comm_len = comm[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);
        io::write_all(1, &comm[..comm_len]);
        io::write_str(1, b"\n");
    }

    io::closedir(dir);
    0
}

/// kill - send signal to process
pub fn kill(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = 15i32; // SIGTERM
    let mut pids_start = 1;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if arg[0] == b'-' && arg.len() > 1 {
            // Parse signal
            if arg[1] >= b'0' && arg[1] <= b'9' {
                signal = sys::parse_i64(&arg[1..]).unwrap_or(15) as i32;
            } else {
                // Named signal
                match &arg[1..] {
                    b"HUP" | b"SIGHUP" => signal = 1,
                    b"INT" | b"SIGINT" => signal = 2,
                    b"QUIT" | b"SIGQUIT" => signal = 3,
                    b"KILL" | b"SIGKILL" => signal = 9,
                    b"TERM" | b"SIGTERM" => signal = 15,
                    b"CONT" | b"SIGCONT" => signal = 18,
                    b"STOP" | b"SIGSTOP" => signal = 19,
                    _ => {}
                }
            }
            pids_start = i + 1;
        } else {
            break;
        }
    }

    let mut exit_code = 0;

    for i in pids_start..argc {
        let pid_str = match unsafe { get_arg(argv, i) } {
            Some(p) => p,
            None => continue,
        };

        let pid = match sys::parse_i64(pid_str) {
            Some(p) => p as i32,
            None => {
                io::write_str(2, b"kill: invalid pid: ");
                io::write_all(2, pid_str);
                io::write_str(2, b"\n");
                exit_code = 1;
                continue;
            }
        };

        if io::kill(pid, signal) != 0 {
            sys::perror(pid_str);
            exit_code = 1;
        }
    }

    exit_code
}

/// dmesg - print kernel ring buffer
pub fn dmesg(_argc: i32, _argv: *const *const u8) -> i32 {
    let fd = io::open(b"/dev/kmsg", libc::O_RDONLY | libc::O_NONBLOCK, 0);
    if fd >= 0 {
        let mut buf = [0u8; 8192];
        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 {
                break;
            }
            // Parse kmsg format and output
            // For now, just pass through
            io::write_all(1, &buf[..n as usize]);
        }
        io::close(fd);
        return 0;
    }

    // Fallback to /var/log/dmesg
    let fd = io::open(b"/var/log/dmesg", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 8192];
        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 {
                break;
            }
            io::write_all(1, &buf[..n as usize]);
        }
        io::close(fd);
        return 0;
    }

    sys::perror(b"dmesg");
    1
}

/// mount - mount filesystem (stub)
pub fn mount(_argc: i32, _argv: *const *const u8) -> i32 {
    // Just list mounts from /proc/mounts
    let fd = io::open(b"/proc/mounts", libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(b"/proc/mounts");
        return 1;
    }

    let mut buf = [0u8; 8192];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 {
            break;
        }
        io::write_all(1, &buf[..n as usize]);
    }

    io::close(fd);
    0
}

/// umount - unmount filesystem (stub)
pub fn umount(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"umount: usage: umount <mountpoint>\n");
        return 1;
    }

    io::write_str(2, b"umount: not implemented (requires root)\n");
    1
}

/// free - display memory usage
pub fn free(argc: i32, argv: *const *const u8) -> i32 {
    let mut human = false;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if has_opt(arg, b'h') { human = true; }
    }

    // Read /proc/meminfo
    let fd = io::open(b"/proc/meminfo", libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(b"/proc/meminfo");
        return 1;
    }

    let mut buf = [0u8; 4096];
    let n = io::read(fd, &mut buf);
    io::close(fd);

    if n <= 0 {
        return 1;
    }

    let mut mem_total: u64 = 0;
    let mut mem_free: u64 = 0;
    let mut mem_available: u64 = 0;
    let mut buffers: u64 = 0;
    let mut cached: u64 = 0;
    let mut swap_total: u64 = 0;
    let mut swap_free: u64 = 0;

    // Parse lines
    let mut line_start = 0;
    for i in 0..n as usize {
        if buf[i] == b'\n' {
            let line = &buf[line_start..i];
            if line.starts_with(b"MemTotal:") {
                mem_total = parse_meminfo_value(line);
            } else if line.starts_with(b"MemFree:") {
                mem_free = parse_meminfo_value(line);
            } else if line.starts_with(b"MemAvailable:") {
                mem_available = parse_meminfo_value(line);
            } else if line.starts_with(b"Buffers:") {
                buffers = parse_meminfo_value(line);
            } else if line.starts_with(b"Cached:") {
                cached = parse_meminfo_value(line);
            } else if line.starts_with(b"SwapTotal:") {
                swap_total = parse_meminfo_value(line);
            } else if line.starts_with(b"SwapFree:") {
                swap_free = parse_meminfo_value(line);
            }
            line_start = i + 1;
        }
    }

    let mem_used = mem_total.saturating_sub(mem_free).saturating_sub(buffers).saturating_sub(cached);
    let swap_used = swap_total.saturating_sub(swap_free);

    // Header
    io::write_str(1, b"              total        used        free      shared  buff/cache   available\n");

    // Memory line
    io::write_str(1, b"Mem:    ");
    print_free_size(mem_total, human);
    print_free_size(mem_used, human);
    print_free_size(mem_free, human);
    print_free_size(0, human); // shared (simplified)
    print_free_size(buffers + cached, human);
    print_free_size(mem_available, human);
    io::write_str(1, b"\n");

    // Swap line
    io::write_str(1, b"Swap:   ");
    print_free_size(swap_total, human);
    print_free_size(swap_used, human);
    print_free_size(swap_free, human);
    io::write_str(1, b"\n");

    0
}

fn parse_meminfo_value(line: &[u8]) -> u64 {
    // Find first digit
    let start = line.iter().position(|&c| c >= b'0' && c <= b'9').unwrap_or(0);
    let end = line[start..].iter().position(|&c| c < b'0' || c > b'9').unwrap_or(line.len() - start);
    sys::parse_u64(&line[start..start + end]).unwrap_or(0)
}

fn print_free_size(kb: u64, human: bool) {
    let mut num_buf = [0u8; 20];

    if human {
        let (size, suffix) = if kb >= 1024 * 1024 {
            (kb / (1024 * 1024), b"Gi")
        } else if kb >= 1024 {
            (kb / 1024, b"Mi")
        } else {
            (kb, b"Ki")
        };
        let s = sys::format_u64(size, &mut num_buf);
        for _ in (s.len() + 2)..12 { io::write_str(1, b" "); }
        io::write_all(1, s);
        io::write_all(1, suffix);
    } else {
        let s = sys::format_u64(kb, &mut num_buf);
        for _ in s.len()..12 { io::write_str(1, b" "); }
        io::write_all(1, s);
    }
}

/// hostid - print host identifier
pub fn hostid(_argc: i32, _argv: *const *const u8) -> i32 {
    let id = unsafe { libc::gethostid() } as u32;
    let mut buf = [0u8; 16];
    let s = sys::format_hex(id as u64, &mut buf);
    io::write_all(1, s);
    io::write_str(1, b"\n");
    0
}

/// lsmod - list loaded kernel modules
pub fn lsmod(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Module                  Size  Used by\n");

    let fd = io::open(b"/proc/modules", libc::O_RDONLY, 0);
    if fd < 0 {
        return 0; // No modules or no permission
    }

    let mut buf = [0u8; 8192];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 {
            break;
        }

        // Parse and output
        let mut line_start = 0;
        for i in 0..n as usize {
            if buf[i] == b'\n' {
                let line = &buf[line_start..i];
                // Format: name size used_count deps state offset
                let mut fields = line.split(|&c| c == b' ');
                if let Some(name) = fields.next() {
                    io::write_all(1, name);
                    for _ in name.len()..24 { io::write_str(1, b" "); }

                    if let Some(size) = fields.next() {
                        io::write_all(1, size);
                        for _ in size.len()..6 { io::write_str(1, b" "); }
                    }

                    if let Some(used) = fields.next() {
                        io::write_all(1, used);
                        io::write_str(1, b" ");
                    }

                    if let Some(deps) = fields.next() {
                        if deps != b"-" {
                            io::write_all(1, deps);
                        }
                    }

                    io::write_str(1, b"\n");
                }
                line_start = i + 1;
            }
        }
    }

    io::close(fd);
    0
}

/// halt - halt the system
pub fn halt(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"halt: requires root privileges\n");
    1
}

/// reboot - reboot the system
pub fn reboot(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"reboot: requires root privileges\n");
    1
}

/// poweroff - power off the system
pub fn poweroff(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"poweroff: requires root privileges\n");
    1
}

/// w - show who is logged in and what they are doing
pub fn w(_argc: i32, _argv: *const *const u8) -> i32 {
    uptime(1, core::ptr::null());
    io::write_str(1, b"USER     TTY      FROM             LOGIN@   IDLE   JCPU   PCPU WHAT\n");
    // Reading utmp requires more complex parsing
    0
}

/// users - print logged in users
pub fn users(_argc: i32, _argv: *const *const u8) -> i32 {
    // Read /var/run/utmp or simplified
    if let Some(user) = io::getenv(b"USER") {
        io::write_all(1, user);
        io::write_str(1, b"\n");
    }
    0
}

/// who - show who is logged in
pub fn who(_argc: i32, _argv: *const *const u8) -> i32 {
    // Simplified - just show current user
    if let Some(user) = io::getenv(b"USER") {
        io::write_all(1, user);
        io::write_str(1, b"  tty1         ");
        date(1, core::ptr::null());
    }
    0
}

/// chroot - run command with different root
pub fn chroot(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"chroot: missing operand\n");
        return 1;
    }

    let newroot = match unsafe { get_arg(argv, 1) } {
        Some(r) => r,
        None => return 1,
    };

    // Need null-terminated path
    let mut path_buf = [0u8; 4096];
    if newroot.len() >= path_buf.len() {
        io::write_str(2, b"chroot: path too long\n");
        return 1;
    }
    path_buf[..newroot.len()].copy_from_slice(newroot);
    path_buf[newroot.len()] = 0;

    if unsafe { libc::chroot(path_buf.as_ptr() as *const i8) } != 0 {
        sys::perror(newroot);
        return 1;
    }

    if io::chdir(b"/") != 0 {
        sys::perror(b"/");
        return 1;
    }

    // If command specified, exec it
    if argc > 2 {
        io::write_str(2, b"chroot: exec not implemented\n");
        return 1;
    }

    0
}

/// timeout - run command with time limit
pub fn timeout(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"timeout: usage: timeout DURATION COMMAND [ARGS]\n");
        return 1;
    }

    io::write_str(2, b"timeout: not implemented (requires fork/exec)\n");
    1
}

/// nice - run with modified priority
pub fn nice(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        // Print current nice value
        let n = unsafe { libc::nice(0) };
        io::write_signed(1, n as i64);
        io::write_str(1, b"\n");
        return 0;
    }

    io::write_str(2, b"nice: running commands not implemented\n");
    1
}

/// nohup - run immune to hangups
pub fn nohup(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"nohup: missing operand\n");
        return 1;
    }

    io::write_str(2, b"nohup: not implemented (requires fork/exec)\n");
    1
}

/// killall - kill processes by name
pub fn killall(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = 15i32; // SIGTERM
    let mut name_start = 1;

    // Parse signal
    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if arg[0] == b'-' && arg.len() > 1 {
            if arg[1] >= b'0' && arg[1] <= b'9' {
                signal = sys::parse_i64(&arg[1..]).unwrap_or(15) as i32;
            } else {
                match &arg[1..] {
                    b"HUP" | b"SIGHUP" => signal = 1,
                    b"INT" | b"SIGINT" => signal = 2,
                    b"QUIT" | b"SIGQUIT" => signal = 3,
                    b"KILL" | b"SIGKILL" => signal = 9,
                    b"TERM" | b"SIGTERM" => signal = 15,
                    b"CONT" | b"SIGCONT" => signal = 18,
                    b"STOP" | b"SIGSTOP" => signal = 19,
                    _ => {}
                }
            }
            name_start = i + 1;
        } else {
            break;
        }
    }

    if name_start >= argc {
        io::write_str(2, b"killall: missing process name\n");
        return 1;
    }

    let target_name = match unsafe { get_arg(argv, name_start) } {
        Some(n) => n,
        None => return 1,
    };

    let mut found = false;
    let mut exit_code = 0;

    // Scan /proc for matching processes
    let dir = io::opendir(b"/proc");
    if dir.is_null() {
        sys::perror(b"/proc");
        return 1;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let (name_slice, _) = unsafe { io::dirent_name(entry) };

        // Check if it's a PID directory
        if !name_slice.iter().all(|&c| c >= b'0' && c <= b'9') {
            continue;
        }

        // Read /proc/PID/comm
        let mut comm_path = [0u8; 64];
        let mut len = 0;
        for &c in b"/proc/" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in name_slice { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in b"/comm" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }

        let fd = io::open(&comm_path[..len], libc::O_RDONLY, 0);
        if fd < 0 {
            continue;
        }

        let mut comm = [0u8; 256];
        let n = io::read(fd, &mut comm);
        io::close(fd);

        if n <= 0 {
            continue;
        }

        // Strip newline and compare
        let comm_len = comm[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);

        if io::bytes_eq(&comm[..comm_len], target_name) {
            let pid = sys::parse_i64(name_slice).unwrap_or(0) as i32;
            if pid > 0 {
                found = true;
                if io::kill(pid, signal) != 0 {
                    sys::perror(name_slice);
                    exit_code = 1;
                }
            }
        }
    }

    io::closedir(dir);

    if !found {
        io::write_all(2, target_name);
        io::write_str(2, b": no process found\n");
        return 1;
    }

    exit_code
}

/// pidof - find PID of running program
pub fn pidof(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"pidof: missing program name\n");
        return 1;
    }

    let target_name = match unsafe { get_arg(argv, 1) } {
        Some(n) => n,
        None => return 1,
    };

    let mut found = false;
    let mut first = true;

    // Scan /proc for matching processes
    let dir = io::opendir(b"/proc");
    if dir.is_null() {
        return 1;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let (name_slice, _) = unsafe { io::dirent_name(entry) };

        // Check if it's a PID directory
        if !name_slice.iter().all(|&c| c >= b'0' && c <= b'9') {
            continue;
        }

        // Read /proc/PID/comm
        let mut comm_path = [0u8; 64];
        let mut len = 0;
        for &c in b"/proc/" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in name_slice { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in b"/comm" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }

        let fd = io::open(&comm_path[..len], libc::O_RDONLY, 0);
        if fd < 0 {
            continue;
        }

        let mut comm = [0u8; 256];
        let n = io::read(fd, &mut comm);
        io::close(fd);

        if n <= 0 {
            continue;
        }

        // Strip newline and compare
        let comm_len = comm[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);

        if io::bytes_eq(&comm[..comm_len], target_name) {
            if !first {
                io::write_str(1, b" ");
            }
            io::write_all(1, name_slice);
            first = false;
            found = true;
        }
    }

    io::closedir(dir);

    if found {
        io::write_str(1, b"\n");
        0
    } else {
        1
    }
}

/// pwdx - report current working directory of a process
pub fn pwdx(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        let pid = match unsafe { get_arg(argv, i) } {
            Some(p) => p,
            None => continue,
        };

        // Build /proc/PID/cwd path
        let mut link_path = [0u8; 64];
        let mut len = 0;
        for &c in b"/proc/" { if len < link_path.len() - 1 { link_path[len] = *c; len += 1; } }
        for &c in pid { if len < link_path.len() - 1 { link_path[len] = *c; len += 1; } }
        for &c in b"/cwd" { if len < link_path.len() - 1 { link_path[len] = *c; len += 1; } }
        link_path[len] = 0;

        let mut target = [0u8; 4096];
        let n = unsafe { libc::readlink(link_path.as_ptr() as *const i8, target.as_mut_ptr() as *mut i8, target.len()) };

        if n < 0 {
            io::write_all(1, pid);
            io::write_str(1, b": Permission denied or no such process\n");
            continue;
        }

        io::write_all(1, pid);
        io::write_str(1, b": ");
        io::write_all(1, &target[..n as usize]);
        io::write_str(1, b"\n");
    }

    0
}

/// renice - alter priority of running processes
pub fn renice(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"renice: usage: renice priority pid...\n");
        return 1;
    }

    let priority = match unsafe { get_arg(argv, 1) }.and_then(sys::parse_i64) {
        Some(p) => p as i32,
        None => {
            io::write_str(2, b"renice: invalid priority\n");
            return 1;
        }
    };

    let mut exit_code = 0;

    for i in 2..argc {
        let pid_str = match unsafe { get_arg(argv, i) } {
            Some(p) => p,
            None => continue,
        };

        let pid = match sys::parse_i64(pid_str) {
            Some(p) => p as i32,
            None => {
                io::write_str(2, b"renice: invalid pid: ");
                io::write_all(2, pid_str);
                io::write_str(2, b"\n");
                exit_code = 1;
                continue;
            }
        };

        if unsafe { libc::setpriority(libc::PRIO_PROCESS as u32, pid as u32, priority) } != 0 {
            sys::perror(pid_str);
            exit_code = 1;
        } else {
            io::write_all(1, pid_str);
            io::write_str(1, b" (process ID) old priority 0, new priority ");
            io::write_signed(1, priority as i64);
            io::write_str(1, b"\n");
        }
    }

    exit_code
}

/// setsid - run program in new session
pub fn setsid(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"setsid: missing operand\n");
        return 1;
    }

    // Create new session
    if unsafe { libc::setsid() } < 0 {
        // If we're already session leader, fork
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            sys::perror(b"fork");
            return 1;
        }
        if pid > 0 {
            // Parent exits
            return 0;
        }
        // Child creates new session
        unsafe { libc::setsid() };
    }

    io::write_str(2, b"setsid: exec not implemented\n");
    1
}

/// logger - write to syslog
pub fn logger(argc: i32, argv: *const *const u8) -> i32 {
    let mut tag: Option<&[u8]> = None;
    let mut message_start = 1;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if io::bytes_eq(arg, b"-t") && i + 1 < argc {
            tag = unsafe { get_arg(argv, i + 1) };
            message_start = i + 2;
        } else if arg[0] != b'-' {
            break;
        } else {
            message_start = i + 1;
        }
    }

    // Build message
    let mut message = [0u8; 4096];
    let mut msg_len = 0;

    for i in message_start..argc {
        if i > message_start && msg_len < message.len() - 1 {
            message[msg_len] = b' ';
            msg_len += 1;
        }

        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => continue,
        };

        for &c in arg {
            if msg_len < message.len() - 1 {
                message[msg_len] = c;
                msg_len += 1;
            }
        }
    }

    // Write to /dev/log or stderr as fallback
    let fd = io::open(b"/dev/log", libc::O_WRONLY, 0);
    if fd >= 0 {
        if let Some(t) = tag {
            io::write_all(fd, t);
            io::write_str(fd, b": ");
        }
        io::write_all(fd, &message[..msg_len]);
        io::write_str(fd, b"\n");
        io::close(fd);
    } else {
        // Fallback to stderr
        if let Some(t) = tag {
            io::write_all(2, t);
            io::write_str(2, b": ");
        }
        io::write_all(2, &message[..msg_len]);
        io::write_str(2, b"\n");
    }

    0
}

/// mountpoint - check if directory is a mountpoint
pub fn mountpoint(argc: i32, argv: *const *const u8) -> i32 {
    let mut quiet = false;
    let mut path_idx = 1;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if has_opt(arg, b'q') {
            quiet = true;
            path_idx = i + 1;
        } else if arg[0] != b'-' {
            path_idx = i;
            break;
        }
    }

    if path_idx >= argc {
        io::write_str(2, b"mountpoint: missing path\n");
        return 1;
    }

    let path = match unsafe { get_arg(argv, path_idx) } {
        Some(p) => p,
        None => return 1,
    };

    // Check if path is a mountpoint by comparing device IDs
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    if io::stat(path, &mut st) != 0 {
        if !quiet {
            sys::perror(path);
        }
        return 1;
    }

    let path_dev = st.st_dev;

    // Check parent directory
    let mut parent = [0u8; 4096];
    let mut len = 0;
    for &c in path { if len < parent.len() - 1 { parent[len] = c; len += 1; } }
    if len > 0 && parent[len - 1] != b'/' { parent[len] = b'/'; len += 1; }
    parent[len] = b'.';
    parent[len + 1] = b'.';
    len += 2;

    if io::stat(&parent[..len], &mut st) != 0 {
        // Root is always a mountpoint
        if !quiet {
            io::write_all(1, path);
            io::write_str(1, b" is a mountpoint\n");
        }
        return 0;
    }

    if st.st_dev != path_dev {
        if !quiet {
            io::write_all(1, path);
            io::write_str(1, b" is a mountpoint\n");
        }
        0
    } else {
        if !quiet {
            io::write_all(1, path);
            io::write_str(1, b" is not a mountpoint\n");
        }
        1
    }
}

/// sysctl - read/write kernel parameters
pub fn sysctl(argc: i32, argv: *const *const u8) -> i32 {
    let mut show_all = false;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if has_opt(arg, b'a') {
            show_all = true;
        } else if arg[0] != b'-' {
            // Read/write specific parameter
            return sysctl_rw(arg);
        }
    }

    if show_all {
        return sysctl_show_all(b"/proc/sys", &mut [0u8; 256], 0);
    }

    0
}

fn sysctl_rw(param: &[u8]) -> i32 {
    // Check for assignment
    if let Some(eq_pos) = param.iter().position(|&c| c == b'=') {
        let key = &param[..eq_pos];
        let value = &param[eq_pos + 1..];

        // Convert key to path (replace . with /)
        let mut path = [0u8; 256];
        let mut len = 0;
        for &c in b"/proc/sys/" { if len < path.len() - 1 { path[len] = *c; len += 1; } }
        for &c in key {
            if len < path.len() - 1 {
                path[len] = if c == b'.' { b'/' } else { c };
                len += 1;
            }
        }

        let fd = io::open(&path[..len], libc::O_WRONLY, 0);
        if fd < 0 {
            sys::perror(key);
            return 1;
        }

        io::write_all(fd, value);
        io::close(fd);

        io::write_all(1, key);
        io::write_str(1, b" = ");
        io::write_all(1, value);
        io::write_str(1, b"\n");
    } else {
        // Read parameter
        let mut path = [0u8; 256];
        let mut len = 0;
        for &c in b"/proc/sys/" { if len < path.len() - 1 { path[len] = *c; len += 1; } }
        for &c in param {
            if len < path.len() - 1 {
                path[len] = if c == b'.' { b'/' } else { c };
                len += 1;
            }
        }

        let fd = io::open(&path[..len], libc::O_RDONLY, 0);
        if fd < 0 {
            sys::perror(param);
            return 1;
        }

        let mut value = [0u8; 4096];
        let n = io::read(fd, &mut value);
        io::close(fd);

        io::write_all(1, param);
        io::write_str(1, b" = ");
        if n > 0 {
            let val_len = value[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);
            io::write_all(1, &value[..val_len]);
        }
        io::write_str(1, b"\n");
    }

    0
}

fn sysctl_show_all(dir: &[u8], key_buf: &mut [u8], key_len: usize) -> i32 {
    let d = io::opendir(dir);
    if d.is_null() {
        return 0;
    }

    loop {
        let entry = io::readdir(d);
        if entry.is_null() {
            break;
        }

        let (name_slice, name_len) = unsafe { io::dirent_name(entry) };

        if (name_len == 1 && name_slice[0] == b'.') ||
           (name_len == 2 && name_slice[0] == b'.' && name_slice[1] == b'.') {
            continue;
        }

        // Build path
        let mut path = [0u8; 256];
        let mut len = 0;
        for &c in dir { if len < path.len() - 1 { path[len] = *c; len += 1; } }
        if len > 0 && path[len - 1] != b'/' { path[len] = b'/'; len += 1; }
        for &c in name_slice { if len < path.len() - 1 { path[len] = *c; len += 1; } }

        // Build key
        let mut new_key_len = key_len;
        if key_len > 0 {
            key_buf[new_key_len] = b'.';
            new_key_len += 1;
        }
        for &c in name_slice {
            if new_key_len < key_buf.len() - 1 {
                key_buf[new_key_len] = c;
                new_key_len += 1;
            }
        }

        let mut st: libc::stat = unsafe { core::mem::zeroed() };
        if io::stat(&path[..len], &mut st) == 0 {
            if (st.st_mode & libc::S_IFMT) == libc::S_IFDIR {
                sysctl_show_all(&path[..len], key_buf, new_key_len);
            } else {
                // Read and print value
                let fd = io::open(&path[..len], libc::O_RDONLY, 0);
                if fd >= 0 {
                    let mut value = [0u8; 256];
                    let n = io::read(fd, &mut value);
                    io::close(fd);

                    io::write_all(1, &key_buf[..new_key_len]);
                    io::write_str(1, b" = ");
                    if n > 0 {
                        let val_len = value[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);
                        io::write_all(1, &value[..val_len]);
                    }
                    io::write_str(1, b"\n");
                }
            }
        }
    }

    io::closedir(d);
    0
}

/// swapoff - disable swap
pub fn swapoff(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"swapoff: usage: swapoff <device>\n");
        return 1;
    }

    let device = match unsafe { get_arg(argv, 1) } {
        Some(d) => d,
        None => return 1,
    };

    let mut path_buf = [0u8; 256];
    if device.len() >= path_buf.len() {
        return 1;
    }
    path_buf[..device.len()].copy_from_slice(device);
    path_buf[device.len()] = 0;

    if unsafe { libc::swapoff(path_buf.as_ptr() as *const i8) } != 0 {
        sys::perror(device);
        return 1;
    }

    0
}

/// swapon - enable swap
pub fn swapon(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        // Show current swap status
        let fd = io::open(b"/proc/swaps", libc::O_RDONLY, 0);
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

    let device = match unsafe { get_arg(argv, 1) } {
        Some(d) => d,
        None => return 1,
    };

    let mut path_buf = [0u8; 256];
    if device.len() >= path_buf.len() {
        return 1;
    }
    path_buf[..device.len()].copy_from_slice(device);
    path_buf[device.len()] = 0;

    if unsafe { libc::swapon(path_buf.as_ptr() as *const i8, 0) } != 0 {
        sys::perror(device);
        return 1;
    }

    0
}

/// chvt - change foreground virtual terminal
pub fn chvt(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"chvt: usage: chvt N\n");
        return 1;
    }

    let vt = match unsafe { get_arg(argv, 1) }.and_then(sys::parse_u64) {
        Some(v) => v as i32,
        None => {
            io::write_str(2, b"chvt: invalid VT number\n");
            return 1;
        }
    };

    let fd = io::open(b"/dev/console", libc::O_RDWR, 0);
    if fd < 0 {
        sys::perror(b"/dev/console");
        return 1;
    }

    // VT_ACTIVATE = 0x5606
    const VT_ACTIVATE: u64 = 0x5606;
    // VT_WAITACTIVE = 0x5607
    const VT_WAITACTIVE: u64 = 0x5607;

    if unsafe { libc::ioctl(fd, VT_ACTIVATE, vt) } != 0 {
        sys::perror(b"VT_ACTIVATE");
        io::close(fd);
        return 1;
    }

    if unsafe { libc::ioctl(fd, VT_WAITACTIVE, vt) } != 0 {
        sys::perror(b"VT_WAITACTIVE");
        io::close(fd);
        return 1;
    }

    io::close(fd);
    0
}

/// fgconsole - print foreground VT number
pub fn fgconsole(_argc: i32, _argv: *const *const u8) -> i32 {
    let fd = io::open(b"/dev/console", libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(b"/dev/console");
        return 1;
    }

    // VT_GETSTATE = 0x5603
    const VT_GETSTATE: u64 = 0x5603;

    #[repr(C)]
    struct VtState {
        v_active: u16,
        v_signal: u16,
        v_state: u16,
    }

    let mut state: VtState = unsafe { core::mem::zeroed() };
    if unsafe { libc::ioctl(fd, VT_GETSTATE, &mut state as *mut VtState) } != 0 {
        sys::perror(b"VT_GETSTATE");
        io::close(fd);
        return 1;
    }

    io::close(fd);
    io::write_num(1, state.v_active as u64);
    io::write_str(1, b"\n");
    0
}

/// pgrep - look up processes by name/pattern
pub fn pgrep(argc: i32, argv: *const *const u8) -> i32 {
    let mut full = false;
    let mut pattern_idx = 1;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if has_opt(arg, b'f') {
            full = true;
            pattern_idx = i + 1;
        } else if arg[0] != b'-' {
            pattern_idx = i;
            break;
        } else {
            pattern_idx = i + 1;
        }
    }

    if pattern_idx >= argc {
        io::write_str(2, b"pgrep: missing pattern\n");
        return 1;
    }

    let pattern = match unsafe { get_arg(argv, pattern_idx) } {
        Some(p) => p,
        None => return 1,
    };

    let mut found = false;
    let dir = io::opendir(b"/proc");
    if dir.is_null() {
        return 1;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let (name_slice, _) = unsafe { io::dirent_name(entry) };

        if !name_slice.iter().all(|&c| c >= b'0' && c <= b'9') {
            continue;
        }

        // Read comm or cmdline
        let mut comm_path = [0u8; 64];
        let mut len = 0;
        for &c in b"/proc/" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in name_slice { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        if full {
            for &c in b"/cmdline" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        } else {
            for &c in b"/comm" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        }

        let fd = io::open(&comm_path[..len], libc::O_RDONLY, 0);
        if fd < 0 {
            continue;
        }

        let mut comm = [0u8; 256];
        let n = io::read(fd, &mut comm);
        io::close(fd);

        if n <= 0 {
            continue;
        }

        let comm_len = comm[..n as usize].iter().position(|&c| c == b'\n' || c == 0).unwrap_or(n as usize);

        // Simple substring match
        if contains_pattern(&comm[..comm_len], pattern) {
            io::write_all(1, name_slice);
            io::write_str(1, b"\n");
            found = true;
        }
    }

    io::closedir(dir);
    if found { 0 } else { 1 }
}

/// pkill - signal processes by name/pattern
pub fn pkill(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = 15i32;
    let mut pattern_idx = 1;

    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => break,
        };

        if arg[0] == b'-' && arg.len() > 1 && arg[1] >= b'0' && arg[1] <= b'9' {
            signal = sys::parse_i64(&arg[1..]).unwrap_or(15) as i32;
            pattern_idx = i + 1;
        } else if arg[0] != b'-' {
            pattern_idx = i;
            break;
        } else {
            pattern_idx = i + 1;
        }
    }

    if pattern_idx >= argc {
        io::write_str(2, b"pkill: missing pattern\n");
        return 1;
    }

    let pattern = match unsafe { get_arg(argv, pattern_idx) } {
        Some(p) => p,
        None => return 1,
    };

    let mut found = false;
    let dir = io::opendir(b"/proc");
    if dir.is_null() {
        return 1;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() {
            break;
        }

        let (name_slice, _) = unsafe { io::dirent_name(entry) };

        if !name_slice.iter().all(|&c| c >= b'0' && c <= b'9') {
            continue;
        }

        let mut comm_path = [0u8; 64];
        let mut len = 0;
        for &c in b"/proc/" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in name_slice { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }
        for &c in b"/comm" { if len < comm_path.len() - 1 { comm_path[len] = *c; len += 1; } }

        let fd = io::open(&comm_path[..len], libc::O_RDONLY, 0);
        if fd < 0 {
            continue;
        }

        let mut comm = [0u8; 256];
        let n = io::read(fd, &mut comm);
        io::close(fd);

        if n <= 0 {
            continue;
        }

        let comm_len = comm[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);

        if contains_pattern(&comm[..comm_len], pattern) {
            let pid = sys::parse_i64(name_slice).unwrap_or(0) as i32;
            if pid > 0 {
                io::kill(pid, signal);
                found = true;
            }
        }
    }

    io::closedir(dir);
    if found { 0 } else { 1 }
}

fn contains_pattern(text: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return true;
    }
    if pattern.len() > text.len() {
        return false;
    }
    for i in 0..=(text.len() - pattern.len()) {
        if io::bytes_eq(&text[i..i + pattern.len()], pattern) {
            return true;
        }
    }
    false
}

// ============================================================================
// BLKID - Identify block devices by UUID/LABEL
// ============================================================================

/// blkid - locate/print block device attributes
pub fn blkid(argc: i32, argv: *const *const u8) -> i32 {
    // If a device is specified, show info for that device
    if argc > 1 {
        for i in 1..argc {
            let dev = match unsafe { get_arg(argv, i) } {
                Some(d) => d,
                None => continue,
            };
            if dev.starts_with(b"-") { continue; }
            blkid_show_device(dev);
        }
        return 0;
    }

    // Otherwise, probe all block devices
    let dir = io::opendir(b"/sys/block");
    if dir.is_null() {
        return 1;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() { break; }

        let (name, _) = unsafe { io::dirent_name(entry) };
        if name.starts_with(b".") { continue; }

        // Build /dev/NAME path
        let mut dev_path = [0u8; 128];
        let mut len = 0;
        for &c in b"/dev/" { if len < dev_path.len() - 1 { dev_path[len] = *c; len += 1; } }
        for &c in name { if len < dev_path.len() - 1 { dev_path[len] = *c; len += 1; } }

        blkid_show_device(&dev_path[..len]);
    }

    io::closedir(dir);
    0
}

fn blkid_show_device(dev: &[u8]) {
    io::write_all(1, dev);

    // Try to read UUID from /sys/block/DEV/uuid or look for LABEL
    // For now, just show the device path
    let mut uuid_path = [0u8; 256];
    let mut len = 0;

    // Extract device name from path
    let name = dev.rsplit(|&c| c == b'/').next().unwrap_or(dev);

    for &c in b"/sys/block/" { if len < uuid_path.len() - 1 { uuid_path[len] = *c; len += 1; } }
    for &c in name { if len < uuid_path.len() - 1 { uuid_path[len] = *c; len += 1; } }
    for &c in b"/uuid" { if len < uuid_path.len() - 1 { uuid_path[len] = *c; len += 1; } }

    let fd = io::open(&uuid_path[..len], libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut uuid = [0u8; 64];
        let n = io::read(fd, &mut uuid);
        io::close(fd);
        if n > 0 {
            io::write_str(1, b" UUID=\"");
            let uuid_len = uuid[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);
            io::write_all(1, &uuid[..uuid_len]);
            io::write_str(1, b"\"");
        }
    }

    // Try to detect filesystem type by reading superblock
    let fd = io::open(dev, libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);

        if n >= 1024 {
            // Check for ext2/3/4 magic at offset 0x438
            if buf[0x438] == 0x53 && buf[0x439] == 0xEF {
                io::write_str(1, b" TYPE=\"ext4\"");
            }
        }
    }

    io::write_str(1, b"\n");
}

// ============================================================================
// LOSETUP - Setup loop devices
// ============================================================================

/// losetup - set up and control loop devices
pub fn losetup(argc: i32, argv: *const *const u8) -> i32 {
    let mut detach = false;
    let mut find_free = false;
    let mut loop_dev: Option<&[u8]> = None;
    let mut file_path: Option<&[u8]> = None;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => { i += 1; continue; }
        };

        if arg == b"-d" {
            detach = true;
            i += 1;
            loop_dev = unsafe { get_arg(argv, i) };
        } else if arg == b"-f" {
            find_free = true;
        } else if !arg.starts_with(b"-") {
            if loop_dev.is_none() {
                loop_dev = Some(arg);
            } else {
                file_path = Some(arg);
            }
        }
        i += 1;
    }

    if detach {
        if let Some(dev) = loop_dev {
            return losetup_detach(dev);
        }
        io::write_str(2, b"losetup: -d requires device\n");
        return 1;
    }

    if find_free && file_path.is_none() && loop_dev.is_none() {
        // Just find and print free loop device
        return losetup_find_free();
    }

    if let (Some(dev), Some(file)) = (loop_dev, file_path) {
        return losetup_attach(dev, file);
    }

    if let Some(file) = loop_dev.or(file_path) {
        // -f with file: find free loop and attach
        if find_free {
            return losetup_attach_free(file);
        }
    }

    // List all loop devices
    losetup_list()
}

fn losetup_find_free() -> i32 {
    for i in 0..256 {
        let mut path = [0u8; 32];
        let mut len = 0;
        for &c in b"/dev/loop" { if len < path.len() - 1 { path[len] = *c; len += 1; } }

        let mut num_buf = [0u8; 8];
        let num_str = sys::format_u64(i, &mut num_buf);
        for &c in num_str { if len < path.len() - 1 { path[len] = *c; len += 1; } }

        // Check if device exists and is free
        let fd = io::open(&path[..len], libc::O_RDONLY, 0);
        if fd >= 0 {
            // Try LOOP_GET_STATUS - if it fails with ENXIO, device is free
            let mut info: [u8; 128] = [0; 128];
            let ret = unsafe { libc::ioctl(fd, 0x4C05, info.as_mut_ptr()) }; // LOOP_GET_STATUS64
            io::close(fd);

            if ret < 0 {
                io::write_all(1, &path[..len]);
                io::write_str(1, b"\n");
                return 0;
            }
        }
    }
    io::write_str(2, b"losetup: no free loop device found\n");
    1
}

fn losetup_attach(loop_dev: &[u8], file: &[u8]) -> i32 {
    // Open loop device
    let loop_fd = io::open(loop_dev, libc::O_RDWR, 0);
    if loop_fd < 0 {
        io::write_str(2, b"losetup: cannot open loop device\n");
        return 1;
    }

    // Open backing file
    let file_fd = io::open(file, libc::O_RDONLY, 0);
    if file_fd < 0 {
        io::close(loop_fd);
        io::write_str(2, b"losetup: cannot open file\n");
        return 1;
    }

    // Set backing file
    let ret = unsafe { libc::ioctl(loop_fd, 0x4C00, file_fd) }; // LOOP_SET_FD
    io::close(file_fd);
    io::close(loop_fd);

    if ret < 0 {
        io::write_str(2, b"losetup: LOOP_SET_FD failed\n");
        return 1;
    }

    0
}

fn losetup_attach_free(file: &[u8]) -> i32 {
    for i in 0..256 {
        let mut path = [0u8; 32];
        let mut len = 0;
        for &c in b"/dev/loop" { if len < path.len() - 1 { path[len] = *c; len += 1; } }

        let mut num_buf = [0u8; 8];
        let num_str = sys::format_u64(i, &mut num_buf);
        for &c in num_str { if len < path.len() - 1 { path[len] = *c; len += 1; } }

        let fd = io::open(&path[..len], libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut info: [u8; 128] = [0; 128];
            let ret = unsafe { libc::ioctl(fd, 0x4C05, info.as_mut_ptr()) };
            io::close(fd);

            if ret < 0 {
                // Device is free
                let result = losetup_attach(&path[..len], file);
                if result == 0 {
                    io::write_all(1, &path[..len]);
                    io::write_str(1, b"\n");
                }
                return result;
            }
        }
    }
    io::write_str(2, b"losetup: no free loop device found\n");
    1
}

fn losetup_detach(dev: &[u8]) -> i32 {
    let fd = io::open(dev, libc::O_RDONLY, 0);
    if fd < 0 {
        io::write_str(2, b"losetup: cannot open device\n");
        return 1;
    }

    let ret = unsafe { libc::ioctl(fd, 0x4C01, 0) }; // LOOP_CLR_FD
    io::close(fd);

    if ret < 0 { 1 } else { 0 }
}

fn losetup_list() -> i32 {
    io::write_str(1, b"NAME       SIZELIMIT OFFSET AUTOCLEAR BACK-FILE\n");

    for i in 0..256 {
        let mut path = [0u8; 32];
        let mut len = 0;
        for &c in b"/dev/loop" { if len < path.len() - 1 { path[len] = *c; len += 1; } }

        let mut num_buf = [0u8; 8];
        let num_str = sys::format_u64(i, &mut num_buf);
        for &c in num_str { if len < path.len() - 1 { path[len] = *c; len += 1; } }

        let fd = io::open(&path[..len], libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut info: [u8; 128] = [0; 128];
            let ret = unsafe { libc::ioctl(fd, 0x4C05, info.as_mut_ptr()) };
            io::close(fd);

            if ret == 0 {
                io::write_all(1, &path[..len]);
                io::write_str(1, b"\n");
            }
        }
    }
    0
}

// ============================================================================
// INSMOD / RMMOD / MODPROBE - Kernel module management
// ============================================================================

/// insmod - insert a module into the kernel
pub fn insmod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"Usage: insmod MODULE [PARAMS...]\n");
        return 1;
    }

    let module_path = match unsafe { get_arg(argv, 1) } {
        Some(p) => p,
        None => return 1,
    };

    // Read module file
    let fd = io::open(module_path, libc::O_RDONLY, 0);
    if fd < 0 {
        io::write_str(2, b"insmod: cannot open module\n");
        return 1;
    }

    // Get file size
    let size = unsafe { libc::lseek(fd, 0, libc::SEEK_END) } as usize;
    unsafe { libc::lseek(fd, 0, libc::SEEK_SET) };

    // Allocate buffer
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;
        let mut buf = Vec::with_capacity(size);
        buf.resize(size, 0);

        io::read(fd, &mut buf);
        io::close(fd);

        // Build params string
        let mut params = Vec::new();
        for i in 2..argc {
            if let Some(p) = unsafe { get_arg(argv, i) } {
                if !params.is_empty() { params.push(b' '); }
                params.extend_from_slice(p);
            }
        }
        params.push(0);

        let ret = unsafe {
            libc::syscall(
                libc::SYS_init_module,
                buf.as_ptr(),
                buf.len(),
                params.as_ptr(),
            )
        };

        if ret < 0 { 1 } else { 0 }
    }

    #[cfg(not(feature = "alloc"))]
    {
        io::close(fd);
        io::write_str(2, b"insmod: requires alloc feature\n");
        1
    }
}

/// rmmod - remove a module from the kernel
pub fn rmmod(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"Usage: rmmod MODULE\n");
        return 1;
    }

    let module_name = match unsafe { get_arg(argv, 1) } {
        Some(n) => n,
        None => return 1,
    };

    let mut name_buf = [0u8; 256];
    let len = module_name.len().min(255);
    name_buf[..len].copy_from_slice(&module_name[..len]);

    let ret = unsafe {
        libc::syscall(
            libc::SYS_delete_module,
            name_buf.as_ptr(),
            0u32, // O_NONBLOCK
        )
    };

    if ret < 0 {
        io::write_str(2, b"rmmod: failed to remove module\n");
        1
    } else {
        0
    }
}

/// modprobe - add or remove modules from the kernel
pub fn modprobe(argc: i32, argv: *const *const u8) -> i32 {
    let mut remove = false;
    let mut module_name: Option<&[u8]> = None;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => { i += 1; continue; }
        };

        if arg == b"-r" {
            remove = true;
        } else if !arg.starts_with(b"-") {
            module_name = Some(arg);
            break;
        }
        i += 1;
    }

    let module = match module_name {
        Some(m) => m,
        None => {
            io::write_str(2, b"modprobe: missing module name\n");
            return 1;
        }
    };

    if remove {
        // Create argv for rmmod
        let rmmod_argv: [*const u8; 3] = [
            b"rmmod\0".as_ptr(),
            module.as_ptr(),
            core::ptr::null(),
        ];
        return rmmod(2, rmmod_argv.as_ptr());
    }

    // Find module in /lib/modules/$(uname -r)/
    let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
    if io::uname(&mut uts) != 0 {
        return 1;
    }

    let release_len = uts.release.iter().position(|&c| c == 0).unwrap_or(uts.release.len());

    let mut module_path = [0u8; 512];
    let mut len = 0;
    for &c in b"/lib/modules/" { if len < module_path.len() - 1 { module_path[len] = *c; len += 1; } }
    for i in 0..release_len { if len < module_path.len() - 1 { module_path[len] = uts.release[i] as u8; len += 1; } }
    for &c in b"/" { if len < module_path.len() - 1 { module_path[len] = *c; len += 1; } }
    for &c in module { if len < module_path.len() - 1 { module_path[len] = *c; len += 1; } }
    for &c in b".ko" { if len < module_path.len() - 1 { module_path[len] = *c; len += 1; } }

    // Create argv for insmod
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;
        let mut insmod_argv: Vec<*const u8> = Vec::new();
        insmod_argv.push(b"insmod\0".as_ptr());
        insmod_argv.push(module_path.as_ptr());
        insmod_argv.push(core::ptr::null());
        return insmod(2, insmod_argv.as_ptr());
    }

    #[cfg(not(feature = "alloc"))]
    {
        io::write_str(2, b"modprobe: requires alloc feature\n");
        1
    }
}

// ============================================================================
// DNSDOMAINNAME - Show DNS domain name
// ============================================================================

/// dnsdomainname - show the system's DNS domain name
pub fn dnsdomainname(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
    if io::uname(&mut uts) != 0 {
        return 1;
    }

    // Get the domainname
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        let len = uts.domainname.iter().position(|&c| c == 0).unwrap_or(uts.domainname.len());
        if len > 0 && uts.domainname[0] != b'(' as i8 {
            io::write_all(1, unsafe { core::slice::from_raw_parts(uts.domainname.as_ptr() as *const u8, len) });
            io::write_str(1, b"\n");
        }
    }

    0
}

// ============================================================================
// FLOCK - File locking
// ============================================================================

/// flock - manage file locks from shell scripts
pub fn flock(argc: i32, argv: *const *const u8) -> i32 {
    let mut exclusive = true;
    let mut nonblock = false;
    let mut unlock = false;
    let mut fd_arg: Option<i32> = None;
    let mut file_arg: Option<&[u8]> = None;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => { i += 1; continue; }
        };

        if arg == b"-s" || arg == b"--shared" {
            exclusive = false;
        } else if arg == b"-x" || arg == b"--exclusive" {
            exclusive = true;
        } else if arg == b"-n" || arg == b"--nonblock" {
            nonblock = true;
        } else if arg == b"-u" || arg == b"--unlock" {
            unlock = true;
        } else if !arg.starts_with(b"-") {
            // Check if it's a file descriptor number
            if let Some(fd) = sys::parse_i64(arg) {
                fd_arg = Some(fd as i32);
            } else {
                file_arg = Some(arg);
            }
            break;
        }
        i += 1;
    }

    let fd = if let Some(fd) = fd_arg {
        fd
    } else if let Some(file) = file_arg {
        let f = io::open(file, libc::O_RDWR | libc::O_CREAT, 0o644);
        if f < 0 {
            io::write_str(2, b"flock: cannot open file\n");
            return 1;
        }
        f
    } else {
        io::write_str(2, b"flock: missing file or fd\n");
        return 1;
    };

    let mut operation = if unlock {
        libc::LOCK_UN
    } else if exclusive {
        libc::LOCK_EX
    } else {
        libc::LOCK_SH
    };

    if nonblock {
        operation |= libc::LOCK_NB;
    }

    let ret = unsafe { libc::flock(fd, operation) };

    if file_arg.is_some() {
        io::close(fd);
    }

    if ret < 0 { 1 } else { 0 }
}

// ============================================================================
// FSYNC - Synchronize file state
// ============================================================================

/// fsync - synchronize a file's state with storage device
pub fn fsync_cmd(argc: i32, argv: *const *const u8) -> i32 {
    let mut data_only = false;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => { i += 1; continue; }
        };

        if arg == b"-d" || arg == b"--data" {
            data_only = true;
        } else if !arg.starts_with(b"-") {
            let fd = io::open(arg, libc::O_RDONLY, 0);
            if fd < 0 {
                io::write_str(2, b"fsync: cannot open file\n");
                return 1;
            }

            let ret = if data_only {
                unsafe { libc::fdatasync(fd) }
            } else {
                unsafe { libc::fsync(fd) }
            };

            io::close(fd);

            if ret < 0 {
                return 1;
            }
        }
        i += 1;
    }

    0
}

// ============================================================================
// PIVOT_ROOT - Change root filesystem
// ============================================================================

/// pivot_root - change the root filesystem
pub fn pivot_root(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"Usage: pivot_root new_root put_old\n");
        return 1;
    }

    let new_root = match unsafe { get_arg(argv, 1) } {
        Some(p) => p,
        None => return 1,
    };

    let put_old = match unsafe { get_arg(argv, 2) } {
        Some(p) => p,
        None => return 1,
    };

    let mut new_root_buf = [0u8; 512];
    let mut put_old_buf = [0u8; 512];

    let nr_len = new_root.len().min(511);
    let po_len = put_old.len().min(511);

    new_root_buf[..nr_len].copy_from_slice(&new_root[..nr_len]);
    put_old_buf[..po_len].copy_from_slice(&put_old[..po_len]);

    let ret = unsafe {
        libc::syscall(
            libc::SYS_pivot_root,
            new_root_buf.as_ptr(),
            put_old_buf.as_ptr(),
        )
    };

    if ret < 0 {
        io::write_str(2, b"pivot_root: failed\n");
        1
    } else {
        0
    }
}

// ============================================================================
// READAHEAD - Preload files into page cache
// ============================================================================

/// readahead - preload files into page cache
pub fn readahead_cmd(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => continue,
        };

        if arg.starts_with(b"-") { continue; }

        let fd = io::open(arg, libc::O_RDONLY, 0);
        if fd < 0 { continue; }

        // Get file size
        let size = unsafe { libc::lseek(fd, 0, libc::SEEK_END) };
        if size > 0 {
            unsafe { libc::lseek(fd, 0, libc::SEEK_SET) };
            unsafe { libc::readahead(fd, 0, size as usize) };
        }

        io::close(fd);
    }

    0
}

// ============================================================================
// TASKSET - Set/get CPU affinity
// ============================================================================

/// taskset - set or retrieve a process's CPU affinity
pub fn taskset(argc: i32, argv: *const *const u8) -> i32 {
    let mut pid: Option<i32> = None;
    let mut mask: Option<u64> = None;
    let mut get_mode = false;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => { i += 1; continue; }
        };

        if arg == b"-p" {
            i += 1;
            if let Some(p) = unsafe { get_arg(argv, i) } {
                pid = sys::parse_i64(p).map(|n| n as i32);
            }
            get_mode = true;
        } else if !arg.starts_with(b"-") {
            if mask.is_none() {
                // Parse hex mask
                if arg.starts_with(b"0x") || arg.starts_with(b"0X") {
                    mask = u64::from_str_radix(
                        core::str::from_utf8(&arg[2..]).unwrap_or("0"),
                        16
                    ).ok();
                } else {
                    mask = sys::parse_u64(arg);
                }
            } else if pid.is_none() {
                pid = sys::parse_i64(arg).map(|n| n as i32);
            }
        }
        i += 1;
    }

    let target_pid = pid.unwrap_or(0);

    if get_mode && mask.is_none() {
        // Get affinity
        let mut cpu_set: libc::cpu_set_t = unsafe { core::mem::zeroed() };
        let ret = unsafe {
            libc::sched_getaffinity(
                target_pid,
                core::mem::size_of::<libc::cpu_set_t>(),
                &mut cpu_set,
            )
        };

        if ret < 0 {
            io::write_str(2, b"taskset: failed to get affinity\n");
            return 1;
        }

        io::write_str(1, b"pid ");
        io::write_num(1, target_pid as u64);
        io::write_str(1, b"'s current affinity mask: ");

        // Print mask as hex
        let mut mask_val: u64 = 0;
        for i in 0..64 {
            if unsafe { libc::CPU_ISSET(i, &cpu_set) } {
                mask_val |= 1 << i;
            }
        }
        io::write_str(1, b"0x");
        let mut buf = [0u8; 20];
        let s = format_hex(mask_val, &mut buf);
        io::write_all(1, s);
        io::write_str(1, b"\n");

        return 0;
    }

    if let Some(m) = mask {
        // Set affinity
        let mut cpu_set: libc::cpu_set_t = unsafe { core::mem::zeroed() };
        unsafe { libc::CPU_ZERO(&mut cpu_set); }

        for i in 0..64 {
            if m & (1 << i) != 0 {
                unsafe { libc::CPU_SET(i, &mut cpu_set); }
            }
        }

        let ret = unsafe {
            libc::sched_setaffinity(
                target_pid,
                core::mem::size_of::<libc::cpu_set_t>(),
                &cpu_set,
            )
        };

        if ret < 0 {
            io::write_str(2, b"taskset: failed to set affinity\n");
            return 1;
        }

        return 0;
    }

    io::write_str(2, b"Usage: taskset [OPTIONS] MASK PID\n");
    1
}

fn format_hex(val: u64, buf: &mut [u8]) -> &[u8] {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut n = val;
    let mut i = buf.len();

    if n == 0 {
        if i > 0 {
            i -= 1;
            buf[i] = b'0';
        }
        return &buf[i..];
    }

    while n > 0 && i > 0 {
        i -= 1;
        buf[i] = HEX[(n & 0xf) as usize];
        n >>= 4;
    }

    &buf[i..]
}

// ============================================================================
// RFKILL - Control wireless devices
// ============================================================================

/// rfkill - tool for enabling/disabling wireless devices
pub fn rfkill(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        return rfkill_list();
    }

    let cmd = match unsafe { get_arg(argv, 1) } {
        Some(c) => c,
        None => return rfkill_list(),
    };

    match cmd {
        b"list" => rfkill_list(),
        b"block" => {
            if argc < 3 {
                io::write_str(2, b"rfkill: block requires device\n");
                return 1;
            }
            let dev = unsafe { get_arg(argv, 2) }.unwrap_or(b"all");
            rfkill_block(dev, true)
        }
        b"unblock" => {
            if argc < 3 {
                io::write_str(2, b"rfkill: unblock requires device\n");
                return 1;
            }
            let dev = unsafe { get_arg(argv, 2) }.unwrap_or(b"all");
            rfkill_block(dev, false)
        }
        _ => {
            io::write_str(2, b"Usage: rfkill [list|block|unblock] [device]\n");
            1
        }
    }
}

fn rfkill_list() -> i32 {
    io::write_str(1, b"ID TYPE      SOFT      HARD\n");

    let dir = io::opendir(b"/sys/class/rfkill");
    if dir.is_null() {
        return 1;
    }

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() { break; }

        let (name, _) = unsafe { io::dirent_name(entry) };
        if name.starts_with(b".") { continue; }
        if !name.starts_with(b"rfkill") { continue; }

        // Read type
        let mut type_path = [0u8; 128];
        let mut len = 0;
        for &c in b"/sys/class/rfkill/" { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }
        for &c in name { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }
        for &c in b"/type" { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }

        let fd = io::open(&type_path[..len], libc::O_RDONLY, 0);
        if fd < 0 { continue; }

        let mut type_buf = [0u8; 32];
        let n = io::read(fd, &mut type_buf);
        io::close(fd);

        // Print ID
        io::write_all(1, &name[6..]); // Skip "rfkill"
        io::write_str(1, b"  ");

        // Print type
        if n > 0 {
            let type_len = type_buf[..n as usize].iter().position(|&c| c == b'\n').unwrap_or(n as usize);
            io::write_all(1, &type_buf[..type_len]);
        }
        for _ in 0..(10 - n as usize) { io::write_str(1, b" "); }

        // Read soft
        len = 0;
        for &c in b"/sys/class/rfkill/" { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }
        for &c in name { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }
        for &c in b"/soft" { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }

        let fd = io::open(&type_path[..len], libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut soft_buf = [0u8; 4];
            let n = io::read(fd, &mut soft_buf);
            io::close(fd);
            if n > 0 && soft_buf[0] == b'1' {
                io::write_str(1, b"blocked   ");
            } else {
                io::write_str(1, b"unblocked ");
            }
        }

        // Read hard
        len = 0;
        for &c in b"/sys/class/rfkill/" { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }
        for &c in name { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }
        for &c in b"/hard" { if len < type_path.len() - 1 { type_path[len] = *c; len += 1; } }

        let fd = io::open(&type_path[..len], libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut hard_buf = [0u8; 4];
            let n = io::read(fd, &mut hard_buf);
            io::close(fd);
            if n > 0 && hard_buf[0] == b'1' {
                io::write_str(1, b"blocked");
            } else {
                io::write_str(1, b"unblocked");
            }
        }

        io::write_str(1, b"\n");
    }

    io::closedir(dir);
    0
}

fn rfkill_block(dev: &[u8], block: bool) -> i32 {
    let dir = io::opendir(b"/sys/class/rfkill");
    if dir.is_null() {
        return 1;
    }

    let block_val = if block { b"1" } else { b"0" };

    loop {
        let entry = io::readdir(dir);
        if entry.is_null() { break; }

        let (name, _) = unsafe { io::dirent_name(entry) };
        if name.starts_with(b".") { continue; }
        if !name.starts_with(b"rfkill") { continue; }

        // Check if this matches the requested device
        if dev != b"all" && &name[6..] != dev {
            continue;
        }

        let mut soft_path = [0u8; 128];
        let mut len = 0;
        for &c in b"/sys/class/rfkill/" { if len < soft_path.len() - 1 { soft_path[len] = *c; len += 1; } }
        for &c in name { if len < soft_path.len() - 1 { soft_path[len] = *c; len += 1; } }
        for &c in b"/soft" { if len < soft_path.len() - 1 { soft_path[len] = *c; len += 1; } }

        let fd = io::open(&soft_path[..len], libc::O_WRONLY, 0);
        if fd >= 0 {
            io::write_all(fd, block_val);
            io::close(fd);
        }
    }

    io::closedir(dir);
    0
}

// ============================================================================
// IONICE - Set/get I/O scheduling class and priority
// ============================================================================

/// ionice - set or get I/O scheduling class and priority
pub fn ionice(argc: i32, argv: *const *const u8) -> i32 {
    let mut class: Option<i32> = None;
    let mut data: i32 = 4;
    let mut pid: i32 = 0;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => { i += 1; continue; }
        };

        if arg == b"-c" {
            i += 1;
            if let Some(c) = unsafe { get_arg(argv, i) } {
                class = sys::parse_i64(c).map(|n| n as i32);
            }
        } else if arg == b"-n" {
            i += 1;
            if let Some(n) = unsafe { get_arg(argv, i) } {
                data = sys::parse_i64(n).unwrap_or(4) as i32;
            }
        } else if arg == b"-p" {
            i += 1;
            if let Some(p) = unsafe { get_arg(argv, i) } {
                pid = sys::parse_i64(p).unwrap_or(0) as i32;
            }
        }
        i += 1;
    }

    if let Some(c) = class {
        // Set I/O priority
        let ioprio = (c << 13) | data;
        let ret = unsafe {
            libc::syscall(
                libc::SYS_ioprio_set,
                1i32, // IOPRIO_WHO_PROCESS
                pid,
                ioprio,
            )
        };

        if ret < 0 {
            io::write_str(2, b"ionice: failed to set I/O priority\n");
            return 1;
        }
    } else {
        // Get I/O priority
        let ret = unsafe {
            libc::syscall(
                libc::SYS_ioprio_get,
                1i32, // IOPRIO_WHO_PROCESS
                pid,
            )
        };

        if ret < 0 {
            io::write_str(2, b"ionice: failed to get I/O priority\n");
            return 1;
        }

        let c = (ret >> 13) & 0x3;
        let d = ret & 0x1fff;

        let class_name: &[u8] = match c {
            0 => b"none",
            1 => b"realtime",
            2 => b"best-effort",
            3 => b"idle",
            _ => b"unknown",
        };

        io::write_all(1, class_name);
        io::write_str(1, b": prio ");
        io::write_num(1, d as u64);
        io::write_str(1, b"\n");
    }

    0
}

// ============================================================================
// CHRT - Set/get real-time scheduling attributes
// ============================================================================

/// chrt - manipulate real-time attributes of a process
pub fn chrt(argc: i32, argv: *const *const u8) -> i32 {
    let mut policy: i32 = libc::SCHED_RR;
    let mut priority: i32 = 0;
    let mut pid: Option<i32> = None;
    let mut get_mode = false;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => { i += 1; continue; }
        };

        if arg == b"-f" || arg == b"--fifo" {
            policy = libc::SCHED_FIFO;
        } else if arg == b"-r" || arg == b"--rr" {
            policy = libc::SCHED_RR;
        } else if arg == b"-o" || arg == b"--other" {
            policy = libc::SCHED_OTHER;
        } else if arg == b"-b" || arg == b"--batch" {
            policy = libc::SCHED_BATCH;
        } else if arg == b"-i" || arg == b"--idle" {
            policy = libc::SCHED_IDLE;
        } else if arg == b"-p" || arg == b"--pid" {
            i += 1;
            if let Some(p) = unsafe { get_arg(argv, i) } {
                pid = sys::parse_i64(p).map(|n| n as i32);
            }
            get_mode = true;
        } else if !arg.starts_with(b"-") {
            if priority == 0 {
                priority = sys::parse_i64(arg).unwrap_or(0) as i32;
            }
        }
        i += 1;
    }

    let target_pid = pid.unwrap_or(0);

    if get_mode && priority == 0 {
        // Get scheduling attributes
        let current_policy = unsafe { libc::sched_getscheduler(target_pid) };
        if current_policy < 0 {
            io::write_str(2, b"chrt: failed to get scheduler\n");
            return 1;
        }

        let mut param: libc::sched_param = unsafe { core::mem::zeroed() };
        if unsafe { libc::sched_getparam(target_pid, &mut param) } < 0 {
            io::write_str(2, b"chrt: failed to get params\n");
            return 1;
        }

        io::write_str(1, b"pid ");
        io::write_num(1, target_pid as u64);
        io::write_str(1, b"'s current scheduling policy: ");

        let policy_name: &[u8] = match current_policy {
            0 => b"SCHED_OTHER",
            1 => b"SCHED_FIFO",
            2 => b"SCHED_RR",
            3 => b"SCHED_BATCH",
            5 => b"SCHED_IDLE",
            _ => b"unknown",
        };
        io::write_all(1, policy_name);
        io::write_str(1, b"\n");

        io::write_str(1, b"pid ");
        io::write_num(1, target_pid as u64);
        io::write_str(1, b"'s current scheduling priority: ");
        io::write_num(1, param.sched_priority as u64);
        io::write_str(1, b"\n");

        return 0;
    }

    // Set scheduling attributes
    let param = libc::sched_param { sched_priority: priority };
    if unsafe { libc::sched_setscheduler(target_pid, policy, &param) } < 0 {
        io::write_str(2, b"chrt: failed to set scheduler\n");
        return 1;
    }

    0
}

/// acpi - show ACPI power/battery status
pub fn acpi(argc: i32, argv: *const *const u8) -> i32 {
    let mut show_battery = true;
    let mut show_ac = false;
    let mut show_thermal = false;
    let mut show_all = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'b') { show_battery = true; }
            if has_opt(arg, b'a') { show_ac = true; }
            if has_opt(arg, b't') { show_thermal = true; }
            if has_opt(arg, b'V') { show_all = true; }
        }
    }

    if show_all {
        show_battery = true;
        show_ac = true;
        show_thermal = true;
    }

    // Read battery info from /sys/class/power_supply
    if show_battery {
        let mut buf = [0u8; 256];
        let fd = io::open(b"/sys/class/power_supply/BAT0/capacity", libc::O_RDONLY, 0);
        if fd >= 0 {
            let n = io::read(fd, &mut buf);
            io::close(fd);
            if n > 0 {
                io::write_str(1, b"Battery 0: ");
                // Read status
                let status_fd = io::open(b"/sys/class/power_supply/BAT0/status", libc::O_RDONLY, 0);
                if status_fd >= 0 {
                    let mut status_buf = [0u8; 64];
                    let sn = io::read(status_fd, &mut status_buf);
                    io::close(status_fd);
                    if sn > 0 {
                        io::write_all(1, &status_buf[..sn as usize - 1]);
                        io::write_str(1, b", ");
                    }
                }
                io::write_all(1, &buf[..n as usize - 1]);
                io::write_str(1, b"%\n");
            }
        } else {
            io::write_str(1, b"No battery information available\n");
        }
    }

    if show_ac {
        let fd = io::open(b"/sys/class/power_supply/AC/online", libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut buf = [0u8; 16];
            let n = io::read(fd, &mut buf);
            io::close(fd);
            if n > 0 {
                io::write_str(1, b"Adapter 0: ");
                if buf[0] == b'1' {
                    io::write_str(1, b"on-line\n");
                } else {
                    io::write_str(1, b"off-line\n");
                }
            }
        }
    }

    if show_thermal {
        let fd = io::open(b"/sys/class/thermal/thermal_zone0/temp", libc::O_RDONLY, 0);
        if fd >= 0 {
            let mut buf = [0u8; 32];
            let n = io::read(fd, &mut buf);
            io::close(fd);
            if n > 0 {
                let temp = sys::parse_u64(&buf[..n as usize - 1]).unwrap_or(0);
                io::write_str(1, b"Thermal 0: ");
                io::write_num(1, temp / 1000);
                io::write_str(1, b".");
                io::write_num(1, (temp % 1000) / 100);
                io::write_str(1, b" degrees C\n");
            }
        }
    }

    0
}

/// cal - display a calendar
pub fn cal(argc: i32, argv: *const *const u8) -> i32 {
    // Get current time
    let now = unsafe { libc::time(core::ptr::null_mut()) };
    let tm = unsafe { libc::localtime(&now) };
    if tm.is_null() {
        return 1;
    }

    let tm = unsafe { &*tm };
    let mut month = (tm.tm_mon + 1) as u32;
    let mut year = (tm.tm_year + 1900) as u32;

    // Parse arguments
    let mut arg_idx = 1;
    while arg_idx < argc {
        if let Some(arg) = unsafe { get_arg(argv, arg_idx) } {
            if let Some(n) = sys::parse_u64(arg) {
                if n <= 12 && arg_idx + 1 < argc {
                    month = n as u32;
                } else {
                    year = n as u32;
                }
            }
        }
        arg_idx += 1;
    }

    // Calculate first day of month (0=Sunday)
    let days_in_month = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let max_days = if month == 2 && is_leap {
        29
    } else {
        days_in_month[month as usize]
    };

    // Zeller's congruence for first day
    let m = if month < 3 { month + 12 } else { month };
    let y = if month < 3 { year - 1 } else { year };
    let k = y % 100;
    let j = y / 100;
    let first_day = (1 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    let first_day = ((first_day as i32 + 6) % 7) as u32; // Convert to 0=Sunday

    // Print header
    let month_names: [&[u8]; 12] = [
        b"January" as &[u8], b"February", b"March", b"April", b"May", b"June",
        b"July", b"August", b"September", b"October", b"November", b"December"
    ];

    io::write_str(1, b"     ");
    io::write_all(1, month_names[(month - 1) as usize]);
    io::write_str(1, b" ");
    io::write_num(1, year as u64);
    io::write_str(1, b"\n");
    io::write_str(1, b"Su Mo Tu We Th Fr Sa\n");

    // Print leading spaces
    for _ in 0..first_day {
        io::write_str(1, b"   ");
    }

    // Print days
    let mut day_of_week = first_day;
    for day in 1..=max_days {
        if day < 10 {
            io::write_str(1, b" ");
        }
        io::write_num(1, day as u64);
        day_of_week += 1;
        if day_of_week == 7 {
            io::write_str(1, b"\n");
            day_of_week = 0;
        } else {
            io::write_str(1, b" ");
        }
    }
    if day_of_week != 0 {
        io::write_str(1, b"\n");
    }

    0
}

/// top - display Linux processes
pub fn top(argc: i32, argv: *const *const u8) -> i32 {
    let mut iterations = -1i32;
    let mut delay = 3;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'n') && i + 1 < argc {
                if let Some(n_arg) = unsafe { get_arg(argv, i + 1) } {
                    iterations = sys::parse_u64(n_arg).unwrap_or(1) as i32;
                }
            }
            if has_opt(arg, b'd') && i + 1 < argc {
                if let Some(d_arg) = unsafe { get_arg(argv, i + 1) } {
                    delay = sys::parse_u64(d_arg).unwrap_or(3) as u32;
                }
            }
            if has_opt(arg, b'b') {
                // Batch mode - just run once
                iterations = 1;
            }
        }
    }

    let mut count = 0;
    loop {
        if iterations > 0 && count >= iterations {
            break;
        }

        // Clear screen (in non-batch mode)
        if iterations != 1 {
            io::write_str(1, b"\x1b[H\x1b[2J");
        }

        // Read /proc/uptime
        let mut uptime_buf = [0u8; 64];
        let uptime_fd = io::open(b"/proc/uptime", libc::O_RDONLY, 0);
        if uptime_fd >= 0 {
            let n = io::read(uptime_fd, &mut uptime_buf);
            io::close(uptime_fd);
            if n > 0 {
                io::write_str(1, b"top - uptime: ");
                // Parse uptime seconds
                let mut i = 0;
                while i < n as usize && uptime_buf[i] != b' ' {
                    i += 1;
                }
                io::write_all(1, &uptime_buf[..i]);
                io::write_str(1, b"s\n");
            }
        }

        // Read /proc/meminfo
        let mut mem_buf = [0u8; 1024];
        let mem_fd = io::open(b"/proc/meminfo", libc::O_RDONLY, 0);
        if mem_fd >= 0 {
            let n = io::read(mem_fd, &mut mem_buf);
            io::close(mem_fd);
            if n > 0 {
                // Parse MemTotal and MemFree
                let content = &mem_buf[..n as usize];
                io::write_str(1, b"Mem: ");
                // Simple display - just show first few lines
                let mut line_count = 0;
                for c in content {
                    if *c == b'\n' {
                        line_count += 1;
                        if line_count >= 3 {
                            break;
                        }
                    }
                }
                io::write_str(1, b"(see /proc/meminfo)\n");
            }
        }

        io::write_str(1, b"\n  PID USER      PR  NI    VIRT    RES  COMMAND\n");

        // Read /proc entries
        let proc_fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
        if proc_fd < 0 {
            return 1;
        }

        let mut entry_count = 0;
        loop {
            let mut dirent: libc::dirent = unsafe { core::mem::zeroed() };
            let n = unsafe { libc::syscall(libc::SYS_getdents64, proc_fd, &mut dirent as *mut _, core::mem::size_of::<libc::dirent>()) };
            if n <= 0 {
                break;
            }

            let name_ptr = dirent.d_name.as_ptr() as *const u8;
            let name = unsafe { io::cstr_to_slice(name_ptr) };

            // Check if numeric (PID directory)
            if name.len() > 0 && name[0] >= b'0' && name[0] <= b'9' {
                // Read /proc/[pid]/stat
                let mut stat_path = [0u8; 64];
                let mut len = 0;
                for c in b"/proc/" {
                    stat_path[len] = *c;
                    len += 1;
                }
                for c in name {
                    stat_path[len] = *c;
                    len += 1;
                }
                for c in b"/stat" {
                    stat_path[len] = *c;
                    len += 1;
                }

                let stat_fd = io::open(&stat_path[..len], libc::O_RDONLY, 0);
                if stat_fd >= 0 {
                    let mut stat_buf = [0u8; 512];
                    let sn = io::read(stat_fd, &mut stat_buf);
                    io::close(stat_fd);

                    if sn > 0 {
                        // Format: pid (comm) state ...
                        io::write_str(1, b"  ");
                        io::write_all(1, name);
                        // Pad to 5 chars
                        for _ in name.len()..5 {
                            io::write_str(1, b" ");
                        }
                        io::write_str(1, b" ");

                        // Extract command name
                        let content = &stat_buf[..sn as usize];
                        if let Some(start) = content.iter().position(|&c| c == b'(') {
                            if let Some(end) = content.iter().position(|&c| c == b')') {
                                let cmd = &content[start + 1..end];
                                io::write_str(1, b"root      20   0       0      0  ");
                                io::write_all(1, cmd);
                                io::write_str(1, b"\n");
                            }
                        }

                        entry_count += 1;
                        if entry_count >= 20 {
                            break;
                        }
                    }
                }
            }
        }
        io::close(proc_fd);

        count += 1;
        if iterations == 1 || (iterations > 0 && count >= iterations) {
            break;
        }

        unsafe { libc::sleep(delay) };
    }

    0
}

/// vmstat - report virtual memory statistics
pub fn vmstat(argc: i32, argv: *const *const u8) -> i32 {
    let mut delay = 1u32;
    let mut count = 1i32;

    // Parse args
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if let Some(n) = sys::parse_u64(arg) {
                if i == 1 {
                    delay = n as u32;
                } else {
                    count = n as i32;
                }
            }
        }
    }

    io::write_str(1, b"procs -----------memory---------- ---swap-- -----io---- -system-- ------cpu-----\n");
    io::write_str(1, b" r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa st\n");

    for _ in 0..count {
        // Read /proc/meminfo
        let mut buf = [0u8; 2048];
        let fd = io::open(b"/proc/meminfo", libc::O_RDONLY, 0);
        if fd < 0 {
            return 1;
        }
        let n = io::read(fd, &mut buf);
        io::close(fd);

        if n > 0 {
            // Parse values (simplified)
            io::write_str(1, b" 0  0      0 ");
            // Just show placeholder values
            io::write_str(1, b"     0      0      0    0    0     0     0    0    0  0  0 100  0  0\n");
        }

        if count > 1 {
            unsafe { libc::sleep(delay) };
        }
    }

    0
}

/// watch - execute a program periodically
pub fn watch(argc: i32, argv: *const *const u8) -> i32 {
    let mut interval = 2u32;
    let mut cmd_start = 1;

    // Parse options
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'n') && i + 1 < argc {
                if let Some(n_arg) = unsafe { get_arg(argv, i + 1) } {
                    interval = sys::parse_u64(n_arg).unwrap_or(2) as u32;
                    cmd_start = i + 2;
                }
            } else if arg[0] != b'-' {
                cmd_start = i;
                break;
            }
        }
    }

    if cmd_start >= argc {
        io::write_str(2, b"watch: no command specified\n");
        return 1;
    }

    loop {
        // Clear screen
        io::write_str(1, b"\x1b[H\x1b[2J");
        io::write_str(1, b"Every ");
        io::write_num(1, interval as u64);
        io::write_str(1, b"s: ");

        // Print command
        for i in cmd_start..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                io::write_all(1, arg);
                io::write_str(1, b" ");
            }
        }
        io::write_str(1, b"\n\n");

        // Execute command
        let pid = unsafe { libc::fork() };
        if pid == 0 {
            // Child - exec command
            #[cfg(feature = "alloc")]
            {
                use alloc::vec::Vec;
                use alloc::ffi::CString;

                let mut args: Vec<CString> = Vec::new();
                for i in cmd_start..argc {
                    if let Some(arg) = unsafe { get_arg(argv, i) } {
                        let mut v = Vec::with_capacity(arg.len() + 1);
                        v.extend_from_slice(arg);
                        v.push(0);
                        if let Ok(cs) = CString::from_vec_with_nul(v) {
                            args.push(cs);
                        }
                    }
                }

                let ptrs: Vec<*const i8> = args.iter().map(|s| s.as_ptr()).chain(core::iter::once(core::ptr::null())).collect();
                unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
            }
            unsafe { libc::_exit(1) };
        } else if pid > 0 {
            // Parent - wait
            let mut status = 0;
            unsafe { libc::waitpid(pid, &mut status, 0) };
        }

        unsafe { libc::sleep(interval) };
    }
}

/// hwclock - query/set hardware clock
pub fn hwclock(argc: i32, argv: *const *const u8) -> i32 {
    let mut show = true;
    let mut set_sys = false;
    let mut set_hw = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'r') { show = true; }
            if has_opt(arg, b's') { set_sys = true; show = false; }
            if has_opt(arg, b'w') { set_hw = true; show = false; }
        }
    }

    // Open RTC device
    let rtc_fd = io::open(b"/dev/rtc0", libc::O_RDONLY, 0);
    if rtc_fd < 0 {
        let rtc_fd = io::open(b"/dev/rtc", libc::O_RDONLY, 0);
        if rtc_fd < 0 {
            io::write_str(2, b"hwclock: cannot open RTC\n");
            return 1;
        }
    }

    if show {
        // Read RTC time via ioctl (RTC_RD_TIME = 0x80247009)
        let mut rtc_time: [i32; 9] = [0; 9];
        let ret = unsafe { libc::ioctl(rtc_fd, 0x80247009, rtc_time.as_mut_ptr()) };
        if ret < 0 {
            io::write_str(2, b"hwclock: cannot read RTC\n");
            io::close(rtc_fd);
            return 1;
        }

        // Format time
        io::write_num(1, (rtc_time[5] + 1900) as u64);
        io::write_str(1, b"-");
        if rtc_time[4] + 1 < 10 { io::write_str(1, b"0"); }
        io::write_num(1, (rtc_time[4] + 1) as u64);
        io::write_str(1, b"-");
        if rtc_time[3] < 10 { io::write_str(1, b"0"); }
        io::write_num(1, rtc_time[3] as u64);
        io::write_str(1, b" ");
        if rtc_time[2] < 10 { io::write_str(1, b"0"); }
        io::write_num(1, rtc_time[2] as u64);
        io::write_str(1, b":");
        if rtc_time[1] < 10 { io::write_str(1, b"0"); }
        io::write_num(1, rtc_time[1] as u64);
        io::write_str(1, b":");
        if rtc_time[0] < 10 { io::write_str(1, b"0"); }
        io::write_num(1, rtc_time[0] as u64);
        io::write_str(1, b"\n");
    }

    if set_sys {
        io::write_str(1, b"hwclock: -s not fully implemented\n");
    }

    if set_hw {
        io::write_str(1, b"hwclock: -w not fully implemented\n");
    }

    io::close(rtc_fd);
    0
}

/// fallocate - preallocate space to a file
pub fn fallocate(argc: i32, argv: *const *const u8) -> i32 {
    let mut length: i64 = 0;
    let mut file_arg = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'l') && i + 1 < argc {
                if let Some(l_arg) = unsafe { get_arg(argv, i + 1) } {
                    length = sys::parse_size(l_arg).unwrap_or(0) as i64;
                }
            } else if arg[0] != b'-' && file_arg == 0 {
                file_arg = i;
            }
        }
    }

    if file_arg == 0 || length == 0 {
        io::write_str(2, b"usage: fallocate -l SIZE FILE\n");
        return 1;
    }

    let file = unsafe { get_arg(argv, file_arg).unwrap() };
    let fd = io::open(file, libc::O_WRONLY | libc::O_CREAT, 0o644);
    if fd < 0 {
        sys::perror(file);
        return 1;
    }

    // Use posix_fallocate or fallback to ftruncate
    let ret = unsafe { libc::ftruncate(fd, length) };
    io::close(fd);

    if ret < 0 {
        sys::perror(file);
        return 1;
    }

    0
}

/// shuf - shuffle lines
pub fn shuf(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut lines: Vec<Vec<u8>> = Vec::new();
        let mut head_count: Option<usize> = None;

        // Parse args
        let mut file_arg = None;
        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if has_opt(arg, b'n') && i + 1 < argc {
                    if let Some(n_arg) = unsafe { get_arg(argv, i + 1) } {
                        head_count = Some(sys::parse_u64(n_arg).unwrap_or(0) as usize);
                    }
                } else if arg[0] != b'-' {
                    file_arg = Some(arg);
                }
            }
        }

        // Read input
        let fd = if let Some(f) = file_arg {
            io::open(f, libc::O_RDONLY, 0)
        } else {
            0
        };

        if fd < 0 {
            return 1;
        }

        let mut buf = [0u8; 8192];
        let mut current_line: Vec<u8> = Vec::new();

        loop {
            let n = io::read(fd, &mut buf);
            if n <= 0 {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                break;
            }

            for i in 0..n as usize {
                if buf[i] == b'\n' {
                    lines.push(core::mem::take(&mut current_line));
                } else {
                    current_line.push(buf[i]);
                }
            }
        }

        if fd > 0 {
            io::close(fd);
        }

        // Fisher-Yates shuffle
        let seed = unsafe { libc::time(core::ptr::null_mut()) } as u64;
        let mut rng = seed;
        for i in (1..lines.len()).rev() {
            // Simple LCG
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (rng as usize) % (i + 1);
            lines.swap(i, j);
        }

        // Output
        let count = head_count.unwrap_or(lines.len());
        for (i, line) in lines.iter().enumerate() {
            if i >= count {
                break;
            }
            io::write_all(1, line);
            io::write_str(1, b"\n");
        }

        return 0;
    }

    #[cfg(not(feature = "alloc"))]
    {
        io::write_str(2, b"shuf: requires alloc feature\n");
        1
    }
}

/// mkswap - set up a Linux swap area
pub fn mkswap(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"usage: mkswap DEVICE\n");
        return 1;
    }

    let device = unsafe { get_arg(argv, 1).unwrap() };
    let fd = io::open(device, libc::O_RDWR, 0);
    if fd < 0 {
        sys::perror(device);
        return 1;
    }

    // Get device size
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    if io::fstat(fd, &mut st) != 0 {
        io::close(fd);
        return 1;
    }

    let size = st.st_size as u64;
    let pages = size / 4096;

    // Write swap header (simplified)
    // Swap signature at offset 4086
    let mut header = [0u8; 4096];
    header[4086..4096].copy_from_slice(b"SWAPSPACE2");

    // Write version and last page
    header[1024] = 1; // version
    header[1028..1032].copy_from_slice(&((pages - 1) as u32).to_le_bytes());

    if io::write_all_fd(fd, &header) != 4096 {
        io::write_str(2, b"mkswap: write error\n");
        io::close(fd);
        return 1;
    }

    io::close(fd);

    io::write_str(1, b"Setting up swapspace version 1, size = ");
    io::write_num(1, (pages - 1) * 4096);
    io::write_str(1, b" bytes\n");

    0
}

/// nologin - politely refuse a login
pub fn nologin(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"This account is currently not available.\n");
    1
}

/// nsenter - run program in different namespaces
pub fn nsenter(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"usage: nsenter -t PID COMMAND...\n");
        return 1;
    }

    let mut target_pid: Option<i32> = None;
    let mut cmd_start = 1;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b't') && i + 1 < argc {
                if let Some(pid_arg) = unsafe { get_arg(argv, i + 1) } {
                    target_pid = Some(sys::parse_u64(pid_arg).unwrap_or(0) as i32);
                    cmd_start = i + 2;
                }
            }
        }
    }

    let pid = match target_pid {
        Some(p) => p,
        None => {
            io::write_str(2, b"nsenter: -t PID required\n");
            return 1;
        }
    };

    // Open namespace files
    let ns_types: [&[u8]; 6] = [b"mnt", b"uts", b"ipc", b"net", b"pid", b"user"];
    for ns in &ns_types {
        let mut path = [0u8; 64];
        let mut len = 0;
        for c in b"/proc/" {
            path[len] = *c;
            len += 1;
        }
        // Write PID
        let pid_str = sys::format_num(pid as u64);
        for c in pid_str {
            path[len] = *c;
            len += 1;
        }
        for c in b"/ns/" {
            path[len] = *c;
            len += 1;
        }
        for c in *ns {
            path[len] = *c;
            len += 1;
        }

        let ns_fd = io::open(&path[..len], libc::O_RDONLY, 0);
        if ns_fd >= 0 {
            // setns syscall
            unsafe { libc::syscall(libc::SYS_setns, ns_fd, 0) };
            io::close(ns_fd);
        }
    }

    // Execute command
    if cmd_start < argc {
        #[cfg(feature = "alloc")]
        {
            use alloc::vec::Vec;
            use alloc::ffi::CString;

            let mut args: Vec<CString> = Vec::new();
            for i in cmd_start..argc {
                if let Some(arg) = unsafe { get_arg(argv, i) } {
                    let mut v = Vec::with_capacity(arg.len() + 1);
                    v.extend_from_slice(arg);
                    v.push(0);
                    if let Ok(cs) = CString::from_vec_with_nul(v) {
                        args.push(cs);
                    }
                }
            }

            let ptrs: Vec<*const i8> = args.iter().map(|s| s.as_ptr()).chain(core::iter::once(core::ptr::null())).collect();
            unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
        }
    }

    1
}

/// unshare - run program in new namespaces
pub fn unshare(argc: i32, argv: *const *const u8) -> i32 {
    let mut flags = 0i32;
    let mut cmd_start = 1;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b'm') { flags |= libc::CLONE_NEWNS; }
                if has_opt(arg, b'u') { flags |= libc::CLONE_NEWUTS; }
                if has_opt(arg, b'i') { flags |= libc::CLONE_NEWIPC; }
                if has_opt(arg, b'n') { flags |= libc::CLONE_NEWNET; }
                if has_opt(arg, b'p') { flags |= libc::CLONE_NEWPID; }
                if has_opt(arg, b'U') { flags |= libc::CLONE_NEWUSER; }
            } else {
                cmd_start = i;
                break;
            }
        }
    }

    // Call unshare
    if unsafe { libc::unshare(flags) } < 0 {
        sys::perror(b"unshare");
        return 1;
    }

    // Execute command
    if cmd_start < argc {
        #[cfg(feature = "alloc")]
        {
            use alloc::vec::Vec;
            use alloc::ffi::CString;

            let mut args: Vec<CString> = Vec::new();
            for i in cmd_start..argc {
                if let Some(arg) = unsafe { get_arg(argv, i) } {
                    let mut v = Vec::with_capacity(arg.len() + 1);
                    v.extend_from_slice(arg);
                    v.push(0);
                    if let Ok(cs) = CString::from_vec_with_nul(v) {
                        args.push(cs);
                    }
                }
            }

            let ptrs: Vec<*const i8> = args.iter().map(|s| s.as_ptr()).chain(core::iter::once(core::ptr::null())).collect();
            unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
        }
    }

    0
}

/// pmap - report memory map of a process
pub fn pmap(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"usage: pmap PID\n");
        return 1;
    }

    let pid = unsafe { get_arg(argv, 1).unwrap() };

    // Read /proc/[pid]/maps
    let mut path = [0u8; 64];
    let mut len = 0;
    for c in b"/proc/" {
        path[len] = *c;
        len += 1;
    }
    for c in pid {
        path[len] = *c;
        len += 1;
    }
    for c in b"/maps" {
        path[len] = *c;
        len += 1;
    }

    let fd = io::open(&path[..len], libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(pid);
        return 1;
    }

    io::write_all(1, pid);
    io::write_str(1, b":   ");

    // Get process name
    let mut cmdline_path = [0u8; 64];
    len = 0;
    for c in b"/proc/" {
        cmdline_path[len] = *c;
        len += 1;
    }
    for c in pid {
        cmdline_path[len] = *c;
        len += 1;
    }
    for c in b"/cmdline" {
        cmdline_path[len] = *c;
        len += 1;
    }

    let cmd_fd = io::open(&cmdline_path[..len], libc::O_RDONLY, 0);
    if cmd_fd >= 0 {
        let mut cmd_buf = [0u8; 256];
        let n = io::read(cmd_fd, &mut cmd_buf);
        io::close(cmd_fd);
        if n > 0 {
            // Find first null
            let end = cmd_buf[..n as usize].iter().position(|&c| c == 0).unwrap_or(n as usize);
            io::write_all(1, &cmd_buf[..end]);
        }
    }
    io::write_str(1, b"\n");

    // Read and display maps
    let mut buf = [0u8; 8192];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 {
            break;
        }
        io::write_all(1, &buf[..n as usize]);
    }

    io::close(fd);
    0
}

/// su - change user ID or become superuser
pub fn su(argc: i32, argv: *const *const u8) -> i32 {
    let mut target_user = b"root".as_slice();
    let mut login_shell = false;
    let mut cmd_start = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-" || has_opt(arg, b'l') {
                login_shell = true;
            } else if has_opt(arg, b'c') && i + 1 < argc {
                cmd_start = i + 1;
            } else if arg[0] != b'-' {
                target_user = arg;
            }
        }
    }

    // For now, just show that we'd switch (real impl needs PAM/passwd)
    io::write_str(1, b"su: would switch to user: ");
    io::write_all(1, target_user);
    io::write_str(1, b"\n");

    if login_shell {
        io::write_str(1, b"su: with login shell\n");
    }

    // In a real implementation, we'd:
    // 1. Check /etc/passwd for target user
    // 2. Verify password (or PAM)
    // 3. setuid/setgid
    // 4. exec shell

    0
}

/// login - begin session on the system
pub fn login(argc: i32, argv: *const *const u8) -> i32 {
    let mut username: Option<&[u8]> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] != b'-' {
                username = Some(arg);
                break;
            }
        }
    }

    // Prompt for username if not provided
    if username.is_none() {
        io::write_str(1, b"login: ");
        // Would read username here
    }

    io::write_str(1, b"Password: ");
    // Would read password here

    io::write_str(1, b"\nlogin: stub implementation\n");
    0
}

/// eject - eject removable media
pub fn eject(argc: i32, argv: *const *const u8) -> i32 {
    let device = if argc > 1 {
        unsafe { get_arg(argv, 1).unwrap_or(b"/dev/cdrom") }
    } else {
        b"/dev/cdrom"
    };

    let fd = io::open(device, libc::O_RDONLY | libc::O_NONBLOCK, 0);
    if fd < 0 {
        sys::perror(device);
        return 1;
    }

    // CDROMEJECT = 0x5309
    let ret = unsafe { libc::ioctl(fd, 0x5309, 0) };
    io::close(fd);

    if ret < 0 {
        sys::perror(device);
        return 1;
    }

    0
}

/// blockdev - call block device ioctls
pub fn blockdev(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"usage: blockdev OPTION DEVICE\n");
        return 1;
    }

    let option = unsafe { get_arg(argv, 1).unwrap() };
    let device = unsafe { get_arg(argv, argc - 1).unwrap() };

    let fd = io::open(device, libc::O_RDONLY, 0);
    if fd < 0 {
        sys::perror(device);
        return 1;
    }

    let ret = if option == b"--getsize64" || option == b"--getsz" {
        // BLKGETSIZE64 = 0x80081272
        let mut size: u64 = 0;
        let r = unsafe { libc::ioctl(fd, 0x80081272, &mut size) };
        if r >= 0 {
            if option == b"--getsz" {
                io::write_num(1, size / 512);
            } else {
                io::write_num(1, size);
            }
            io::write_str(1, b"\n");
        }
        r
    } else if option == b"--getss" {
        // BLKSSZGET = 0x1268
        let mut ss: i32 = 0;
        let r = unsafe { libc::ioctl(fd, 0x1268, &mut ss) };
        if r >= 0 {
            io::write_num(1, ss as u64);
            io::write_str(1, b"\n");
        }
        r
    } else if option == b"--getro" {
        // BLKROGET = 0x125e
        let mut ro: i32 = 0;
        let r = unsafe { libc::ioctl(fd, 0x125e, &mut ro) };
        if r >= 0 {
            io::write_num(1, ro as u64);
            io::write_str(1, b"\n");
        }
        r
    } else {
        io::write_str(2, b"blockdev: unknown option\n");
        -1
    };

    io::close(fd);
    if ret < 0 { 1 } else { 0 }
}

/// killall5 - send signal to all processes
pub fn killall5(argc: i32, argv: *const *const u8) -> i32 {
    let mut signal = libc::SIGTERM;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' && arg.len() > 1 {
                if let Some(sig) = sys::parse_u64(&arg[1..]) {
                    signal = sig as i32;
                }
            }
        }
    }

    // Get our PID and session ID to exclude
    let my_pid = unsafe { libc::getpid() };
    let my_sid = unsafe { libc::getsid(0) };

    // Iterate /proc
    let proc_fd = io::open(b"/proc", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if proc_fd < 0 {
        return 1;
    }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, proc_fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 {
            break;
        }

        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if let Some(pid) = sys::parse_u64(name) {
                let pid = pid as i32;
                if pid != my_pid && pid != 1 {
                    let sid = unsafe { libc::getsid(pid) };
                    if sid != my_sid {
                        unsafe { libc::kill(pid, signal) };
                    }
                }
            }

            offset += dirent.d_reclen as usize;
        }
    }

    io::close(proc_fd);
    0
}
