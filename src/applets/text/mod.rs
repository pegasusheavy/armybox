//! Text processing applets

use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

/// echo - print arguments
pub fn echo(argc: i32, argv: *const *const u8) -> i32 {
    let mut newline = true;
    let mut start = 1;

    if argc > 1 {
        if let Some(arg) = unsafe { get_arg(argv, 1) } {
            if arg == b"-n" {
                newline = false;
                start = 2;
            }
        }
    }

    for i in start..argc {
        if i > start {
            io::write_str(1, b" ");
        }
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            io::write_all(1, arg);
        }
    }

    if newline {
        io::write_str(1, b"\n");
    }
    0
}

/// printf - format and print data
pub fn printf(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        return 0;
    }

    let fmt = unsafe { get_arg(argv, 1).unwrap() };
    let mut arg_idx = 2;
    let mut i = 0;

    while i < fmt.len() {
        if fmt[i] == b'%' && i + 1 < fmt.len() {
            i += 1;
            match fmt[i] {
                b's' => {
                    if arg_idx < argc {
                        if let Some(arg) = unsafe { get_arg(argv, arg_idx) } {
                            io::write_all(1, arg);
                            arg_idx += 1;
                        }
                    }
                }
                b'd' | b'i' => {
                    if arg_idx < argc {
                        if let Some(arg) = unsafe { get_arg(argv, arg_idx) } {
                            if let Some(n) = sys::parse_i64(arg) {
                                io::write_signed(1, n);
                            }
                            arg_idx += 1;
                        }
                    }
                }
                b'x' => {
                    if arg_idx < argc {
                        if let Some(arg) = unsafe { get_arg(argv, arg_idx) } {
                            if let Some(n) = sys::parse_u64(arg) {
                                let mut buf = [0u8; 20];
                                let s = sys::format_hex(n, &mut buf);
                                io::write_all(1, s);
                            }
                            arg_idx += 1;
                        }
                    }
                }
                b'%' => { io::write_str(1, b"%"); }
                b'n' => { io::write_str(1, b"\n"); }
                _ => {
                    io::write_str(1, b"%");
                    io::write_all(1, &[fmt[i]]);
                }
            }
        } else if fmt[i] == b'\\' && i + 1 < fmt.len() {
            i += 1;
            match fmt[i] {
                b'n' => { io::write_str(1, b"\n"); }
                b't' => { io::write_str(1, b"\t"); }
                b'r' => { io::write_str(1, b"\r"); }
                b'\\' => { io::write_str(1, b"\\"); }
                _ => { io::write_all(1, &[fmt[i]]); }
            }
        } else {
            io::write_all(1, &[fmt[i]]);
        }
        i += 1;
    }
    0
}

/// head - output first part of files
pub fn head(argc: i32, argv: *const *const u8) -> i32 {
    let mut lines = 10i64;
    let mut files_start = 1;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'n') && i + 1 < argc {
                if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                    lines = sys::parse_i64(n).unwrap_or(10);
                }
                files_start = i + 2;
            } else if arg[0] == b'-' && arg.len() > 1 && arg[1] >= b'0' && arg[1] <= b'9' {
                lines = sys::parse_i64(&arg[1..]).unwrap_or(10);
                files_start = i + 1;
            }
        }
    }

    if files_start >= argc {
        head_fd(0, lines);
    } else {
        for i in files_start..argc {
            if let Some(path) = unsafe { get_arg(argv, i) } {
                if path == b"-" {
                    head_fd(0, lines);
                } else {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd >= 0 {
                        head_fd(fd, lines);
                        io::close(fd);
                    }
                }
            }
        }
    }
    0
}

fn head_fd(fd: i32, mut lines: i64) {
    let mut buf = [0u8; 4096];
    while lines > 0 {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for i in 0..n as usize {
            io::write_all(1, &buf[i..i+1]);
            if buf[i] == b'\n' {
                lines -= 1;
                if lines <= 0 { return; }
            }
        }
    }
}

