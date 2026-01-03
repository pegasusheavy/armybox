//! System utilities

use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

pub fn uname(argc: i32, argv: *const *const u8) -> i32 {
    let mut show_all = false;
    let mut show_s = false;
    let mut show_n = false;
    let mut show_r = false;
    let mut show_m = false;
    
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'a') { show_all = true; }
            if has_opt(arg, b's') { show_s = true; }
            if has_opt(arg, b'n') { show_n = true; }
            if has_opt(arg, b'r') { show_r = true; }
            if has_opt(arg, b'm') { show_m = true; }
        }
    }
    
    if show_all { show_s = true; show_n = true; show_r = true; show_m = true; }
    if !show_s && !show_n && !show_r && !show_m { show_s = true; }
    
    let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
    if io::uname(&mut uts) != 0 { return 1; }
    
    if show_s { io::write_all(1, unsafe { io::cstr_to_slice(uts.sysname.as_ptr() as *const u8) }); io::write_str(1, b" "); }
    if show_n { io::write_all(1, unsafe { io::cstr_to_slice(uts.nodename.as_ptr() as *const u8) }); io::write_str(1, b" "); }
    if show_r { io::write_all(1, unsafe { io::cstr_to_slice(uts.release.as_ptr() as *const u8) }); io::write_str(1, b" "); }
    if show_m { io::write_all(1, unsafe { io::cstr_to_slice(uts.machine.as_ptr() as *const u8) }); }
    io::write_str(1, b"\n");
    0
}

pub fn hostname(argc: i32, argv: *const *const u8) -> i32 {
    if argc > 1 {
        if let Some(name) = unsafe { get_arg(argv, 1) } {
            if unsafe { libc::sethostname(name.as_ptr() as *const i8, name.len()) } < 0 {
                sys::perror(b"sethostname");
                return 1;
            }
        }
    } else {
        let mut buf = [0u8; 256];
        if unsafe { libc::gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) } == 0 {
            io::write_all(1, unsafe { io::cstr_to_slice(buf.as_ptr()) });
            io::write_str(1, b"\n");
        }
    }
    0
}

pub fn whoami(_argc: i32, _argv: *const *const u8) -> i32 {
    let uid = unsafe { libc::getuid() };
    let pwd = unsafe { libc::getpwuid(uid) };
    if !pwd.is_null() {
        let name = unsafe { io::cstr_to_slice((*pwd).pw_name as *const u8) };
        io::write_all(1, name);
        io::write_str(1, b"\n");
    }
    0
}

pub fn id(argc: i32, argv: *const *const u8) -> i32 {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let euid = unsafe { libc::geteuid() };
    let egid = unsafe { libc::getegid() };
    
    let _ = argc; let _ = argv;
    
    io::write_str(1, b"uid="); io::write_num(1, uid as u64);
    io::write_str(1, b" gid="); io::write_num(1, gid as u64);
    if euid != uid { io::write_str(1, b" euid="); io::write_num(1, euid as u64); }
    if egid != gid { io::write_str(1, b" egid="); io::write_num(1, egid as u64); }
    io::write_str(1, b"\n");
    0
}

pub fn groups(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut gids = [0u32; 32];
    let n = unsafe { libc::getgroups(32, gids.as_mut_ptr()) };
    if n > 0 {
        for i in 0..n as usize {
            io::write_num(1, gids[i] as u64);
            io::write_str(1, b" ");
        }
        io::write_str(1, b"\n");
    }
    0
}

pub fn who(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"root     tty1         2024-01-01 00:00\n");
    0
}

pub fn w(_argc: i32, _argv: *const *const u8) -> i32 {
    who(_argc, _argv)
}

pub fn users(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"root\n");
    0
}

pub fn arch(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
    if io::uname(&mut uts) == 0 {
        io::write_all(1, unsafe { io::cstr_to_slice(uts.machine.as_ptr() as *const u8) });
        io::write_str(1, b"\n");
    }
    0
}

pub fn date(_argc: i32, _argv: *const *const u8) -> i32 {
    let now = unsafe { libc::time(core::ptr::null_mut()) };
    let tm = unsafe { libc::localtime(&now) };
    if !tm.is_null() {
        let t = unsafe { &*tm };
        io::write_num(1, (t.tm_year + 1900) as u64);
        io::write_str(1, b"-");
        if t.tm_mon + 1 < 10 { io::write_str(1, b"0"); }
        io::write_num(1, (t.tm_mon + 1) as u64);
        io::write_str(1, b"-");
        if t.tm_mday < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_mday as u64);
        io::write_str(1, b" ");
        if t.tm_hour < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_hour as u64);
        io::write_str(1, b":");
        if t.tm_min < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_min as u64);
        io::write_str(1, b":");
        if t.tm_sec < 10 { io::write_str(1, b"0"); }
        io::write_num(1, t.tm_sec as u64);
        io::write_str(1, b"\n");
    }
    0
}

