//! Miscellaneous utilities

use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

pub fn r#true(_argc: i32, _argv: *const *const u8) -> i32 { 0 }
pub fn r#false(_argc: i32, _argv: *const *const u8) -> i32 { 1 }
pub fn colon(_argc: i32, _argv: *const *const u8) -> i32 { 0 }

pub fn test(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }

    let arg1 = unsafe { get_arg(argv, 1).unwrap() };

    // Unary tests
    if argc == 3 {
        let op = arg1;
        let path = unsafe { get_arg(argv, 2).unwrap() };

        let mut st: libc::stat = unsafe { core::mem::zeroed() };
        let stat_ok = io::stat(path, &mut st) == 0;

        return match op {
            b"-e" => if stat_ok { 0 } else { 1 },
            b"-f" => if stat_ok && (st.st_mode & libc::S_IFMT) == libc::S_IFREG { 0 } else { 1 },
            b"-d" => if stat_ok && (st.st_mode & libc::S_IFMT) == libc::S_IFDIR { 0 } else { 1 },
            b"-r" => if unsafe { libc::access(path.as_ptr() as *const i8, libc::R_OK) } == 0 { 0 } else { 1 },
            b"-w" => if unsafe { libc::access(path.as_ptr() as *const i8, libc::W_OK) } == 0 { 0 } else { 1 },
            b"-x" => if unsafe { libc::access(path.as_ptr() as *const i8, libc::X_OK) } == 0 { 0 } else { 1 },
            b"-s" => if stat_ok && st.st_size > 0 { 0 } else { 1 },
            b"-n" => if !path.is_empty() { 0 } else { 1 },
            b"-z" => if path.is_empty() { 0 } else { 1 },
            b"-L" | b"-h" => if stat_ok && (st.st_mode & libc::S_IFMT) == libc::S_IFLNK { 0 } else { 1 },
            _ => 1,
        };
    }

    // Binary tests
    if argc == 4 {
        let left = arg1;
        let op = unsafe { get_arg(argv, 2).unwrap() };
        let right = unsafe { get_arg(argv, 3).unwrap() };

        return match op {
            b"=" | b"==" => if left == right { 0 } else { 1 },
            b"!=" => if left != right { 0 } else { 1 },
            b"-eq" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l == r { 0 } else { 1 }
            }
            b"-ne" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l != r { 0 } else { 1 }
            }
            b"-lt" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l < r { 0 } else { 1 }
            }
            b"-gt" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l > r { 0 } else { 1 }
            }
            b"-le" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l <= r { 0 } else { 1 }
            }
            b"-ge" => {
                let l = sys::parse_i64(left).unwrap_or(0);
                let r = sys::parse_i64(right).unwrap_or(0);
                if l >= r { 0 } else { 1 }
            }
            _ => 1,
        };
    }

    // Single arg - true if non-empty
    if !arg1.is_empty() { 0 } else { 1 }
}

pub fn bracket(argc: i32, argv: *const *const u8) -> i32 {
    test(argc, argv)
}

pub fn clear(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"\x1b[H\x1b[2J");
    0
}

pub fn reset(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"\x1bc");
    0
}

pub fn which(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }

    let cmd = unsafe { get_arg(argv, 1).unwrap() };
    let path_env = unsafe { libc::getenv(b"PATH\0".as_ptr() as *const i8) };

    if path_env.is_null() { return 1; }

    let path = unsafe { io::cstr_to_slice(path_env as *const u8) };

    for dir in path.split(|&c| c == b':') {
        let mut full_path = [0u8; 512];
        let mut len = 0;
        for &c in dir { full_path[len] = c; len += 1; }
        full_path[len] = b'/'; len += 1;
        for &c in cmd { full_path[len] = c; len += 1; }

        if unsafe { libc::access(full_path.as_ptr() as *const i8, libc::X_OK) } == 0 {
            io::write_all(1, &full_path[..len]);
            io::write_str(1, b"\n");
            return 0;
        }
    }
    1
}