/// tail - output last part of files
pub fn tail(argc: i32, argv: *const *const u8) -> i32 {
    let mut lines = 10usize;
    let mut follow = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'n') && i + 1 < argc {
                if let Some(n) = unsafe { get_arg(argv, i + 1) } {
                    lines = sys::parse_u64(n).unwrap_or(10) as usize;
                }
            } else if has_opt(arg, b'f') {
                follow = true;
            } else if arg[0] != b'-' {
                let fd = io::open(arg, libc::O_RDONLY, 0);
                if fd >= 0 {
                    tail_fd(fd, lines);
                    if follow {
                        loop {
                            let mut buf = [0u8; 4096];
                            let n = io::read(fd, &mut buf);
                            if n > 0 {
                                io::write_all(1, &buf[..n as usize]);
                            } else {
                                unsafe { libc::usleep(100000) };
                            }
                        }
                    }
                    io::close(fd);
                }
            }
        }
    }
    0
}

fn tail_fd(fd: i32, lines: usize) {
    #[cfg(feature = "alloc")]
    {
        use alloc::collections::VecDeque;

        let content = io::read_all(fd);
        if content.is_empty() {
            return;
        }

        // Collect line start positions
        let mut line_starts: VecDeque<usize> = VecDeque::new();
        line_starts.push_back(0);

        for (i, &c) in content.iter().enumerate() {
            if c == b'\n' && i + 1 < content.len() {
                line_starts.push_back(i + 1);
            }
        }

        // Keep only the last N lines
        while line_starts.len() > lines {
            line_starts.pop_front();
        }

        if let Some(&start) = line_starts.front() {
            io::write_all(1, &content[start..]);
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        let _ = fd;
        let _ = lines;
        io::write_str(2, b"tail: requires alloc feature\n");
    }
}

/// wc - word, line, character count
pub fn wc(argc: i32, argv: *const *const u8) -> i32 {
    let mut show_lines = false;
    let mut show_words = false;
    let mut show_chars = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b'l') { show_lines = true; }
                if has_opt(arg, b'w') { show_words = true; }
                if has_opt(arg, b'c') { show_chars = true; }
            }
        }
    }

    if !show_lines && !show_words && !show_chars {
        show_lines = true;
        show_words = true;
        show_chars = true;
    }

    let mut total_lines = 0u64;
    let mut total_words = 0u64;
    let mut total_chars = 0u64;
    let mut file_count = 0;

    for i in 1..argc {
        if let Some(path) = unsafe { get_arg(argv, i) } {
            if path[0] != b'-' {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd >= 0 {
                    let (l, w, c) = wc_fd(fd);
                    total_lines += l;
                    total_words += w;
                    total_chars += c;

                    if show_lines { io::write_num(1, l); io::write_str(1, b" "); }
                    if show_words { io::write_num(1, w); io::write_str(1, b" "); }
                    if show_chars { io::write_num(1, c); io::write_str(1, b" "); }
                    io::write_all(1, path);
                    io::write_str(1, b"\n");

                    io::close(fd);
                    file_count += 1;
                }
            }
        }
    }

    if file_count == 0 {
        let (l, w, c) = wc_fd(0);
        if show_lines { io::write_num(1, l); io::write_str(1, b" "); }
        if show_words { io::write_num(1, w); io::write_str(1, b" "); }
        if show_chars { io::write_num(1, c); }
        io::write_str(1, b"\n");
    } else if file_count > 1 {
        if show_lines { io::write_num(1, total_lines); io::write_str(1, b" "); }
        if show_words { io::write_num(1, total_words); io::write_str(1, b" "); }
        if show_chars { io::write_num(1, total_chars); io::write_str(1, b" "); }
        io::write_str(1, b"total\n");
    }
    0
}

fn wc_fd(fd: i32) -> (u64, u64, u64) {
    let mut lines = 0u64;
    let mut words = 0u64;
    let mut chars = 0u64;
    let mut in_word = false;

    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            chars += 1;
            if c == b'\n' { lines += 1; }

            let is_space = c == b' ' || c == b'\n' || c == b'\t' || c == b'\r';
            if is_space {
                in_word = false;
            } else if !in_word {
                in_word = true;
                words += 1;
            }
        }
    }

    (lines, words, chars)
}

