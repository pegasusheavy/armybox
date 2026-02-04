//! Text processing applets
//!
//! POSIX.1-2017 compliant text manipulation utilities.

use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

// Individual utility modules
mod cut;
mod echo;
mod grep;
mod head;
mod nl;
mod paste;
mod printf;
mod rev;
mod sed;
mod seq;
mod sort;
mod tac;
mod tail;
mod tee;
mod tr;
mod uniq;
mod wc;
mod yes;

// Re-export utilities
pub use cut::cut;
pub use echo::echo;
pub use grep::{grep, egrep, fgrep};
pub use head::head;
pub use nl::nl;
pub use paste::paste;
pub use printf::printf;
pub use rev::rev;
pub use sed::sed;
pub use seq::seq;
pub use sort::sort;
pub use tac::tac;
pub use tail::tail;
pub use tee::tee;
pub use tr::tr;
pub use uniq::uniq;
pub use wc::wc;
pub use yes::yes;

/// awk - pattern scanning and processing
pub fn awk(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"awk: missing program\n");
        return 1;
    }

    let program = unsafe { get_arg(argv, 1).unwrap() };

    // Very simple awk - just {print $N}
    let mut print_field: Option<usize> = None;
    let mut print_all = false;

    if program == b"{print}" || program == b"{print $0}" {
        print_all = true;
    } else if program.starts_with(b"{print $") {
        let end = program.iter().position(|&c| c == b'}').unwrap_or(program.len());
        if let Some(n) = sys::parse_u64(&program[8..end]) {
            print_field = Some(n as usize);
        }
    }

    let mut buf = [0u8; 4096];
    let mut line = [0u8; 4096];
    let mut line_len = 0;

    loop {
        let n = io::read(0, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                if print_all {
                    io::write_all(1, &line[..line_len]);
                    io::write_str(1, b"\n");
                } else if let Some(field) = print_field {
                    // Split by whitespace
                    let mut field_num = 0;
                    let mut start = 0;
                    let mut in_field = false;

                    for i in 0..=line_len {
                        let is_space = i == line_len || line[i] == b' ' || line[i] == b'\t';

                        if !in_field && !is_space {
                            in_field = true;
                            field_num += 1;
                            start = i;
                        } else if in_field && is_space {
                            if field_num == field {
                                io::write_all(1, &line[start..i]);
                                io::write_str(1, b"\n");
                                break;
                            }
                            in_field = false;
                        }
                    }
                }
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = c;
                line_len += 1;
            }
        }
    }
    0
}

/// comm - compare two sorted files line by line
pub fn comm(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut suppress_col1 = false;
        let mut suppress_col2 = false;
        let mut suppress_col3 = false;
        let mut file1: Option<&[u8]> = None;
        let mut file2: Option<&[u8]> = None;

        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg.starts_with(b"-") && arg.len() > 1 && arg[1] != b'-' {
                    for &c in &arg[1..] {
                        match c {
                            b'1' => suppress_col1 = true,
                            b'2' => suppress_col2 = true,
                            b'3' => suppress_col3 = true,
                            _ => {}
                        }
                    }
                } else if file1.is_none() {
                    file1 = Some(arg);
                } else if file2.is_none() {
                    file2 = Some(arg);
                }
            }
        }

        let file1 = match file1 {
            Some(f) => f,
            None => {
                io::write_str(2, b"comm: missing operand\n");
                return 1;
            }
        };
        let file2 = match file2 {
            Some(f) => f,
            None => {
                io::write_str(2, b"comm: missing operand\n");
                return 1;
            }
        };

        // Read both files
        let fd1 = if file1 == b"-" { 0 } else { io::open(file1, libc::O_RDONLY, 0) };
        if fd1 < 0 && file1 != b"-" {
            io::write_str(2, b"comm: cannot open file1\n");
            return 1;
        }
        let content1 = io::read_all(fd1);
        if fd1 > 0 { io::close(fd1); }

        let fd2 = if file2 == b"-" { 0 } else { io::open(file2, libc::O_RDONLY, 0) };
        if fd2 < 0 && file2 != b"-" {
            io::write_str(2, b"comm: cannot open file2\n");
            return 1;
        }
        let content2 = io::read_all(fd2);
        if fd2 > 0 { io::close(fd2); }

        let lines1: Vec<&[u8]> = content1.split(|&c| c == b'\n').filter(|l| !l.is_empty()).collect();
        let lines2: Vec<&[u8]> = content2.split(|&c| c == b'\n').filter(|l| !l.is_empty()).collect();

        let mut i = 0;
        let mut j = 0;

        while i < lines1.len() || j < lines2.len() {
            if i >= lines1.len() {
                // Only file2 has remaining lines
                if !suppress_col2 {
                    if !suppress_col1 { io::write_str(1, b"\t"); }
                    io::write_all(1, lines2[j]);
                    io::write_str(1, b"\n");
                }
                j += 1;
            } else if j >= lines2.len() {
                // Only file1 has remaining lines
                if !suppress_col1 {
                    io::write_all(1, lines1[i]);
                    io::write_str(1, b"\n");
                }
                i += 1;
            } else {
                let cmp = cmp_bytes(lines1[i], lines2[j]);
                if cmp < 0 {
                    // Line only in file1
                    if !suppress_col1 {
                        io::write_all(1, lines1[i]);
                        io::write_str(1, b"\n");
                    }
                    i += 1;
                } else if cmp > 0 {
                    // Line only in file2
                    if !suppress_col2 {
                        if !suppress_col1 { io::write_str(1, b"\t"); }
                        io::write_all(1, lines2[j]);
                        io::write_str(1, b"\n");
                    }
                    j += 1;
                } else {
                    // Line in both files
                    if !suppress_col3 {
                        if !suppress_col1 { io::write_str(1, b"\t"); }
                        if !suppress_col2 { io::write_str(1, b"\t"); }
                        io::write_all(1, lines1[i]);
                        io::write_str(1, b"\n");
                    }
                    i += 1;
                    j += 1;
                }
            }
        }
    }
    0
}