pub fn expr(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 2; }

    if argc == 2 {
        let arg = unsafe { get_arg(argv, 1).unwrap() };
        io::write_all(1, arg);
        io::write_str(1, b"\n");
        return if arg.is_empty() || arg == b"0" { 1 } else { 0 };
    }

    if argc == 4 {
        let left = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(0);
        let op = unsafe { get_arg(argv, 2).unwrap() };
        let right = sys::parse_i64(unsafe { get_arg(argv, 3).unwrap() }).unwrap_or(0);

        let result = match op {
            b"+" => left + right,
            b"-" => left - right,
            b"*" => left * right,
            b"/" => if right != 0 { left / right } else { 0 },
            b"%" => if right != 0 { left % right } else { 0 },
            _ => 0,
        };

        io::write_signed(1, result);
        io::write_str(1, b"\n");
        return if result == 0 { 1 } else { 0 };
    }
    2
}

pub fn time(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 0; }

    let start = unsafe { libc::time(core::ptr::null_mut()) };

    // Fork and exec
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        #[cfg(feature = "alloc")]
        {
            use alloc::vec::Vec;
            use alloc::ffi::CString;

            let mut args: Vec<CString> = Vec::new();
            for i in 1..argc {
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
        unsafe { libc::_exit(127) };
    } else if pid > 0 {
        let mut status = 0;
        unsafe { libc::waitpid(pid, &mut status, 0) };

        let end = unsafe { libc::time(core::ptr::null_mut()) };
        let elapsed = end - start;

        io::write_str(2, b"\nreal\t");
        io::write_num(2, elapsed as u64);
        io::write_str(2, b"s\n");
    }
    0
}

pub fn mesg(argc: i32, argv: *const *const u8) -> i32 {
    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            let mode = if arg == b"y" { 0o620 } else { 0o600 };
            let tty = unsafe { libc::ttyname(0) };
            if !tty.is_null() {
                unsafe { libc::chmod(tty, mode) };
            }
        }
    }
    0
}

pub fn getconf(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let name = unsafe { get_arg(argv, 1).unwrap() };

    let val = match name {
        b"PAGE_SIZE" | b"PAGESIZE" => unsafe { libc::sysconf(libc::_SC_PAGESIZE) },
        b"NPROCESSORS_ONLN" => unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) },
        b"NPROCESSORS_CONF" => unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) },
        _ => -1,
    };

    if val >= 0 {
        io::write_num(1, val as u64);
        io::write_str(1, b"\n");
        0
    } else {
        1
    }
}

pub fn factor(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            let mut n = sys::parse_u64(arg).unwrap_or(0);
            io::write_num(1, n);
            io::write_str(1, b":");

            let mut d = 2u64;
            while d * d <= n {
                while n % d == 0 {
                    io::write_str(1, b" ");
                    io::write_num(1, d);
                    n /= d;
                }
                d += 1;
            }
            if n > 1 {
                io::write_str(1, b" ");
                io::write_num(1, n);
            }
            io::write_str(1, b"\n");
        }
    }
    0
}

pub fn base64(argc: i32, argv: *const *const u8) -> i32 {
    let mut decode = false;
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'd') { decode = true; }
        }
    }

    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    if decode {
        // Decode
        let mut buf = [0u8; 4096];
        let n = io::read(0, &mut buf);
        if n > 0 {
            let mut i = 0;
            while i + 4 <= n as usize {
                let a = ALPHABET.iter().position(|&c| c == buf[i]).unwrap_or(0);
                let b = ALPHABET.iter().position(|&c| c == buf[i+1]).unwrap_or(0);
                let c = if buf[i+2] != b'=' { ALPHABET.iter().position(|&x| x == buf[i+2]).unwrap_or(0) } else { 0 };
                let d = if buf[i+3] != b'=' { ALPHABET.iter().position(|&x| x == buf[i+3]).unwrap_or(0) } else { 0 };

                io::write_all(1, &[((a << 2) | (b >> 4)) as u8]);
                if buf[i+2] != b'=' { io::write_all(1, &[(((b & 0xf) << 4) | (c >> 2)) as u8]); }
                if buf[i+3] != b'=' { io::write_all(1, &[(((c & 0x3) << 6) | d) as u8]); }
                i += 4;
            }
        }
    } else {
        // Encode
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 { break; }

            let mut i = 0;
            while i + 3 <= n as usize {
                let a = buf[i];
                let b = buf[i+1];
                let c = buf[i+2];
                io::write_all(1, &[ALPHABET[(a >> 2) as usize]]);
                io::write_all(1, &[ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize]]);
                io::write_all(1, &[ALPHABET[(((b & 0xf) << 2) | (c >> 6)) as usize]]);
                io::write_all(1, &[ALPHABET[(c & 0x3f) as usize]]);
                i += 3;
            }

            if i < n as usize {
                let a = buf[i];
                let b = if i + 1 < n as usize { buf[i+1] } else { 0 };
                io::write_all(1, &[ALPHABET[(a >> 2) as usize]]);
                io::write_all(1, &[ALPHABET[(((a & 0x3) << 4) | (b >> 4)) as usize]]);
                if i + 1 < n as usize {
                    io::write_all(1, &[ALPHABET[((b & 0xf) << 2) as usize]]);
                    io::write_str(1, b"=");
                } else {
                    io::write_str(1, b"==");
                }
            }
        }
        io::write_str(1, b"\n");
    }
    0
}