/// tee - read from stdin and write to stdout and files
pub fn tee(argc: i32, argv: *const *const u8) -> i32 {
    let mut append = false;

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut fds: Vec<i32> = Vec::new();
        fds.push(1); // stdout

        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if has_opt(arg, b'a') {
                    append = true;
                } else if arg[0] != b'-' {
                    let flags = if append {
                        libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND
                    } else {
                        libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC
                    };
                    let fd = io::open(arg, flags, 0o644);
                    if fd >= 0 {
                        fds.push(fd);
                    }
                }
            }
        }

        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 { break; }

            for &fd in &fds {
                io::write_all(fd, &buf[..n as usize]);
            }
        }

        for &fd in &fds[1..] {
            io::close(fd);
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        let _ = append;
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 { break; }
            io::write_all(1, &buf[..n as usize]);
        }
    }
    0
}

/// tac - concatenate files in reverse
pub fn tac(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        for i in 1..argc {
            if let Some(path) = unsafe { get_arg(argv, i) } {
                if path[0] != b'-' {
                    let fd = io::open(path, libc::O_RDONLY, 0);
                    if fd >= 0 {
                        let content = io::read_all(fd);
                        io::close(fd);

                        let lines: Vec<&[u8]> = content.split(|&c| c == b'\n').collect();
                        for line in lines.iter().rev() {
                            if !line.is_empty() {
                                io::write_all(1, line);
                                io::write_str(1, b"\n");
                            }
                        }
                    }
                }
            }
        }

        if argc < 2 {
            let content = io::read_all(0);
            let lines: Vec<&[u8]> = content.split(|&c| c == b'\n').collect();
            for line in lines.iter().rev() {
                if !line.is_empty() {
                    io::write_all(1, line);
                    io::write_str(1, b"\n");
                }
            }
        }
    }
    0
}

/// rev - reverse lines character-wise
pub fn rev(argc: i32, argv: *const *const u8) -> i32 {
    let fd = if argc > 1 {
        if let Some(path) = unsafe { get_arg(argv, 1) } {
            if path[0] != b'-' {
                io::open(path, libc::O_RDONLY, 0)
            } else { 0 }
        } else { 0 }
    } else { 0 };

    let mut buf = [0u8; 4096];
    let mut line = [0u8; 1024];
    let mut line_len = 0;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                // Reverse and print
                for i in (0..line_len).rev() {
                    io::write_all(1, &line[i..i+1]);
                }
                io::write_str(1, b"\n");
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = c;
                line_len += 1;
            }
        }
    }

    // Print remaining
    if line_len > 0 {
        for i in (0..line_len).rev() {
            io::write_all(1, &line[i..i+1]);
        }
    }

    if fd != 0 { io::close(fd); }
    0
}

/// yes - output a string repeatedly
pub fn yes(argc: i32, argv: *const *const u8) -> i32 {
    let text = if argc > 1 {
        unsafe { get_arg(argv, 1).unwrap_or(b"y") }
    } else {
        b"y"
    };

    loop {
        io::write_all(1, text);
        io::write_str(1, b"\n");
    }
}

/// seq - print sequence of numbers
pub fn seq(argc: i32, argv: *const *const u8) -> i32 {
    let (first, last, incr) = match argc {
        2 => {
            let last = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(1);
            (1i64, last, 1i64)
        }
        3 => {
            let first = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(1);
            let last = sys::parse_i64(unsafe { get_arg(argv, 2).unwrap() }).unwrap_or(1);
            (first, last, 1)
        }
        _ if argc >= 4 => {
            let first = sys::parse_i64(unsafe { get_arg(argv, 1).unwrap() }).unwrap_or(1);
            let incr = sys::parse_i64(unsafe { get_arg(argv, 2).unwrap() }).unwrap_or(1);
            let last = sys::parse_i64(unsafe { get_arg(argv, 3).unwrap() }).unwrap_or(1);
            (first, last, incr)
        }
        _ => (1, 10, 1),
    };

    let mut n = first;
    if incr > 0 {
        while n <= last {
            io::write_signed(1, n);
            io::write_str(1, b"\n");
            n += incr;
        }
    } else if incr < 0 {
        while n >= last {
            io::write_signed(1, n);
            io::write_str(1, b"\n");
            n += incr;
        }
    }
    0
}

/// nl - number lines
pub fn nl(argc: i32, argv: *const *const u8) -> i32 {
    let fd = if argc > 1 {
        if let Some(path) = unsafe { get_arg(argv, argc - 1) } {
            if path[0] != b'-' {
                io::open(path, libc::O_RDONLY, 0)
            } else { 0 }
        } else { 0 }
    } else { 0 };

    let mut line_num = 1u64;
    let mut buf = [0u8; 4096];
    let mut at_line_start = true;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if at_line_start {
                io::write_num(1, line_num);
                io::write_str(1, b"\t");
                at_line_start = false;
            }
            io::write_all(1, &[c]);
            if c == b'\n' {
                line_num += 1;
                at_line_start = true;
            }
        }
    }

    if fd != 0 { io::close(fd); }
    0
}

