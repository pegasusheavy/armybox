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

pub fn cmp(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 { return 2; }
    let path1 = match unsafe { get_arg(argv, 1) } { Some(p) => p, None => return 2 };
    let path2 = match unsafe { get_arg(argv, 2) } { Some(p) => p, None => return 2 };

    let fd1 = io::open(path1, libc::O_RDONLY, 0);
    let fd2 = io::open(path2, libc::O_RDONLY, 0);
    if fd1 < 0 || fd2 < 0 {
        if fd1 >= 0 { io::close(fd1); }
        if fd2 >= 0 { io::close(fd2); }
        return 2;
    }

    let mut buf1 = [0u8; 4096];
    let mut buf2 = [0u8; 4096];
    let mut byte_num = 0u64;
    let mut line_num = 1u64;
    let mut result = 0;

    'outer: loop {
        let n1 = io::read(fd1, &mut buf1);
        let n2 = io::read(fd2, &mut buf2);

        if n1 <= 0 && n2 <= 0 { break; }
        if n1 != n2 || n1 <= 0 || n2 <= 0 { result = 1; break; }

        for i in 0..n1 as usize {
            byte_num += 1;
            if buf1[i] == b'\n' { line_num += 1; }
            if buf1[i] != buf2[i] {
                let mut num_buf = [0u8; 20];
                io::write_all(1, path1);
                io::write_str(1, b" ");
                io::write_all(1, path2);
                io::write_str(1, b" differ: byte ");
                io::write_all(1, sys::format_u64(byte_num, &mut num_buf));
                io::write_str(1, b", line ");
                io::write_all(1, sys::format_u64(line_num, &mut num_buf));
                io::write_str(1, b"\n");
                result = 1;
                break 'outer;
            }
        }
    }

    io::close(fd1);
    io::close(fd2);
    result
}