fn cmp_bytes(a: &[u8], b: &[u8]) -> i32 {
    let min_len = a.len().min(b.len());
    for i in 0..min_len {
        if a[i] < b[i] { return -1; }
        if a[i] > b[i] { return 1; }
    }
    if a.len() < b.len() { -1 }
    else if a.len() > b.len() { 1 }
    else { 0 }
}

/// expand - convert tabs to spaces
pub fn expand(argc: i32, argv: *const *const u8) -> i32 {
    let fd = if argc > 1 {
        if let Some(path) = unsafe { get_arg(argv, argc - 1) } {
            if path.len() > 0 && path[0] != b'-' {
                io::open(path, libc::O_RDONLY, 0)
            } else { 0 }
        } else { 0 }
    } else { 0 };

    let mut buf = [0u8; 4096];
    let mut col = 0;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\t' {
                let spaces = 8 - (col % 8);
                for _ in 0..spaces {
                    io::write_str(1, b" ");
                }
                col += spaces;
            } else if c == b'\n' {
                io::write_str(1, b"\n");
                col = 0;
            } else {
                io::write_all(1, &[c]);
                col += 1;
            }
        }
    }

    if fd != 0 { io::close(fd); }
    0
}

/// unexpand - convert spaces to tabs
pub fn unexpand(argc: i32, argv: *const *const u8) -> i32 {
    let fd = if argc > 1 {
        if let Some(path) = unsafe { get_arg(argv, argc - 1) } {
            if path.len() > 0 && path[0] != b'-' {
                io::open(path, libc::O_RDONLY, 0)
            } else { 0 }
        } else { 0 }
    } else { 0 };

    let mut buf = [0u8; 4096];
    let mut spaces = 0;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b' ' {
                spaces += 1;
                if spaces == 8 {
                    io::write_str(1, b"\t");
                    spaces = 0;
                }
            } else {
                for _ in 0..spaces {
                    io::write_str(1, b" ");
                }
                spaces = 0;
                io::write_all(1, &[c]);
            }
        }
    }

    if fd != 0 { io::close(fd); }
    0
}

/// fold - wrap lines to specified width
pub fn fold(argc: i32, argv: *const *const u8) -> i32 {
    let mut width = 80usize;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'w') && i + 1 < argc {
                if let Some(w) = unsafe { get_arg(argv, i + 1) } {
                    width = sys::parse_u64(w).unwrap_or(80) as usize;
                }
            }
        }
    }

    let mut buf = [0u8; 4096];
    let mut col = 0;

    loop {
        let n = io::read(0, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                io::write_str(1, b"\n");
                col = 0;
            } else {
                if col >= width {
                    io::write_str(1, b"\n");
                    col = 0;
                }
                io::write_all(1, &[c]);
                col += 1;
            }
        }
    }
    0
}

/// fmt - simple text formatter
pub fn fmt(argc: i32, argv: *const *const u8) -> i32 {
    fold(argc, argv)
}

/// strings - print printable strings from binary
pub fn strings(argc: i32, argv: *const *const u8) -> i32 {
    let min_len = 4;

    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path.len() > 0 && path[0] != b'-' {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd < 0 { continue; }

                let mut buf = [0u8; 4096];
                let mut string = [0u8; 256];
                let mut string_len = 0;

                loop {
                    let n = io::read(fd, &mut buf);
                    if n <= 0 { break; }

                    for &c in &buf[..n as usize] {
                        if c >= 0x20 && c < 0x7f {
                            if string_len < string.len() {
                                string[string_len] = c;
                                string_len += 1;
                            }
                        } else {
                            if string_len >= min_len {
                                io::write_all(1, &string[..string_len]);
                                io::write_str(1, b"\n");
                            }
                            string_len = 0;
                        }
                    }
                }

                io::close(fd);
            }
        }
    }
    0
}

/// dos2unix - convert line endings
pub fn dos2unix(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path.len() > 0 && path[0] != b'-' {
                #[cfg(feature = "alloc")]
                {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd < 0 { continue; }

                    let content = io::read_all(fd);
                    io::close(fd);

                    let fd = io::open(path, libc::O_WRONLY | libc::O_TRUNC, 0);
                    if fd < 0 { continue; }

                    for &c in &content {
                        if c != b'\r' {
                            io::write_all(fd, &[c]);
                        }
                    }
                    io::close(fd);
                }
            }
        }
    }
    0
}

/// unix2dos - convert line endings
pub fn unix2dos(argc: i32, argv: *const *const u8) -> i32 {
    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path.len() > 0 && path[0] != b'-' {
                #[cfg(feature = "alloc")]
                {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd < 0 { continue; }

                    let content = io::read_all(fd);
                    io::close(fd);

                    let fd = io::open(path, libc::O_WRONLY | libc::O_TRUNC, 0);
                    if fd < 0 { continue; }

                    for &c in &content {
                        if c == b'\n' {
                            io::write_str(fd, b"\r\n");
                        } else if c != b'\r' {
                            io::write_all(fd, &[c]);
                        }
                    }
                    io::close(fd);
                }
            }
        }
    }
    0
}