/// tr - translate characters
pub fn tr(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        let mut delete = false;
        let mut squeeze = false;
        let mut complement = false;
        let mut set1_idx = 0;
        let mut set2_idx = 0;

        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg[0] == b'-' {
                    if has_opt(arg, b'd') { delete = true; }
                    if has_opt(arg, b's') { squeeze = true; }
                    if has_opt(arg, b'c') || has_opt(arg, b'C') { complement = true; }
                } else if set1_idx == 0 {
                    set1_idx = i;
                } else if set2_idx == 0 {
                    set2_idx = i;
                }
            }
        }

        if set1_idx == 0 {
            io::write_str(2, b"tr: missing operand\n");
            return 1;
        }

        let set1 = unsafe { get_arg(argv, set1_idx).unwrap() };
        let set2 = if set2_idx > 0 { unsafe { get_arg(argv, set2_idx) } } else { None };

        let mut map = [0u8; 256];
        for i in 0..256 { map[i] = i as u8; }

        let set1_expanded = expand_set(set1);

        if delete {
            // Delete mode
            let mut buf = [0u8; 4096];
            let mut last_char: Option<u8> = None;

            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 { break; }

                for &c in &buf[..n as usize] {
                    let in_set = if complement {
                        !set1_expanded.contains(&c)
                    } else {
                        set1_expanded.contains(&c)
                    };

                    if !in_set {
                        if squeeze {
                            if Some(c) != last_char {
                                io::write_all(1, &[c]);
                                last_char = Some(c);
                            }
                        } else {
                            io::write_all(1, &[c]);
                        }
                    }
                }
            }
        } else if let Some(s2) = set2 {
            // Translate mode
            let set2_expanded = expand_set(s2);

            for (i, &c) in set1_expanded.iter().enumerate() {
                let replacement = if i < set2_expanded.len() {
                    set2_expanded[i]
                } else if !set2_expanded.is_empty() {
                    set2_expanded[set2_expanded.len() - 1]
                } else {
                    c
                };

                if complement {
                    for j in 0..256 {
                        if !set1_expanded.contains(&(j as u8)) {
                            map[j] = replacement;
                        }
                    }
                } else {
                    map[c as usize] = replacement;
                }
            }

            let mut buf = [0u8; 4096];
            let mut last_char: Option<u8> = None;

            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 { break; }

                for &c in &buf[..n as usize] {
                    let out = map[c as usize];
                    if squeeze && set2_expanded.contains(&out) {
                        if Some(out) != last_char {
                            io::write_all(1, &[out]);
                            last_char = Some(out);
                        }
                    } else {
                        io::write_all(1, &[out]);
                        last_char = Some(out);
                    }
                }
            }
        }
        return 0;
    }

    #[cfg(not(feature = "alloc"))]
    {
        let _ = argc;
        let _ = argv;
        io::write_str(2, b"tr: requires alloc feature\n");
        return 1;
    }
}

#[cfg(feature = "alloc")]
fn expand_set(s: &[u8]) -> alloc::vec::Vec<u8> {
    use alloc::vec::Vec;
    let mut result = Vec::new();
    let mut i = 0;

    while i < s.len() {
        // Check for range pattern: x-y
        if i + 2 < s.len() && s[i + 1] == b'-' {
            let start = s[i];
            let end = s[i + 2];
            if start <= end {
                for c in start..=end {
                    result.push(c);
                }
            } else {
                // Descending range
                for c in (end..=start).rev() {
                    result.push(c);
                }
            }
            i += 3;
        } else {
            result.push(s[i]);
            i += 1;
        }
    }
    result
}

#[cfg(not(feature = "alloc"))]
fn expand_set(s: &[u8]) -> &[u8] {
    // Without alloc, just return as-is (limited functionality)
    s
}