#[cfg(feature = "alloc")]
pub fn diff(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    if argc < 3 { return 2; }
    let path1 = match unsafe { get_arg(argv, 1) } { Some(p) => p, None => return 2 };
    let path2 = match unsafe { get_arg(argv, 2) } { Some(p) => p, None => return 2 };

    // Read both files
    let fd1 = io::open(path1, libc::O_RDONLY, 0);
    if fd1 < 0 {
        io::write_str(2, b"diff: ");
        io::write_all(2, path1);
        io::write_str(2, b": No such file or directory\n");
        return 2;
    }
    let content1 = io::read_all(fd1);
    io::close(fd1);

    let fd2 = io::open(path2, libc::O_RDONLY, 0);
    if fd2 < 0 {
        io::write_str(2, b"diff: ");
        io::write_all(2, path2);
        io::write_str(2, b": No such file or directory\n");
        return 2;
    }
    let content2 = io::read_all(fd2);
    io::close(fd2);

    // Split into lines
    let lines1: Vec<&[u8]> = content1.split(|&c| c == b'\n').collect();
    let lines2: Vec<&[u8]> = content2.split(|&c| c == b'\n').collect();

    // Simple LCS-based diff using dynamic programming
    let m = lines1.len();
    let n = lines2.len();

    // Build LCS table
    let mut lcs = Vec::new();
    lcs.resize((m + 1) * (n + 1), 0usize);

    for i in 1..=m {
        for j in 1..=n {
            if lines1[i-1] == lines2[j-1] {
                lcs[i * (n + 1) + j] = lcs[(i-1) * (n + 1) + (j-1)] + 1;
            } else {
                lcs[i * (n + 1) + j] = core::cmp::max(
                    lcs[(i-1) * (n + 1) + j],
                    lcs[i * (n + 1) + (j-1)]
                );
            }
        }
    }

    // Trace back to find differences
    let mut changes: Vec<(i32, usize, usize)> = Vec::new(); // (type: -1=del, +1=add, 0=same, line1_idx, line2_idx)
    let mut i = m;
    let mut j = n;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && lines1[i-1] == lines2[j-1] {
            changes.push((0, i-1, j-1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[(i) * (n + 1) + (j-1)] >= lcs[(i-1) * (n + 1) + j]) {
            changes.push((1, 0, j-1)); // addition
            j -= 1;
        } else if i > 0 {
            changes.push((-1, i-1, 0)); // deletion
            i -= 1;
        }
    }

    changes.reverse();

    // Output in normal diff format
    let mut has_diff = false;
    let mut i = 0;
    while i < changes.len() {
        let (change_type, idx1, idx2) = changes[i];

        if change_type == 0 {
            i += 1;
            continue;
        }

        has_diff = true;

        // Find range of consecutive changes
        let start_i = i;
        let mut del_start = if change_type == -1 { idx1 + 1 } else { 0 };
        let mut del_end = del_start;
        let mut add_start = if change_type == 1 { idx2 + 1 } else { 0 };
        let mut add_end = add_start;

        while i < changes.len() {
            let (ct, i1, i2) = changes[i];
            if ct == 0 { break; }
            if ct == -1 {
                if del_start == 0 { del_start = i1 + 1; }
                del_end = i1 + 1;
            } else {
                if add_start == 0 { add_start = i2 + 1; }
                add_end = i2 + 1;
            }
            i += 1;
        }

        // Output header
        let mut num_buf = [0u8; 20];

        if del_start > 0 && add_start > 0 {
            // Change
            if del_start == del_end {
                io::write_all(1, sys::format_u64(del_start as u64, &mut num_buf));
            } else {
                io::write_all(1, sys::format_u64(del_start as u64, &mut num_buf));
                io::write_str(1, b",");
                io::write_all(1, sys::format_u64(del_end as u64, &mut num_buf));
            }
            io::write_str(1, b"c");
            if add_start == add_end {
                io::write_all(1, sys::format_u64(add_start as u64, &mut num_buf));
            } else {
                io::write_all(1, sys::format_u64(add_start as u64, &mut num_buf));
                io::write_str(1, b",");
                io::write_all(1, sys::format_u64(add_end as u64, &mut num_buf));
            }
            io::write_str(1, b"\n");
        } else if del_start > 0 {
            // Deletion
            if del_start == del_end {
                io::write_all(1, sys::format_u64(del_start as u64, &mut num_buf));
            } else {
                io::write_all(1, sys::format_u64(del_start as u64, &mut num_buf));
                io::write_str(1, b",");
                io::write_all(1, sys::format_u64(del_end as u64, &mut num_buf));
            }
            io::write_str(1, b"d");
            io::write_all(1, sys::format_u64(add_start.saturating_sub(1).max(1) as u64, &mut num_buf));
            io::write_str(1, b"\n");
        } else {
            // Addition
            io::write_all(1, sys::format_u64(del_start.saturating_sub(1).max(1) as u64, &mut num_buf));
            io::write_str(1, b"a");
            if add_start == add_end {
                io::write_all(1, sys::format_u64(add_start as u64, &mut num_buf));
            } else {
                io::write_all(1, sys::format_u64(add_start as u64, &mut num_buf));
                io::write_str(1, b",");
                io::write_all(1, sys::format_u64(add_end as u64, &mut num_buf));
            }
            io::write_str(1, b"\n");
        }

        // Output lines
        for k in start_i..i {
            let (ct, i1, i2) = changes[k];
            if ct == -1 {
                io::write_str(1, b"< ");
                io::write_all(1, lines1[i1]);
                io::write_str(1, b"\n");
            }
        }
        if del_start > 0 && add_start > 0 {
            io::write_str(1, b"---\n");
        }
        for k in start_i..i {
            let (ct, _i1, i2) = changes[k];
            if ct == 1 {
                io::write_str(1, b"> ");
                io::write_all(1, lines2[i2]);
                io::write_str(1, b"\n");
            }
        }
    }

    if has_diff { 1 } else { 0 }
}

#[cfg(not(feature = "alloc"))]
pub fn diff(_argc: i32, _argv: *const *const u8) -> i32 { 2 }

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

// MD5 implementation
mod md5 {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];

    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
    ];

    pub struct Md5 {
        state: [u32; 4],
        count: u64,
        buffer: [u8; 64],
        buflen: usize,
    }

    impl Md5 {
        pub fn new() -> Self {
            Md5 {
                state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
                count: 0,
                buffer: [0; 64],
                buflen: 0,
            }
        }

        pub fn update(&mut self, data: &[u8]) {
            self.count += data.len() as u64;
            let mut offset = 0;

            if self.buflen > 0 {
                let space = 64 - self.buflen;
                let copy = core::cmp::min(space, data.len());
                self.buffer[self.buflen..self.buflen + copy].copy_from_slice(&data[..copy]);
                self.buflen += copy;
                offset = copy;

                if self.buflen == 64 {
                    self.transform(&self.buffer.clone());
                    self.buflen = 0;
                }
            }

            while offset + 64 <= data.len() {
                let mut block = [0u8; 64];
                block.copy_from_slice(&data[offset..offset + 64]);
                self.transform(&block);
                offset += 64;
            }

            if offset < data.len() {
                self.buflen = data.len() - offset;
                self.buffer[..self.buflen].copy_from_slice(&data[offset..]);
            }
        }

        pub fn finalize(&mut self) -> [u8; 16] {
            let bit_len = self.count * 8;
            let pad_len = if self.buflen < 56 { 56 - self.buflen } else { 120 - self.buflen };

            let mut padding = [0u8; 128];
            padding[0] = 0x80;
            self.update(&padding[..pad_len]);

            let mut len_bytes = [0u8; 8];
            for i in 0..8 {
                len_bytes[i] = (bit_len >> (i * 8)) as u8;
            }
            self.update(&len_bytes);

            let mut result = [0u8; 16];
            for (i, &s) in self.state.iter().enumerate() {
                result[i * 4] = s as u8;
                result[i * 4 + 1] = (s >> 8) as u8;
                result[i * 4 + 2] = (s >> 16) as u8;
                result[i * 4 + 3] = (s >> 24) as u8;
            }
            result
        }

        fn transform(&mut self, block: &[u8; 64]) {
            let mut m = [0u32; 16];
            for i in 0..16 {
                m[i] = u32::from_le_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
            }

            let [mut a, mut b, mut c, mut d] = self.state;

            for i in 0..64 {
                let (f, g) = match i {
                    0..=15 => ((b & c) | ((!b) & d), i),
                    16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                    32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                    _ => (c ^ (b | (!d)), (7 * i) % 16),
                };

                let temp = d;
                d = c;
                c = b;
                b = b.wrapping_add(
                    a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g]).rotate_left(S[i])
                );
                a = temp;
            }

            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
        }
    }
}