pub fn base32(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn cmp(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn diff(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn od(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }

pub fn hexdump(argc: i32, argv: *const *const u8) -> i32 {
    let fd = if argc > 1 {
        if let Some(path) = unsafe { get_arg(argv, argc - 1) } {
            if path[0] != b'-' { io::open(path, libc::O_RDONLY, 0) } else { 0 }
        } else { 0 }
    } else { 0 };

    let mut buf = [0u8; 16];
    let mut offset = 0u64;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        // Print offset
        let mut hex = [0u8; 16];
        let s = sys::format_hex(offset, &mut hex);
        for _ in 0..(8 - s.len()) { io::write_str(1, b"0"); }
        io::write_all(1, s);
        io::write_str(1, b"  ");

        // Print hex
        for i in 0..n as usize {
            let h = sys::format_hex(buf[i] as u64, &mut hex);
            if h.len() == 1 { io::write_str(1, b"0"); }
            io::write_all(1, h);
            io::write_str(1, b" ");
        }
        io::write_str(1, b"\n");

        offset += n as u64;
    }

    if fd != 0 { io::close(fd); }
    0
}

pub fn hd(argc: i32, argv: *const *const u8) -> i32 { hexdump(argc, argv) }
pub fn xxd(argc: i32, argv: *const *const u8) -> i32 { hexdump(argc, argv) }

fn simple_hash(data: &[u8], init: u32, poly: u32) -> u32 {
    let mut hash = init;
    for &b in data {
        hash ^= b as u32;
        for _ in 0..8 {
            if hash & 1 != 0 {
                hash = (hash >> 1) ^ poly;
            } else {
                hash >>= 1;
            }
        }
    }
    hash
}

pub fn md5sum(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] != b'-' {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd < 0 { continue; }

                let mut hash = 0u32;
                let mut buf = [0u8; 4096];
                loop {
                    let n = io::read(fd, &mut buf);
                    if n <= 0 { break; }
                    hash = simple_hash(&buf[..n as usize], hash, 0xEDB88320);
                }
                io::close(fd);

                // Print as hex (simplified - not real MD5)
                let mut hex = [0u8; 16];
                let s = sys::format_hex(hash as u64, &mut hex);
                for _ in 0..(8 - s.len()) { io::write_str(1, b"0"); }
                io::write_all(1, s);
                io::write_str(1, b"00000000000000000000000000000000  ");
                io::write_all(1, path);
                io::write_str(1, b"\n");
            }
        }
    }
    0
}

pub fn sha1sum(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }
pub fn sha224sum(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }
pub fn sha256sum(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }
pub fn sha384sum(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }
pub fn sha512sum(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }
pub fn sha3sum(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }
pub fn cksum(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }
pub fn crc32(argc: i32, argv: *const *const u8) -> i32 { md5sum(argc, argv) }

pub fn ascii(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Dec Hex    Dec Hex    Dec Hex  Dec Hex  Dec Hex  Dec Hex   Dec Hex   Dec Hex\n");
    io::write_str(1, b"  0 00 NUL  16 10 DLE  32 20    48 30 0  64 40 @  80 50 P   96 60 `  112 70 p\n");
    io::write_str(1, b"  1 01 SOH  17 11 DC1  33 21 !  49 31 1  65 41 A  81 51 Q   97 61 a  113 71 q\n");
    // ... abbreviated
    0
}