/// cut - remove sections from lines
pub fn cut(argc: i32, argv: *const *const u8) -> i32 {
    let mut delimiter = b'\t';
    let mut field: Option<usize> = None;
    let mut chars: Option<usize> = None;
    let mut i = 1;

    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.len() >= 2 && arg[0] == b'-' && arg[1] == b'd' {
                // Handle -dX (attached) or -d X (separate)
                if arg.len() > 2 {
                    delimiter = arg[2];
                } else if i + 1 < argc {
                    if let Some(d) = unsafe { get_arg(argv, i + 1) } {
                        if !d.is_empty() { delimiter = d[0]; }
                    }
                    i += 1;
                }
            } else if arg.len() >= 2 && arg[0] == b'-' && arg[1] == b'f' {
                // Handle -fN (attached) or -f N (separate)
                if arg.len() > 2 {
                    field = Some(sys::parse_u64(&arg[2..]).unwrap_or(1) as usize);
                } else if i + 1 < argc {
                    if let Some(f) = unsafe { get_arg(argv, i + 1) } {
                        field = Some(sys::parse_u64(f).unwrap_or(1) as usize);
                    }
                    i += 1;
                }
            } else if arg.len() >= 2 && arg[0] == b'-' && arg[1] == b'c' {
                // Handle -cN (attached) or -c N (separate)
                if arg.len() > 2 {
                    chars = Some(sys::parse_u64(&arg[2..]).unwrap_or(1) as usize);
                } else if i + 1 < argc {
                    if let Some(c) = unsafe { get_arg(argv, i + 1) } {
                        chars = Some(sys::parse_u64(c).unwrap_or(1) as usize);
                    }
                    i += 1;
                }
            }
        }
        i += 1;
    }

    let mut buf = [0u8; 4096];
    let mut line = [0u8; 4096];
    let mut line_len = 0;

    loop {
        let n = io::read(0, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                if let Some(f) = field {
                    let mut field_num = 1;
                    let mut start = 0;
                    let mut found = false;

                    for j in 0..line_len {
                        if line[j] == delimiter {
                            if field_num == f {
                                io::write_all(1, &line[start..j]);
                                found = true;
                                break;
                            }
                            field_num += 1;
                            start = j + 1;
                        }
                    }
                    if !found && field_num == f {
                        io::write_all(1, &line[start..line_len]);
                    }
                } else if let Some(c) = chars {
                    if c <= line_len {
                        io::write_all(1, &line[..c]);
                    } else {
                        io::write_all(1, &line[..line_len]);
                    }
                }
                io::write_str(1, b"\n");
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = c;
                line_len += 1;
            }
        }
    }
    0
}

/// paste - merge lines of files
/// paste - merge lines of files
pub fn paste(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut delimiter = b'\t';
        let mut serial = false;
        let mut files: Vec<&[u8]> = Vec::new();

        let mut i = 1;
        while i < argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg.starts_with(b"-") && arg.len() > 1 {
                    if arg == b"-s" {
                        serial = true;
                    } else if arg == b"-d" || arg.starts_with(b"-d") {
                        // Delimiter
                        if arg.len() > 2 {
                            delimiter = arg[2];
                        } else if i + 1 < argc {
                            i += 1;
                            if let Some(d) = unsafe { get_arg(argv, i) } {
                                if !d.is_empty() {
                                    delimiter = d[0];
                                }
                            }
                        }
                    }
                } else if arg == b"-" {
                    files.push(b"-");
                } else {
                    files.push(arg);
                }
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(b"-");
        }

        if serial {
            // Serial mode: output each file on a single line, fields delimited
            for &file in &files {
                let fd = if file == b"-" {
                    0
                } else {
                    io::open(file, libc::O_RDONLY, 0)
                };
                if fd < 0 && file != b"-" {
                    io::write_str(2, b"paste: cannot open file\n");
                    continue;
                }

                let content = io::read_all(fd);
                if fd > 0 { io::close(fd); }

                let mut first = true;
                for line in content.split(|&c| c == b'\n') {
                    if line.is_empty() { continue; }
                    if !first {
                        io::write_all(1, &[delimiter]);
                    }
                    io::write_all(1, line);
                    first = false;
                }
                io::write_str(1, b"\n");
            }
        } else {
            // Normal mode: merge corresponding lines from each file
            let mut file_data: Vec<Vec<u8>> = Vec::new();
            let mut fds: Vec<i32> = Vec::new();

            for &file in &files {
                let fd = if file == b"-" {
                    0
                } else {
                    io::open(file, libc::O_RDONLY, 0)
                };
                if fd < 0 && file != b"-" {
                    io::write_str(2, b"paste: cannot open file\n");
                    file_data.push(Vec::new());
                    fds.push(-1);
                } else {
                    let content = io::read_all(fd);
                    if fd > 0 { io::close(fd); }
                    file_data.push(content);
                    fds.push(0);
                }
            }

            // Convert to lines
            let file_lines: Vec<Vec<&[u8]>> = file_data.iter()
                .map(|d| d.split(|&c| c == b'\n').collect::<Vec<_>>())
                .collect();

            // Find max number of lines
            let max_lines = file_lines.iter().map(|l| l.len()).max().unwrap_or(0);

            for line_idx in 0..max_lines {
                for (file_idx, lines) in file_lines.iter().enumerate() {
                    if file_idx > 0 {
                        io::write_all(1, &[delimiter]);
                    }
                    if line_idx < lines.len() {
                        io::write_all(1, lines[line_idx]);
                    }
                }
                io::write_str(1, b"\n");
            }
        }
    }
    0
}