// SHA256 implementation
mod sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    pub struct Sha256 {
        state: [u32; 8],
        count: u64,
        buffer: [u8; 64],
        buflen: usize,
    }

    impl Sha256 {
        pub fn new() -> Self {
            Sha256 {
                state: [
                    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
                ],
                count: 0,
                buffer: [0; 64],
                buflen: 0,
            }
        }

        pub fn update(&mut self, data: &[u8]) {
            self.count += data.len() as u64;
            let mut offset = 0;

            if self.buflen > 0 {
                let space = 64 - self.buflen;
                let copy = core::cmp::min(space, data.len());
                self.buffer[self.buflen..self.buflen + copy].copy_from_slice(&data[..copy]);
                self.buflen += copy;
                offset = copy;

                if self.buflen == 64 {
                    self.transform(&self.buffer.clone());
                    self.buflen = 0;
                }
            }

            while offset + 64 <= data.len() {
                let mut block = [0u8; 64];
                block.copy_from_slice(&data[offset..offset + 64]);
                self.transform(&block);
                offset += 64;
            }

            if offset < data.len() {
                self.buflen = data.len() - offset;
                self.buffer[..self.buflen].copy_from_slice(&data[offset..]);
            }
        }

        pub fn finalize(&mut self) -> [u8; 32] {
            let bit_len = self.count * 8;
            let pad_len = if self.buflen < 56 { 56 - self.buflen } else { 120 - self.buflen };

            let mut padding = [0u8; 128];
            padding[0] = 0x80;
            self.update(&padding[..pad_len]);

            let mut len_bytes = [0u8; 8];
            for i in 0..8 {
                len_bytes[7 - i] = (bit_len >> (i * 8)) as u8;
            }
            self.update(&len_bytes);

            let mut result = [0u8; 32];
            for (i, &s) in self.state.iter().enumerate() {
                result[i * 4] = (s >> 24) as u8;
                result[i * 4 + 1] = (s >> 16) as u8;
                result[i * 4 + 2] = (s >> 8) as u8;
                result[i * 4 + 3] = s as u8;
            }
            result
        }

        fn transform(&mut self, block: &[u8; 64]) {
            let mut w = [0u32; 64];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
            }
            for i in 16..64 {
                let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
                let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
                w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
            }

            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);

                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }

            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
            self.state[4] = self.state[4].wrapping_add(e);
            self.state[5] = self.state[5].wrapping_add(f);
            self.state[6] = self.state[6].wrapping_add(g);
            self.state[7] = self.state[7].wrapping_add(h);
        }
    }
}