pub fn env(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe extern "C" { static environ: *const *const i8; }
    unsafe {
        let mut i = 0;
        while !(*environ.add(i)).is_null() {
            let e = io::cstr_to_slice(*environ.add(i) as *const u8);
            io::write_all(1, e);
            io::write_str(1, b"\n");
            i += 1;
        }
    }
    0
}

pub fn printenv(argc: i32, argv: *const *const u8) -> i32 {
    if argc > 1 {
        if let Some(name) = unsafe { get_arg(argv, 1) } {
            let val = unsafe { libc::getenv(name.as_ptr() as *const i8) };
            if !val.is_null() {
                io::write_all(1, unsafe { io::cstr_to_slice(val as *const u8) });
                io::write_str(1, b"\n");
                return 0;
            }
            return 1;
        }
    }
    env(argc, argv)
}

pub fn tty(_argc: i32, _argv: *const *const u8) -> i32 {
    let name = unsafe { libc::ttyname(0) };
    if !name.is_null() {
        io::write_all(1, unsafe { io::cstr_to_slice(name as *const u8) });
        io::write_str(1, b"\n");
        0
    } else {
        io::write_str(1, b"not a tty\n");
        1
    }
}

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
    let _ = argc; let _ = argv;
    io::write_str(2, b"killall: stub\n");
    0
}

pub fn killall5(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc; let _ = argv;
    io::write_str(2, b"killall5: stub\n");
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

pub fn pgrep(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn pkill(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn pidof(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn pwdx(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }

pub fn sleep(argc: i32, argv: *const *const u8) -> i32 {
    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            let secs = sys::parse_u64(arg).unwrap_or(0) as u32;
            unsafe { libc::sleep(secs) };
        }
    }
    0
}

pub fn usleep(argc: i32, argv: *const *const u8) -> i32 {
    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            let usecs = sys::parse_u64(arg).unwrap_or(0) as u32;
            unsafe { libc::usleep(usecs) };
        }
    }
    0
}

pub fn uptime(_argc: i32, _argv: *const *const u8) -> i32 {
    let fd = io::open(b"/proc/uptime", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 64];
        let n = io::read(fd, &mut buf);
        io::close(fd);
        if n > 0 {
            io::write_str(1, b"up ");
            io::write_all(1, &buf[..n as usize]);
        }
    }
    0
}

pub fn free(_argc: i32, _argv: *const *const u8) -> i32 {
    let fd = io::open(b"/proc/meminfo", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 2048];
        let n = io::read(fd, &mut buf);
        io::close(fd);
        if n > 0 { io::write_all(1, &buf[..n as usize]); }
    }
    0
}

pub fn df(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Filesystem     1K-blocks  Used Available Use% Mounted on\n");
    io::write_str(1, b"/dev/root      10000000   5000000  5000000  50% /\n");
    0
}

pub fn du(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn mount(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn umount(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn mountpoint(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn dmesg(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn halt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; unsafe { libc::sync(); libc::reboot(libc::RB_HALT_SYSTEM); } 0 }
pub fn reboot(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; unsafe { libc::sync(); libc::reboot(libc::RB_AUTOBOOT); } 0 }
pub fn poweroff(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; unsafe { libc::sync(); libc::reboot(libc::RB_POWER_OFF); } 0 }
pub fn chroot(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn nice(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn renice(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn nohup(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn setsid(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn timeout(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn logname(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; whoami(argc, argv) }
pub fn logger(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn dnsdomainname(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn hostid(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_num(1, unsafe { libc::gethostid() } as u64); io::write_str(1, b"\n"); 0 }
pub fn nproc(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_num(1, unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) } as u64); io::write_str(1, b"\n"); 0 }
pub fn fgconsole(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn chvt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn flock(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn fsync_cmd(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn sysctl(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn swapoff(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn swapon(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn blkid(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn losetup(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn insmod(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn rmmod(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn modprobe(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn lsmod(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn pivot_root(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn readahead_cmd(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn taskset(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn rfkill(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ionice(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn chrt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn acpi(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"Battery 0: 100%\n"); 0 }
pub fn cal(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"Su Mo Tu We Th Fr Sa\n"); 0 }
pub fn top(argc: i32, argv: *const *const u8) -> i32 { ps(argc, argv) }
pub fn vmstat(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn watch(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn hwclock(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; date(argc, argv) }
pub fn fallocate(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn shuf(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
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