/// sort - sort lines
pub fn sort(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let mut reverse = false;
        let mut numeric = false;
        let mut unique = false;
        let mut file_idx = 0;

        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if arg[0] == b'-' && arg.len() > 1 {
                    if has_opt(arg, b'r') { reverse = true; }
                    if has_opt(arg, b'n') { numeric = true; }
                    if has_opt(arg, b'u') { unique = true; }
                } else if file_idx == 0 {
                    file_idx = i;
                }
            }
        }

        // Read from file or stdin
        let content = if file_idx > 0 {
            if let Some(path) = unsafe { get_arg(argv, file_idx) } {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd < 0 {
                    io::write_str(2, b"sort: cannot open file\n");
                    return 1;
                }
                let c = io::read_all(fd);
                io::close(fd);
                c
            } else {
                io::read_all(0)
            }
        } else {
            io::read_all(0)
        };

        let mut lines: Vec<&[u8]> = content.split(|&c| c == b'\n').filter(|l| !l.is_empty()).collect();

        if numeric {
            lines.sort_by(|a, b| {
                let na = sys::parse_i64(a).unwrap_or(0);
                let nb = sys::parse_i64(b).unwrap_or(0);
                na.cmp(&nb)
            });
        } else {
            lines.sort();
        }

        if reverse {
            lines.reverse();
        }

        let mut last: Option<&[u8]> = None;
        for line in lines {
            if unique {
                if Some(line) == last { continue; }
                last = Some(line);
            }
            io::write_all(1, line);
            io::write_str(1, b"\n");
        }
    }
    0
}

/// uniq - report or omit repeated lines
pub fn uniq(argc: i32, argv: *const *const u8) -> i32 {
    let mut count = false;
    let mut repeated = false;
    let mut unique_only = false;
    let mut file_idx = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' && arg.len() > 1 {
                if has_opt(arg, b'c') { count = true; }
                if has_opt(arg, b'd') { repeated = true; }
                if has_opt(arg, b'u') { unique_only = true; }
            } else if file_idx == 0 {
                file_idx = i;
            }
        }
    }

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        // Read from file or stdin
        let content = if file_idx > 0 {
            if let Some(path) = unsafe { get_arg(argv, file_idx) } {
                let fd = io::open(path, libc::O_RDONLY, 0);
                if fd < 0 {
                    io::write_str(2, b"uniq: cannot open file\n");
                    return 1;
                }
                let c = io::read_all(fd);
                io::close(fd);
                c
            } else {
                io::read_all(0)
            }
        } else {
            io::read_all(0)
        };

        let lines: Vec<&[u8]> = content.split(|&c| c == b'\n').collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let mut cnt = 1;

            while i + cnt < lines.len() && lines[i + cnt] == line {
                cnt += 1;
            }

            let should_print = if repeated {
                cnt > 1
            } else if unique_only {
                cnt == 1
            } else {
                true
            };

            if should_print && !line.is_empty() {
                if count {
                    io::write_num(1, cnt as u64);
                    io::write_str(1, b" ");
                }
                io::write_all(1, line);
                io::write_str(1, b"\n");
            }

            i += cnt;
        }
    }
    0
}