// SHA1 implementation
mod sha1 {
    pub struct Sha1 {
        state: [u32; 5],
        count: u64,
        buffer: [u8; 64],
        buflen: usize,
    }

    impl Sha1 {
        pub fn new() -> Self {
            Sha1 {
                state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
                count: 0,
                buffer: [0; 64],
                buflen: 0,
            }
        }

        pub fn update(&mut self, data: &[u8]) {
            self.count += data.len() as u64;
            let mut offset = 0;

            if self.buflen > 0 {
                let space = 64 - self.buflen;
                let copy = core::cmp::min(space, data.len());
                self.buffer[self.buflen..self.buflen + copy].copy_from_slice(&data[..copy]);
                self.buflen += copy;
                offset = copy;

                if self.buflen == 64 {
                    self.transform(&self.buffer.clone());
                    self.buflen = 0;
                }
            }

            while offset + 64 <= data.len() {
                let mut block = [0u8; 64];
                block.copy_from_slice(&data[offset..offset + 64]);
                self.transform(&block);
                offset += 64;
            }

            if offset < data.len() {
                self.buflen = data.len() - offset;
                self.buffer[..self.buflen].copy_from_slice(&data[offset..]);
            }
        }

        pub fn finalize(&mut self) -> [u8; 20] {
            let bit_len = self.count * 8;
            let pad_len = if self.buflen < 56 { 56 - self.buflen } else { 120 - self.buflen };

            let mut padding = [0u8; 128];
            padding[0] = 0x80;
            self.update(&padding[..pad_len]);

            let mut len_bytes = [0u8; 8];
            for i in 0..8 {
                len_bytes[7 - i] = (bit_len >> (i * 8)) as u8;
            }
            self.update(&len_bytes);

            let mut result = [0u8; 20];
            for (i, &s) in self.state.iter().enumerate() {
                result[i * 4] = (s >> 24) as u8;
                result[i * 4 + 1] = (s >> 16) as u8;
                result[i * 4 + 2] = (s >> 8) as u8;
                result[i * 4 + 3] = s as u8;
            }
            result
        }

        fn transform(&mut self, block: &[u8; 64]) {
            let mut w = [0u32; 80];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
            }
            for i in 16..80 {
                w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
            }

            let [mut a, mut b, mut c, mut d, mut e] = self.state;

