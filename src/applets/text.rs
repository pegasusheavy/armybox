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
        use alloc::vec::Vec;
        use alloc::collections::VecDeque;

        let content = io::read_all(fd);
        let mut line_starts: VecDeque<usize> = VecDeque::new();
        line_starts.push_back(0);

        for (i, &c) in content.iter().enumerate() {
            if c == b'\n' && i + 1 < content.len() {
                line_starts.push_back(i + 1);
                if line_starts.len() > lines + 1 {
                    line_starts.pop_front();
                }
            }
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
    0
}

fn expand_set(s: &[u8]) -> &[u8] {
    // Simplified - just return as-is
    // Full impl would handle [:alpha:], a-z, etc.
    s
}

/// cut - remove sections from lines
pub fn cut(argc: i32, argv: *const *const u8) -> i32 {
    let mut delimiter = b'\t';
    let mut field: Option<usize> = None;
    let mut chars: Option<usize> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'd') && i + 1 < argc {
                if let Some(d) = unsafe { get_arg(argv, i + 1) } {
                    if !d.is_empty() { delimiter = d[0]; }
                }
            } else if has_opt(arg, b'f') && i + 1 < argc {
                if let Some(f) = unsafe { get_arg(argv, i + 1) } {
                    field = Some(sys::parse_u64(f).unwrap_or(1) as usize);
                }
            } else if has_opt(arg, b'c') && i + 1 < argc {
                if let Some(c) = unsafe { get_arg(argv, i + 1) } {
                    chars = Some(sys::parse_u64(c).unwrap_or(1) as usize);
                }
            }
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
                if let Some(f) = field {
                    let mut field_num = 1;
                    let mut start = 0;

                    for j in 0..line_len {
                        if line[j] == delimiter {
                            if field_num == f {
                                io::write_all(1, &line[start..j]);
                                break;
                            }
                            field_num += 1;
                            start = j + 1;
                        }
                    }
                    if field_num == f {
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
pub fn paste(argc: i32, argv: *const *const u8) -> i32 {
    io::write_str(2, b"paste: stub\n");
    let _ = argc;
    let _ = argv;
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

        for i in 1..argc {
            if let Some(arg) = unsafe { get_arg(argv, i) } {
                if has_opt(arg, b'r') { reverse = true; }
                if has_opt(arg, b'n') { numeric = true; }
                if has_opt(arg, b'u') { unique = true; }
            }
        }

        let content = io::read_all(0);
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

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if has_opt(arg, b'c') { count = true; }
            if has_opt(arg, b'd') { repeated = true; }
            if has_opt(arg, b'u') { unique_only = true; }
        }
    }

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;

        let content = io::read_all(0);
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
    let mut pattern_idx = 0;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg[0] == b'-' {
                if has_opt(arg, b'v') { invert = true; }
                if has_opt(arg, b'c') { count_only = true; }
                if has_opt(arg, b'n') { line_numbers = true; }
                if has_opt(arg, b'i') { ignore_case = true; }
            } else if pattern_idx == 0 {
                pattern_idx = i;
            }
        }
    }

    if pattern_idx == 0 {
        io::write_str(2, b"grep: missing pattern\n");
        return 2;
    }

    let pattern = unsafe { get_arg(argv, pattern_idx).unwrap() };

    let mut count = 0u64;
    let mut line_num = 0u64;
    let mut buf = [0u8; 4096];
    let mut line = [0u8; 4096];
    let mut line_len = 0;

    loop {
        let n = io::read(0, &mut buf);
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
                    if !count_only {
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

    if count_only {
        io::write_num(1, count);
        io::write_str(1, b"\n");
    }

    if count > 0 { 0 } else { 1 }
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
pub fn comm(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"comm: stub\n");
    0
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