/// grep - search for patterns
pub fn grep(argc: i32, argv: *const *const u8) -> i32 {
    let mut invert = false;
    let mut count_only = false;
    let mut line_numbers = false;
    let mut ignore_case = false;
    let mut quiet = false;
    let mut files_with_matches = false;
    let mut pattern_idx = 0;
    let mut files_start = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' && arg.len() > 1 {
                if has_opt(arg, b'v') { invert = true; }
                if has_opt(arg, b'c') { count_only = true; }
                if has_opt(arg, b'n') { line_numbers = true; }
                if has_opt(arg, b'i') { ignore_case = true; }
                if has_opt(arg, b'q') { quiet = true; }
                if has_opt(arg, b'l') { files_with_matches = true; }
            } else if pattern_idx == 0 {
                pattern_idx = i;
            } else if files_start == 0 {
                files_start = i;
            }
        }
    }

    if pattern_idx == 0 {
        io::write_str(2, b"grep: missing pattern\n");
        return 2;
    }

    let pattern = unsafe { get_arg(argv, pattern_idx).unwrap() };
    let mut total_count = 0u64;
    let mut found_match = false;

    // If no files specified, read from stdin
    if files_start == 0 {
        let count = grep_fd(0, None, pattern, invert, count_only, line_numbers, ignore_case, quiet, files_with_matches);
        if count > 0 { found_match = true; }
        total_count += count;
    } else {
        // Process each file
        let multiple_files = (argc - files_start) > 1;
        for i in files_start..argc {
            if let Some(file) = unsafe { get_arg(argv, i) } {
                let fd = if file == b"-" {
                    0
                } else {
                    io::open(file, libc::O_RDONLY, 0)
                };

                if fd < 0 {
                    io::write_str(2, b"grep: ");
                    io::write_all(2, file);
                    io::write_str(2, b": No such file or directory\n");
                    continue;
                }

                let prefix = if multiple_files { Some(file) } else { None };
                let count = grep_fd(fd, prefix, pattern, invert, count_only, line_numbers, ignore_case, quiet, files_with_matches);
                if count > 0 { found_match = true; }
                total_count += count;

                if fd != 0 {
                    io::close(fd);
                }

                // For -l, stop after first match in file
                if files_with_matches && count > 0 {
                    continue;
                }
            }
        }
    }

    if found_match { 0 } else { 1 }
}

fn grep_fd(fd: i32, prefix: Option<&[u8]>, pattern: &[u8], invert: bool, count_only: bool,
           line_numbers: bool, ignore_case: bool, quiet: bool, files_with_matches: bool) -> u64 {
    let mut count = 0u64;
    let mut line_num = 0u64;
    let mut buf = [0u8; 4096];
    let mut line = [0u8; 4096];
    let mut line_len = 0;

    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }

        for &c in &buf[..n as usize] {
            if c == b'\n' {
                line_num += 1;
                let matches = if ignore_case {
                    contains_ignore_case(&line[..line_len], pattern)
                } else {
                    contains(&line[..line_len], pattern)
                };

                if matches != invert {
                    count += 1;

                    if files_with_matches {
                        // For -l, just print filename once and return
                        if let Some(p) = prefix {
                            io::write_all(1, p);
                            io::write_str(1, b"\n");
                        }
                        return count;
                    }

                    if !count_only && !quiet {
                        if let Some(p) = prefix {
                            io::write_all(1, p);
                            io::write_str(1, b":");
                        }
                        if line_numbers {
                            io::write_num(1, line_num);
                            io::write_str(1, b":");
                        }
                        io::write_all(1, &line[..line_len]);
                        io::write_str(1, b"\n");
                    }
                }
                line_len = 0;
            } else if line_len < line.len() {
                line[line_len] = c;
                line_len += 1;
            }
        }
    }

    // Handle last line if no trailing newline
    if line_len > 0 {
        line_num += 1;
        let matches = if ignore_case {
            contains_ignore_case(&line[..line_len], pattern)
        } else {
            contains(&line[..line_len], pattern)
        };

        if matches != invert {
            count += 1;
            if !count_only && !quiet && !files_with_matches {
                if let Some(p) = prefix {
                    io::write_all(1, p);
                    io::write_str(1, b":");
                }
                if line_numbers {
                    io::write_num(1, line_num);
                    io::write_str(1, b":");
                }
                io::write_all(1, &line[..line_len]);
                io::write_str(1, b"\n");
            }
        }
    }

    if count_only && !quiet {
        if let Some(p) = prefix {
            io::write_all(1, p);
            io::write_str(1, b":");
        }
        io::write_num(1, count);
        io::write_str(1, b"\n");
    }

    count
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() { return true; }
    if haystack.len() < needle.len() { return false; }

    for i in 0..=(haystack.len() - needle.len()) {
        if &haystack[i..i+needle.len()] == needle {
            return true;
        }
    }
    false
}