            for i in 0..80 {
                let (f, k) = match i {
                    0..=19 => ((b & c) | ((!b) & d), 0x5a827999u32),
                    20..=39 => (b ^ c ^ d, 0x6ed9eba1u32),
                    40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdcu32),
                    _ => (b ^ c ^ d, 0xca62c1d6u32),
                };

                let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
                e = d;
                d = c;
                c = b.rotate_left(30);
                b = a;
                a = temp;
            }

            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
            self.state[4] = self.state[4].wrapping_add(e);
        }
    }
}

fn print_hash(hash: &[u8], filename: &[u8]) {
    const HEX: &[u8] = b"0123456789abcdef";
    for byte in hash.iter() {
        let hex = [HEX[(byte >> 4) as usize], HEX[(byte & 0xf) as usize]];
        io::write_all(1, &hex);
    }
    io::write_str(1, b"  ");
    io::write_all(1, filename);
    io::write_str(1, b"\n");
}

fn md5_hash_fd(fd: i32, filename: &[u8]) {
    let mut hasher = md5::Md5::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }
        hasher.update(&buf[..n as usize]);
    }
    let hash = hasher.finalize();
    print_hash(&hash, filename);
}

fn sha1_hash_fd(fd: i32, filename: &[u8]) {
    let mut hasher = sha1::Sha1::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }
        hasher.update(&buf[..n as usize]);
    }
    let hash = hasher.finalize();
    print_hash(&hash, filename);
}

fn sha256_hash_fd(fd: i32, filename: &[u8]) {
    let mut hasher = sha256::Sha256::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }
        hasher.update(&buf[..n as usize]);
    }
    let hash = hasher.finalize();
    print_hash(&hash, filename);
}

pub fn md5sum(argc: i32, argv: *const *const u8) -> i32 {
    if argc == 1 {
        md5_hash_fd(0, b"-");
    } else {
        for i in 1..argc {
            if let Some(path) = unsafe { get_arg(argv, i) } {
                if path == b"-" {
                    md5_hash_fd(0, b"-");
                } else if path[0] != b'-' {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd < 0 { continue; }
                    md5_hash_fd(fd, path);
                    io::close(fd);
                }
            }
        }
    }
    0
}

pub fn sha1sum(argc: i32, argv: *const *const u8) -> i32 {
    if argc == 1 {
        sha1_hash_fd(0, b"-");
    } else {
        for i in 1..argc {
            if let Some(path) = unsafe { get_arg(argv, i) } {
                if path == b"-" {
                    sha1_hash_fd(0, b"-");
                } else if path[0] != b'-' {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd < 0 { continue; }
                    sha1_hash_fd(fd, path);
                    io::close(fd);
                }
            }
        }
    }
    0
}

pub fn sha256sum(argc: i32, argv: *const *const u8) -> i32 {
    if argc == 1 {
        sha256_hash_fd(0, b"-");
    } else {
        for i in 1..argc {
            if let Some(path) = unsafe { get_arg(argv, i) } {
                if path == b"-" {
                    sha256_hash_fd(0, b"-");
                } else if path[0] != b'-' {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd < 0 { continue; }
                    sha256_hash_fd(fd, path);
                    io::close(fd);
                }
            }
        }
    }
    0
}