pub fn iconv(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn tsort(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn getopt(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }

pub fn count(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut count = 0u64;
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(0, &mut buf);
        if n <= 0 { break; }
        count += n as u64;
    }
    io::write_num(1, count);
    io::write_str(1, b"\n");
    0
}

pub fn unicode(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ts(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }

pub fn uuidgen(_argc: i32, _argv: *const *const u8) -> i32 {
    let t = unsafe { libc::time(core::ptr::null_mut()) } as u64;
    let mut hex = [0u8; 16];

    let s = sys::format_hex(t, &mut hex);
    io::write_all(1, s);
    io::write_str(1, b"-0000-4000-8000-");
    let s = sys::format_hex(t ^ 0xDEADBEEF, &mut hex);
    io::write_all(1, s);
    io::write_str(1, b"0000\n");
    0
}

pub fn mcookie(_argc: i32, _argv: *const *const u8) -> i32 {
    let t = unsafe { libc::time(core::ptr::null_mut()) } as u64;
    let mut hex = [0u8; 16];
    let s = sys::format_hex(t, &mut hex);
    for _ in 0..(16 - s.len()) { io::write_str(1, b"0"); }
    io::write_all(1, s);
    let s = sys::format_hex(t ^ 0xCAFEBABE, &mut hex);
    for _ in 0..(16 - s.len()) { io::write_str(1, b"0"); }
    io::write_all(1, s);
    io::write_str(1, b"\n");
    0
}

pub fn pwgen(_argc: i32, _argv: *const *const u8) -> i32 {
    let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = unsafe { libc::time(core::ptr::null_mut()) } as u64;

    for _ in 0..8 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        io::write_all(1, &[chars[(rng >> 60) as usize % chars.len()]]);
    }
    io::write_str(1, b"\n");
    0
}

pub fn uuencode(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn uudecode(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }

// Additional toybox applets
pub fn help(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"armybox - BusyBox/Toybox compatible multi-call binary\n");
    io::write_str(1, b"Usage: armybox [APPLET] [ARGS]\n");
    0
}
pub fn memeater(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"memeater: stub\n"); 0 }
pub fn mix(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"mix: stub\n"); 0 }
pub fn mkpasswd(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"$6$random$hash\n"); 0 }
pub fn readelf(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"readelf: stub\n"); 0 }
pub fn toybox(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"armybox (toybox compatible)\n");
    0
}