fn contains_ignore_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() { return true; }
    if haystack.len() < needle.len() { return false; }

    for i in 0..=(haystack.len() - needle.len()) {
        let mut matches = true;
        for j in 0..needle.len() {
            let h = if haystack[i+j] >= b'A' && haystack[i+j] <= b'Z' {
                haystack[i+j] + 32
            } else {
                haystack[i+j]
            };
            let n = if needle[j] >= b'A' && needle[j] <= b'Z' {
                needle[j] + 32
            } else {
                needle[j]
            };
            if h != n {
                matches = false;
                break;
            }
        }
        if matches { return true; }
    }
    false
}

/// egrep - extended grep
pub fn egrep(argc: i32, argv: *const *const u8) -> i32 {
    grep(argc, argv)
}

/// fgrep - fixed string grep
pub fn fgrep(argc: i32, argv: *const *const u8) -> i32 {
    grep(argc, argv)
}

/// sed - stream editor
pub fn sed(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"sed: missing script\n");
        return 1;
    }

    // Find script
    let mut script: Option<&[u8]> = None;
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'e') && i + 1 < argc {
                script = unsafe { get_arg(argv, i + 1) };
                break;
            } else if arg[0] != b'-' {
                script = Some(arg);
                break;
            }
        }
    }

    let script = match script {
        Some(s) => s,
        None => return 1,
    };

    // Parse s/pattern/replacement/flags
    if script.len() > 2 && script[0] == b's' {
        let delim = script[1];
        let mut parts = [0usize; 4];
        let mut part = 0;
        parts[0] = 2;

        for i in 2..script.len() {
            if script[i] == delim && part < 3 {
                part += 1;
                parts[part] = i + 1;
            }
        }

        if part >= 2 {
            let pattern = &script[parts[0]..parts[1]-1];
            let replacement = &script[parts[1]..parts[2]-1];
            let global = part >= 3 && script[parts[2]..].contains(&b'g');

            let mut buf = [0u8; 4096];
            let mut line = [0u8; 4096];
            let mut line_len = 0;

            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 { break; }

                for &c in &buf[..n as usize] {
                    if c == b'\n' {
                        // Do substitution
                        let mut result = [0u8; 4096];
                        let mut result_len = 0;
                        let mut i = 0;
                        let mut did_replace = false;

                        while i < line_len {
                            if i + pattern.len() <= line_len && &line[i..i+pattern.len()] == pattern {
                                for &r in replacement {
                                    if result_len < result.len() {
                                        result[result_len] = r;
                                        result_len += 1;
                                    }
                                }
                                i += pattern.len();
                                did_replace = true;
                                if !global {
                                    // Copy rest
                                    while i < line_len && result_len < result.len() {
                                        result[result_len] = line[i];
                                        result_len += 1;
                                        i += 1;
                                    }
                                    break;
                                }
                            } else {
                                if result_len < result.len() {
                                    result[result_len] = line[i];
                                    result_len += 1;
                                }
                                i += 1;
                            }
                        }

                        if did_replace {
                            io::write_all(1, &result[..result_len]);
                        } else {
                            io::write_all(1, &line[..line_len]);
                        }
                        io::write_str(1, b"\n");
                        line_len = 0;
                    } else if line_len < line.len() {
                        line[line_len] = c;
                        line_len += 1;
                    }
                }
            }
        }
    } else {
        // Other commands - just pass through
        let mut buf = [0u8; 4096];
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 { break; }
            io::write_all(1, &buf[..n as usize]);
        }
    }
    0
}

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

/// comm - compare sorted files
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
            if path[0] != b'-' {
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
            if path[0] != b'-' {
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
            if path[0] != b'-' {
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
            if path[0] != b'-' {
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
            if path[0] != b'-' {
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