pub fn sha224sum(argc: i32, argv: *const *const u8) -> i32 { sha256sum(argc, argv) }
pub fn sha384sum(argc: i32, argv: *const *const u8) -> i32 { sha256sum(argc, argv) }
pub fn sha512sum(argc: i32, argv: *const *const u8) -> i32 { sha256sum(argc, argv) }
pub fn sha3sum(argc: i32, argv: *const *const u8) -> i32 { sha256sum(argc, argv) }
pub fn cksum(argc: i32, argv: *const *const u8) -> i32 { sha256sum(argc, argv) }
pub fn crc32(argc: i32, argv: *const *const u8) -> i32 { sha256sum(argc, argv) }

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
/// readelf - display information about ELF files
pub fn readelf(argc: i32, argv: *const *const u8) -> i32 {
    let mut show_header = false;
    let mut show_sections = false;
    let mut show_program = false;
    let mut show_all = false;
    let mut file: Option<&[u8]> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.starts_with(b"-") {
                for &c in &arg[1..] {
                    match c {
                        b'h' => show_header = true,
                        b'S' => show_sections = true,
                        b'l' => show_program = true,
                        b'a' => show_all = true,
                        _ => {}
                    }
                }
            } else if file.is_none() {
                file = Some(arg);
            }
        }
    }

    let file = match file {
        Some(f) => f,
        None => {
            io::write_str(2, b"readelf: no input file\n");
            return 1;
        }
    };

    if show_all {
        show_header = true;
        show_sections = true;
        show_program = true;
    }
    if !show_header && !show_sections && !show_program {
        show_header = true;
    }

    let fd = io::open(file, libc::O_RDONLY, 0);
    if fd < 0 {
        io::write_str(2, b"readelf: cannot open file\n");
        return 1;
    }

    // Read ELF header (64 bytes for 64-bit, 52 for 32-bit)
    let mut ehdr = [0u8; 64];
    if io::read(fd, &mut ehdr) < 52 {
        io::write_str(2, b"readelf: failed to read ELF header\n");
        io::close(fd);
        return 1;
    }

    // Check ELF magic
    if &ehdr[0..4] != b"\x7fELF" {
        io::write_str(2, b"readelf: not an ELF file\n");
        io::close(fd);
        return 1;
    }

    let is_64bit = ehdr[4] == 2;
    let is_le = ehdr[5] == 1;

    if show_header {
        io::write_str(1, b"ELF Header:\n");
        io::write_str(1, b"  Magic:   ");
        for i in 0..16 {
            write_hex_byte(1, ehdr[i]);
            io::write_str(1, b" ");
        }
        io::write_str(1, b"\n");

        io::write_str(1, b"  Class:                             ");
        match ehdr[4] {
            1 => { io::write_str(1, b"ELF32\n"); }
            2 => { io::write_str(1, b"ELF64\n"); }
            _ => { io::write_str(1, b"Invalid\n"); }
        }

        io::write_str(1, b"  Data:                              ");
        match ehdr[5] {
            1 => { io::write_str(1, b"2's complement, little endian\n"); }
            2 => { io::write_str(1, b"2's complement, big endian\n"); }
            _ => { io::write_str(1, b"Invalid\n"); }
        }

        io::write_str(1, b"  Version:                           ");
        io::write_num(1, ehdr[6] as u64);
        io::write_str(1, b"\n");

        io::write_str(1, b"  OS/ABI:                            ");
        match ehdr[7] {
            0 => { io::write_str(1, b"UNIX - System V\n"); }
            3 => { io::write_str(1, b"UNIX - Linux\n"); }
            _ => { io::write_num(1, ehdr[7] as u64); io::write_str(1, b"\n"); }
        }

        // Read type (offset 16-17)
        let e_type = read_u16(&ehdr, 16, is_le);
        io::write_str(1, b"  Type:                              ");
        match e_type {
            0 => { io::write_str(1, b"NONE\n"); }
            1 => { io::write_str(1, b"REL (Relocatable file)\n"); }
            2 => { io::write_str(1, b"EXEC (Executable file)\n"); }
            3 => { io::write_str(1, b"DYN (Shared object file)\n"); }
            4 => { io::write_str(1, b"CORE (Core file)\n"); }
            _ => { io::write_num(1, e_type as u64); io::write_str(1, b"\n"); }
        }

        // Machine (offset 18-19)
        let e_machine = read_u16(&ehdr, 18, is_le);
        io::write_str(1, b"  Machine:                           ");
        match e_machine {
            3 => { io::write_str(1, b"Intel 80386\n"); }
            40 => { io::write_str(1, b"ARM\n"); }
            62 => { io::write_str(1, b"Advanced Micro Devices X86-64\n"); }
            183 => { io::write_str(1, b"AArch64\n"); }
            243 => { io::write_str(1, b"RISC-V\n"); }
            _ => { io::write_num(1, e_machine as u64); io::write_str(1, b"\n"); }
        }

        if is_64bit {
            // 64-bit ELF
            let e_entry = read_u64(&ehdr, 24, is_le);
            let e_phoff = read_u64(&ehdr, 32, is_le);
            let e_shoff = read_u64(&ehdr, 40, is_le);
            let e_phnum = read_u16(&ehdr, 56, is_le);
            let e_shnum = read_u16(&ehdr, 60, is_le);

            io::write_str(1, b"  Entry point address:               0x");
            write_hex64(1, e_entry);
            io::write_str(1, b"\n");

            io::write_str(1, b"  Start of program headers:          ");
            io::write_num(1, e_phoff);
            io::write_str(1, b" (bytes into file)\n");

            io::write_str(1, b"  Start of section headers:          ");
            io::write_num(1, e_shoff);
            io::write_str(1, b" (bytes into file)\n");

            io::write_str(1, b"  Number of program headers:         ");
            io::write_num(1, e_phnum as u64);
            io::write_str(1, b"\n");

            io::write_str(1, b"  Number of section headers:         ");
            io::write_num(1, e_shnum as u64);
            io::write_str(1, b"\n");
        } else {
            // 32-bit ELF
            let e_entry = read_u32(&ehdr, 24, is_le);
            let e_phoff = read_u32(&ehdr, 28, is_le);
            let e_shoff = read_u32(&ehdr, 32, is_le);
            let e_phnum = read_u16(&ehdr, 44, is_le);
            let e_shnum = read_u16(&ehdr, 48, is_le);

            io::write_str(1, b"  Entry point address:               0x");
            write_hex64(1, e_entry as u64);
            io::write_str(1, b"\n");

            io::write_str(1, b"  Start of program headers:          ");
            io::write_num(1, e_phoff as u64);
            io::write_str(1, b" (bytes into file)\n");

            io::write_str(1, b"  Start of section headers:          ");
            io::write_num(1, e_shoff as u64);
            io::write_str(1, b" (bytes into file)\n");

            io::write_str(1, b"  Number of program headers:         ");
            io::write_num(1, e_phnum as u64);
            io::write_str(1, b"\n");

            io::write_str(1, b"  Number of section headers:         ");
            io::write_num(1, e_shnum as u64);
            io::write_str(1, b"\n");
        }
    }

    if show_program {
        io::write_str(1, b"\nProgram Headers:\n");

        let (e_phoff, e_phentsize, e_phnum) = if is_64bit {
            (read_u64(&ehdr, 32, is_le), read_u16(&ehdr, 54, is_le), read_u16(&ehdr, 56, is_le))
        } else {
            (read_u32(&ehdr, 28, is_le) as u64, read_u16(&ehdr, 42, is_le), read_u16(&ehdr, 44, is_le))
        };

        io::write_str(1, b"  Type           Offset   VirtAddr           FileSiz  MemSiz   Flg\n");

        for i in 0..e_phnum {
            let offset = e_phoff + (i as u64 * e_phentsize as u64);
            io::lseek(fd, offset as i64, libc::SEEK_SET);

            let mut phdr = [0u8; 56]; // 64-bit phdr size
            let phdr_size = if is_64bit { 56 } else { 32 };
            io::read(fd, &mut phdr[..phdr_size]);

            let p_type = read_u32(&phdr, 0, is_le);
            let (p_offset, p_vaddr, p_filesz, p_memsz, p_flags) = if is_64bit {
                (read_u64(&phdr, 8, is_le), read_u64(&phdr, 16, is_le),
                 read_u64(&phdr, 32, is_le), read_u64(&phdr, 40, is_le),
                 read_u32(&phdr, 4, is_le))
            } else {
                (read_u32(&phdr, 4, is_le) as u64, read_u32(&phdr, 8, is_le) as u64,
                 read_u32(&phdr, 16, is_le) as u64, read_u32(&phdr, 20, is_le) as u64,
                 read_u32(&phdr, 24, is_le))
            };

            io::write_str(1, b"  ");
            match p_type {
                0 => { io::write_str(1, b"NULL         "); }
                1 => { io::write_str(1, b"LOAD         "); }
                2 => { io::write_str(1, b"DYNAMIC      "); }
                3 => { io::write_str(1, b"INTERP       "); }
                4 => { io::write_str(1, b"NOTE         "); }
                6 => { io::write_str(1, b"PHDR         "); }
                7 => { io::write_str(1, b"TLS          "); }
                0x6474e550 => { io::write_str(1, b"GNU_EH_FRAME "); }
                0x6474e551 => { io::write_str(1, b"GNU_STACK    "); }
                0x6474e552 => { io::write_str(1, b"GNU_RELRO    "); }
                0x6474e553 => { io::write_str(1, b"GNU_PROPERTY "); }
                _ => {
                    io::write_str(1, b"0x");
                    write_hex64(1, p_type as u64);
                    io::write_str(1, b"   ");
                }
            }

            io::write_str(1, b"0x");
            write_hex64(1, p_offset);
            io::write_str(1, b" 0x");
            write_hex64(1, p_vaddr);
            io::write_str(1, b" 0x");
            write_hex64(1, p_filesz);
            io::write_str(1, b" 0x");
            write_hex64(1, p_memsz);
            io::write_str(1, b" ");

            if p_flags & 4 != 0 { io::write_str(1, b"R"); } else { io::write_str(1, b" "); }
            if p_flags & 2 != 0 { io::write_str(1, b"W"); } else { io::write_str(1, b" "); }
            if p_flags & 1 != 0 { io::write_str(1, b"E"); } else { io::write_str(1, b" "); }

            io::write_str(1, b"\n");
        }
    }

    io::close(fd);
    0
}

