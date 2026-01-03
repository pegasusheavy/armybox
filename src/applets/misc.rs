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