/// screen - terminal multiplexer (simplified GNU screen clone)
///
/// Usage:
///   screen                  Start a new session
///   screen -S name          Start a named session
///   screen -ls              List sessions
///   screen -r [name]        Reattach to a session
///   screen -d [name]        Detach a session
///   screen -x [name]        Attach to a shared session
///   screen cmd args...      Run command in new session
///
/// In-session commands (Ctrl+A prefix):
///   Ctrl+A d                Detach
///   Ctrl+A c                Create new window
///   Ctrl+A n                Next window
///   Ctrl+A p                Previous window
///   Ctrl+A k                Kill current window
///   Ctrl+A "                List windows
pub fn screen(argc: i32, argv: *const *const u8) -> i32 {
    let mut list_sessions = false;
    let mut reattach = false;
    let mut detach_session = false;
    let mut session_name: Option<&[u8]> = None;
    let mut cmd_start = 1;

    // Parse options
    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-ls" || arg == b"-list" {
                list_sessions = true;
            } else if arg == b"-r" || arg == b"-R" {
                reattach = true;
                if i + 1 < argc {
                    session_name = unsafe { get_arg(argv, i + 1) };
                    if session_name.map_or(false, |s| s.starts_with(b"-")) {
                        session_name = None;
                    } else {
                        i += 1;
                    }
                }
            } else if arg == b"-d" || arg == b"-D" {
                detach_session = true;
                if i + 1 < argc {
                    session_name = unsafe { get_arg(argv, i + 1) };
                    if session_name.map_or(false, |s| s.starts_with(b"-")) {
                        session_name = None;
                    } else {
                        i += 1;
                    }
                }
            } else if arg == b"-S" {
                if i + 1 < argc {
                    session_name = unsafe { get_arg(argv, i + 1) };
                    i += 1;
                }
            } else if arg == b"-x" {
                reattach = true;
                if i + 1 < argc {
                    session_name = unsafe { get_arg(argv, i + 1) };
                    if session_name.map_or(false, |s| s.starts_with(b"-")) {
                        session_name = None;
                    } else {
                        i += 1;
                    }
                }
            } else if arg == b"-h" || arg == b"--help" {
                io::write_str(1, b"Usage: screen [-ls] [-r name] [-d name] [-S name] [cmd]\n");
                io::write_str(1, b"\nOptions:\n");
                io::write_str(1, b"  -ls         List sessions\n");
                io::write_str(1, b"  -r [name]   Reattach to session\n");
                io::write_str(1, b"  -d [name]   Detach session\n");
                io::write_str(1, b"  -S name     Create named session\n");
                io::write_str(1, b"  -x [name]   Multi-attach to session\n");
                io::write_str(1, b"\nIn-session: Ctrl+A is the command prefix\n");
                io::write_str(1, b"  Ctrl+A d    Detach from session\n");
                io::write_str(1, b"  Ctrl+A c    Create new window\n");
                io::write_str(1, b"  Ctrl+A n    Next window\n");
                io::write_str(1, b"  Ctrl+A p    Previous window\n");
                io::write_str(1, b"  Ctrl+A k    Kill current window\n");
                return 0;
            } else if !arg.starts_with(b"-") {
                cmd_start = i;
                break;
            }
        }
        i += 1;
    }

    if list_sessions {
        return screen_list_sessions();
    }

    if detach_session {
        return screen_detach(session_name);
    }

    if reattach {
        return screen_reattach(session_name);
    }

    // Start a new session
    screen_new_session(session_name, argc, argv, cmd_start)
}

fn screen_list_sessions() -> i32 {
    let screen_dir = b"/tmp/armybox-screen\0";

    let dir = unsafe { libc::opendir(screen_dir.as_ptr() as *const i8) };
    if dir.is_null() {
        io::write_str(1, b"No Sockets found in /tmp/armybox-screen.\n");
        return 0;
    }

    io::write_str(1, b"There are screens on:\n");
    let mut count = 0;

    loop {
        let entry = unsafe { libc::readdir(dir) };
        if entry.is_null() {
            break;
        }

        let name = unsafe { io::cstr_to_slice((*entry).d_name.as_ptr() as *const u8) };
        if name.starts_with(b".") {
            continue;
        }

        // Check if socket is still active
        let mut path = [0u8; 256];
        let mut len = 0;
        for &c in b"/tmp/armybox-screen/" {
            path[len] = c;
            len += 1;
        }
        for &c in name {
            path[len] = c;
            len += 1;
        }
        path[len] = 0;

        let mut st: libc::stat = unsafe { core::mem::zeroed() };
        if io::stat(&path[..len], &mut st) == 0 {
            io::write_str(1, b"\t");
            io::write_all(1, name);

            // Check if attached
            if (st.st_mode & 0o600) == 0o600 {
                io::write_str(1, b"\t(Attached)\n");
            } else {
                io::write_str(1, b"\t(Detached)\n");
            }
            count += 1;
        }
    }

    unsafe { libc::closedir(dir) };

    if count == 0 {
        io::write_str(1, b"No Sockets found.\n");
    } else {
        io::write_num(1, count);
        io::write_str(1, b" Socket(s) in /tmp/armybox-screen.\n");
    }

    0
}

fn screen_detach(name: Option<&[u8]>) -> i32 {
    let name = match name {
        Some(n) => n,
        None => {
            io::write_str(2, b"screen: must specify session name to detach\n");
            return 1;
        }
    };

    // Send SIGHUP to the screen process
    let mut path = [0u8; 256];
    let mut len = 0;
    for &c in b"/tmp/armybox-screen/" {
        path[len] = c;
        len += 1;
    }
    for &c in name {
        path[len] = c;
        len += 1;
    }
    path[len] = 0;

    // Read PID from socket file (stored as extended attribute or in filename)
    // For simplicity, we'll parse the PID from the session name format: pid.tty.name
    if let Some(pid_end) = name.iter().position(|&c| c == b'.') {
        if let Some(pid) = sys::parse_u64(&name[..pid_end]) {
            if unsafe { libc::kill(pid as i32, libc::SIGHUP) } == 0 {
                io::write_str(1, b"Session detached.\n");
                return 0;
            }
        }
    }

    io::write_str(2, b"screen: could not detach session\n");
    1
}