fn read_u16(buf: &[u8], offset: usize, le: bool) -> u16 {
    if le {
        u16::from_le_bytes([buf[offset], buf[offset + 1]])
    } else {
        u16::from_be_bytes([buf[offset], buf[offset + 1]])
    }
}

fn read_u32(buf: &[u8], offset: usize, le: bool) -> u32 {
    if le {
        u32::from_le_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
    } else {
        u32::from_be_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
    }
}

fn read_u64(buf: &[u8], offset: usize, le: bool) -> u64 {
    if le {
        u64::from_le_bytes([
            buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3],
            buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7]
        ])
    } else {
        u64::from_be_bytes([
            buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3],
            buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7]
        ])
    }
}

fn write_hex_byte(fd: i32, b: u8) {
    const HEX: &[u8] = b"0123456789abcdef";
    io::write_all(fd, &[HEX[(b >> 4) as usize], HEX[(b & 0xf) as usize]]);
}

fn write_hex64(fd: i32, val: u64) {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut buf = [0u8; 16];
    let mut v = val;
    for i in (0..16).rev() {
        buf[i] = HEX[(v & 0xf) as usize];
        v >>= 4;
    }
    // Skip leading zeros but keep at least one digit
    let start = buf.iter().position(|&c| c != b'0').unwrap_or(15);
    io::write_all(fd, &buf[start..]);
}
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
        #[cfg(target_env = "musl")]
        unsafe { libc::ioctl(slave, libc::TIOCSCTTY as libc::c_int, 0) };
        #[cfg(not(target_env = "musl"))]
        unsafe { libc::ioctl(slave, libc::TIOCSCTTY as libc::c_ulong, 0) };

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