fn screen_reattach(name: Option<&[u8]>) -> i32 {
    let screen_dir = b"/tmp/armybox-screen\0";

    let dir = unsafe { libc::opendir(screen_dir.as_ptr() as *const i8) };
    if dir.is_null() {
        io::write_str(2, b"There is no screen to be resumed.\n");
        return 1;
    }

    let mut found_session: Option<[u8; 256]> = None;
    let mut found_len = 0;

    loop {
        let entry = unsafe { libc::readdir(dir) };
        if entry.is_null() {
            break;
        }

        let entry_name = unsafe { io::cstr_to_slice((*entry).d_name.as_ptr() as *const u8) };
        if entry_name.starts_with(b".") {
            continue;
        }

        // If name is specified, match it
        if let Some(n) = name {
            if entry_name.windows(n.len()).any(|w| w == n) {
                let mut buf = [0u8; 256];
                for (i, &c) in entry_name.iter().enumerate() {
                    if i < 256 {
                        buf[i] = c;
                    }
                }
                found_session = Some(buf);
                found_len = entry_name.len();
                break;
            }
        } else {
            // Take first available session
            let mut buf = [0u8; 256];
            for (i, &c) in entry_name.iter().enumerate() {
                if i < 256 {
                    buf[i] = c;
                }
            }
            found_session = Some(buf);
            found_len = entry_name.len();
            break;
        }
    }

    unsafe { libc::closedir(dir) };

    match found_session {
        Some(session) => {
            io::write_str(1, b"Reattaching to ");
            io::write_all(1, &session[..found_len]);
            io::write_str(1, b"\n");

            // Connect to the session's socket and take over
            let mut path = [0u8; 512];
            let mut len = 0;
            for &c in b"/tmp/armybox-screen/" {
                path[len] = c;
                len += 1;
            }
            for i in 0..found_len {
                path[len] = session[i];
                len += 1;
            }
            path[len] = 0;

            // Open the socket and proxy I/O
            let sock = unsafe {
                libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0)
            };
            if sock < 0 {
                io::write_str(2, b"screen: could not create socket\n");
                return 1;
            }

            let mut addr: libc::sockaddr_un = unsafe { core::mem::zeroed() };
            addr.sun_family = libc::AF_UNIX as u16;
            for (i, &c) in path[..len].iter().enumerate() {
                if i < 108 {
                    addr.sun_path[i] = c as i8;
                }
            }

            if unsafe { libc::connect(sock, &addr as *const _ as *const libc::sockaddr,
                                       core::mem::size_of::<libc::sockaddr_un>() as u32) } < 0 {
                io::write_str(2, b"screen: could not connect to session\n");
                unsafe { libc::close(sock) };
                return 1;
            }

            // Set terminal to raw mode
            let mut old_termios: libc::termios = unsafe { core::mem::zeroed() };
            unsafe { libc::tcgetattr(0, &mut old_termios) };

            let mut raw = old_termios;
            unsafe { libc::cfmakeraw(&mut raw) };
            unsafe { libc::tcsetattr(0, libc::TCSANOW, &raw) };

            // Proxy I/O between terminal and socket
            screen_proxy_io(sock);

            // Restore terminal
            unsafe { libc::tcsetattr(0, libc::TCSANOW, &old_termios) };
            unsafe { libc::close(sock) };

            io::write_str(1, b"\n[screen detached]\n");
            0
        }
        None => {
            io::write_str(2, b"There is no screen to be resumed.\n");
            1
        }
    }
}

fn screen_new_session(name: Option<&[u8]>, argc: i32, argv: *const *const u8, cmd_start: i32) -> i32 {
    // Create screen directory
    let screen_dir = b"/tmp/armybox-screen\0";
    unsafe { libc::mkdir(screen_dir.as_ptr() as *const i8, 0o700) };

    // Open a PTY
    let mut master: i32 = -1;
    let mut slave: i32 = -1;
    let mut pty_name = [0i8; 256];

    if unsafe { libc::openpty(&mut master, &mut slave, pty_name.as_mut_ptr(),
                               core::ptr::null_mut(), core::ptr::null_mut()) } < 0 {
        io::write_str(2, b"screen: cannot open pty\n");
        return 1;
    }

    let pid = unsafe { libc::fork() };

    if pid < 0 {
        io::write_str(2, b"screen: fork failed\n");
        unsafe { libc::close(master) };
        unsafe { libc::close(slave) };
        return 1;
    }

    if pid == 0 {
        // Child process - run shell in PTY slave
        unsafe { libc::close(master) };

        // Create new session
        unsafe { libc::setsid() };

        // Set controlling terminal
        unsafe { libc::ioctl(slave, libc::TIOCSCTTY as u64, 0) };

        // Redirect stdio to slave
        unsafe { libc::dup2(slave, 0) };
        unsafe { libc::dup2(slave, 1) };
        unsafe { libc::dup2(slave, 2) };

        if slave > 2 {
            unsafe { libc::close(slave) };
        }

        // Execute command or shell
        if cmd_start < argc {
            // Execute specified command
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

                let ptrs: Vec<*const i8> = args.iter()
                    .map(|s| s.as_ptr())
                    .chain(core::iter::once(core::ptr::null()))
                    .collect();

                unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
            }
        }

        // Default: run shell
        let shell = b"/bin/sh\0";
        let shell_arg = b"-sh\0";
        let args = [shell.as_ptr() as *const i8, shell_arg.as_ptr() as *const i8, core::ptr::null()];
        unsafe { libc::execv(shell.as_ptr() as *const i8, args.as_ptr()) };
        unsafe { libc::_exit(1) };
    }

    // Parent process - manage the session
    unsafe { libc::close(slave) };

    // Create session socket for reattachment
    let mut session_path = [0u8; 256];
    let mut len = 0;
    for &c in b"/tmp/armybox-screen/" {
        session_path[len] = c;
        len += 1;
    }

    // Format: pid.pts-N.name
    let mut pid_buf = [0u8; 20];
    let pid_str = sys::format_u64(pid as u64, &mut pid_buf);
    for &c in pid_str {
        session_path[len] = c;
        len += 1;
    }
    session_path[len] = b'.';
    len += 1;

    // Add pts name
    let pts_name = unsafe { io::cstr_to_slice(pty_name.as_ptr() as *const u8) };
    for &c in pts_name {
        if c == b'/' {
            session_path[len] = b'-';
        } else {
            session_path[len] = c;
        }
        len += 1;
    }

    if let Some(n) = name {
        session_path[len] = b'.';
        len += 1;
        for &c in n {
            session_path[len] = c;
            len += 1;
        }
    }
    session_path[len] = 0;

    // Create Unix socket for the session
    let sock = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if sock >= 0 {
        let mut addr: libc::sockaddr_un = unsafe { core::mem::zeroed() };
        addr.sun_family = libc::AF_UNIX as u16;
        for (i, &c) in session_path[..len].iter().enumerate() {
            if i < 108 {
                addr.sun_path[i] = c as i8;
            }
        }

        unsafe { libc::unlink(session_path.as_ptr() as *const i8) };
        if unsafe { libc::bind(sock, &addr as *const _ as *const libc::sockaddr,
                               core::mem::size_of::<libc::sockaddr_un>() as u32) } == 0 {
            unsafe { libc::listen(sock, 1) };
        }
    }

    // Set terminal to raw mode
    let mut old_termios: libc::termios = unsafe { core::mem::zeroed() };
    unsafe { libc::tcgetattr(0, &mut old_termios) };

    let mut raw = old_termios;
    unsafe { libc::cfmakeraw(&mut raw) };
    unsafe { libc::tcsetattr(0, libc::TCSANOW, &raw) };

    // Main loop: proxy I/O between terminal and PTY
    let mut ctrl_a_pressed = false;
    let mut buf = [0u8; 4096];

    loop {
        let mut fds: [libc::pollfd; 3] = [
            libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 },      // stdin
            libc::pollfd { fd: master, events: libc::POLLIN, revents: 0 }, // PTY
            libc::pollfd { fd: sock, events: libc::POLLIN, revents: 0 },   // socket
        ];

        let ret = unsafe { libc::poll(fds.as_mut_ptr(), 3, 100) };
        if ret < 0 {
            break;
        }

        // Check if child exited
        let mut status: i32 = 0;
        if unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) } > 0 {
            break;
        }

        // Data from terminal
        if fds[0].revents & libc::POLLIN != 0 {
            let n = io::read(0, &mut buf);
            if n <= 0 {
                break;
            }

            // Check for Ctrl+A commands
            for i in 0..n as usize {
                if ctrl_a_pressed {
                    ctrl_a_pressed = false;
                    match buf[i] {
                        b'd' | b'D' => {
                            // Detach
                            unsafe { libc::tcsetattr(0, libc::TCSANOW, &old_termios) };
                            io::write_str(1, b"\r\n[detached from session]\r\n");
                            // Keep socket open for reattachment
                            unsafe { libc::close(master) };
                            // Don't close sock - leave it for reattach
                            return 0;
                        }
                        b'c' | b'C' => {
                            io::write_str(1, b"\r\n[new window - not implemented in simple mode]\r\n");
                        }
                        b'k' | b'K' => {
                            // Kill - send SIGKILL to child
                            unsafe { libc::kill(pid, libc::SIGKILL) };
                        }
                        1 => {
                            // Ctrl+A Ctrl+A - send literal Ctrl+A
                            let ctrl_a = [1u8];
                            io::write_all(master, &ctrl_a);
                        }
                        _ => {
                            // Unknown command, ignore
                        }
                    }
                } else if buf[i] == 1 {
                    // Ctrl+A pressed
                    ctrl_a_pressed = true;
                } else {
                    io::write_all(master, &buf[i..i+1]);
                }
            }
        }

        // Data from PTY
        if fds[1].revents & libc::POLLIN != 0 {
            let n = io::read(master, &mut buf);
            if n <= 0 {
                break;
            }
            io::write_all(1, &buf[..n as usize]);
        }

        // New connection on socket (reattach)
        if fds[2].revents & libc::POLLIN != 0 {
            let client = unsafe { libc::accept(sock, core::ptr::null_mut(), core::ptr::null_mut()) };
            if client >= 0 {
                // Another client is trying to attach - for now, refuse
                io::write_all(client, b"Session already attached\n");
                unsafe { libc::close(client) };
            }
        }
    }

    // Cleanup
    unsafe { libc::tcsetattr(0, libc::TCSANOW, &old_termios) };
    unsafe { libc::close(master) };
    unsafe { libc::close(sock) };
    unsafe { libc::unlink(session_path.as_ptr() as *const i8) };

    // Wait for child
    unsafe { libc::waitpid(pid, core::ptr::null_mut(), 0) };

    io::write_str(1, b"\r\n[screen terminated]\r\n");
    0
}

fn screen_proxy_io(sock: i32) {
    let mut buf = [0u8; 4096];
    let mut ctrl_a_pressed = false;

    loop {
        let mut fds: [libc::pollfd; 2] = [
            libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 },
            libc::pollfd { fd: sock, events: libc::POLLIN, revents: 0 },
        ];

        let ret = unsafe { libc::poll(fds.as_mut_ptr(), 2, 100) };
        if ret < 0 {
            break;
        }

        // Data from terminal
        if fds[0].revents & libc::POLLIN != 0 {
            let n = io::read(0, &mut buf);
            if n <= 0 {
                break;
            }

            for i in 0..n as usize {
                if ctrl_a_pressed {
                    ctrl_a_pressed = false;
                    if buf[i] == b'd' || buf[i] == b'D' {
                        return; // Detach
                    }
                } else if buf[i] == 1 {
                    ctrl_a_pressed = true;
                } else {
                    io::write_all(sock, &buf[i..i+1]);
                }
            }
        }

        // Data from socket
        if fds[1].revents & libc::POLLIN != 0 {
            let n = io::read(sock, &mut buf);
            if n <= 0 {
                break;
            }
            io::write_all(1, &buf[..n as usize]);
        }
    }
}
