//! Stub utilities - simple or unimplemented utilities
//!
//! These utilities are stubs that need full implementation.

use crate::io;
use crate::sys;
use super::get_arg;

/// ascii - display ASCII table
pub fn ascii(_argc: i32, _argv: *const *const u8) -> i32 {
    // Control character names
    const CTRL_NAMES: [&[u8]; 32] = [
        b"NUL", b"SOH", b"STX", b"ETX", b"EOT", b"ENQ", b"ACK", b"BEL",
        b"BS ", b"HT ", b"LF ", b"VT ", b"FF ", b"CR ", b"SO ", b"SI ",
        b"DLE", b"DC1", b"DC2", b"DC3", b"DC4", b"NAK", b"SYN", b"ETB",
        b"CAN", b"EM ", b"SUB", b"ESC", b"FS ", b"GS ", b"RS ", b"US ",
    ];

    io::write_str(1, b"Dec Hex    Dec Hex    Dec Hex  Dec Hex  Dec Hex  Dec Hex   Dec Hex   Dec Hex\n");

    for row in 0..16 {
        for col in 0..8 {
            let val = row + col * 16;
            let mut buf = [0u8; 16];

            // Format decimal (right-aligned in 3 chars)
            if val < 10 {
                io::write_str(1, b"  ");
            } else if val < 100 {
                io::write_str(1, b" ");
            }
            let dec = sys::format_u64(val as u64, &mut buf);
            io::write_all(1, dec);
            io::write_str(1, b" ");

            // Format hex
            let hex = sys::format_hex(val as u64, &mut buf);
            if hex.len() < 2 {
                io::write_str(1, b"0");
            }
            io::write_all(1, hex);
            io::write_str(1, b" ");

            // Format character
            if val < 32 {
                io::write_all(1, CTRL_NAMES[val]);
            } else if val == 32 {
                io::write_str(1, b"   ");
            } else if val == 127 {
                io::write_str(1, b"DEL");
            } else {
                io::write_all(1, &[val as u8, b' ', b' ']);
            }

            if col < 7 {
                io::write_str(1, b" ");
            }
        }
        io::write_str(1, b"\n");
    }

    0
}

/// iconv - character set conversion (POSIX compliant)
///
/// POSIX: Converts text between character encodings.
/// Supported: ASCII, UTF-8, ISO-8859-1 (Latin-1)
#[cfg(feature = "alloc")]
pub fn iconv(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    let mut from_code: &[u8] = b"UTF-8";
    let mut to_code: &[u8] = b"UTF-8";
    let mut input_files: Vec<&[u8]> = Vec::new();
    let mut i = 1;

    // Parse arguments
    while i < argc as usize {
        // SAFETY: argv is valid for argc elements
        let arg = match unsafe { get_arg(argv, i as i32) } {
            Some(a) => a,
            None => break,
        };

        if arg == b"-f" || arg == b"--from-code" {
            i += 1;
            if let Some(code) = unsafe { get_arg(argv, i as i32) } {
                from_code = code;
            }
        } else if arg == b"-t" || arg == b"--to-code" {
            i += 1;
            if let Some(code) = unsafe { get_arg(argv, i as i32) } {
                to_code = code;
            }
        } else if arg == b"-l" || arg == b"--list" {
            io::write_str(1, b"ASCII\nUTF-8\nISO-8859-1\nLATIN1\n");
            return 0;
        } else if !arg.starts_with(b"-") {
            input_files.push(arg);
        }
        i += 1;
    }

    // Normalize encoding names (case-insensitive)
    let from_enc = normalize_encoding(from_code);
    let to_enc = normalize_encoding(to_code);

    // Process files or stdin
    if input_files.is_empty() {
        let content = io::read_all(0);
        convert_and_output(&content, from_enc, to_enc);
    } else {
        for path in input_files {
            let fd = io::open(path, libc::O_RDONLY, 0);
            if fd < 0 {
                io::write_str(2, b"iconv: ");
                io::write_all(2, path);
                io::write_str(2, b": No such file or directory\n");
                return 1;
            }
            let content = io::read_all(fd);
            io::close(fd);
            convert_and_output(&content, from_enc, to_enc);
        }
    }

    0
}

#[cfg(not(feature = "alloc"))]
pub fn iconv(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"iconv: requires alloc feature\n");
    1
}

/// Encoding types we support
#[cfg(feature = "alloc")]
#[derive(Clone, Copy, PartialEq)]
enum Encoding {
    Ascii,
    Utf8,
    Latin1,
}

/// Normalize encoding name
#[cfg(feature = "alloc")]
fn normalize_encoding(name: &[u8]) -> Encoding {
    // Case-insensitive comparison
    let upper: alloc::vec::Vec<u8> = name.iter().map(|c| c.to_ascii_uppercase()).collect();

    if upper == b"ASCII" || upper == b"US-ASCII" {
        Encoding::Ascii
    } else if upper == b"UTF-8" || upper == b"UTF8" {
        Encoding::Utf8
    } else if upper == b"ISO-8859-1" || upper == b"ISO88591" || upper == b"LATIN1" || upper == b"LATIN-1" {
        Encoding::Latin1
    } else {
        // Default to UTF-8
        Encoding::Utf8
    }
}

/// Convert and output data
#[cfg(feature = "alloc")]
fn convert_and_output(data: &[u8], from: Encoding, to: Encoding) {
    use alloc::vec::Vec;

    // First decode to Unicode codepoints
    let codepoints = decode_to_codepoints(data, from);

    // Then encode to target
    let output = encode_from_codepoints(&codepoints, to);

    io::write_all(1, &output);
}

/// Decode bytes to Unicode codepoints
#[cfg(feature = "alloc")]
fn decode_to_codepoints(data: &[u8], enc: Encoding) -> alloc::vec::Vec<u32> {
    use alloc::vec::Vec;
    let mut result: Vec<u32> = Vec::new();

    match enc {
        Encoding::Ascii => {
            for &b in data {
                if b < 128 {
                    result.push(b as u32);
                } else {
                    result.push(0xFFFD); // Replacement character
                }
            }
        }
        Encoding::Latin1 => {
            for &b in data {
                result.push(b as u32); // ISO-8859-1 maps directly to Unicode
            }
        }
        Encoding::Utf8 => {
            let mut i = 0;
            while i < data.len() {
                let b = data[i];
                if b < 0x80 {
                    result.push(b as u32);
                    i += 1;
                } else if b < 0xC0 {
                    result.push(0xFFFD);
                    i += 1;
                } else if b < 0xE0 {
                    if i + 1 < data.len() && data[i + 1] & 0xC0 == 0x80 {
                        let cp = ((b as u32 & 0x1F) << 6) | (data[i + 1] as u32 & 0x3F);
                        result.push(cp);
                        i += 2;
                    } else {
                        result.push(0xFFFD);
                        i += 1;
                    }
                } else if b < 0xF0 {
                    if i + 2 < data.len() && data[i + 1] & 0xC0 == 0x80 && data[i + 2] & 0xC0 == 0x80 {
                        let cp = ((b as u32 & 0x0F) << 12)
                            | ((data[i + 1] as u32 & 0x3F) << 6)
                            | (data[i + 2] as u32 & 0x3F);
                        result.push(cp);
                        i += 3;
                    } else {
                        result.push(0xFFFD);
                        i += 1;
                    }
                } else if b < 0xF8 {
                    if i + 3 < data.len()
                        && data[i + 1] & 0xC0 == 0x80
                        && data[i + 2] & 0xC0 == 0x80
                        && data[i + 3] & 0xC0 == 0x80
                    {
                        let cp = ((b as u32 & 0x07) << 18)
                            | ((data[i + 1] as u32 & 0x3F) << 12)
                            | ((data[i + 2] as u32 & 0x3F) << 6)
                            | (data[i + 3] as u32 & 0x3F);
                        result.push(cp);
                        i += 4;
                    } else {
                        result.push(0xFFFD);
                        i += 1;
                    }
                } else {
                    result.push(0xFFFD);
                    i += 1;
                }
            }
        }
    }

    result
}

/// Encode codepoints to bytes
#[cfg(feature = "alloc")]
fn encode_from_codepoints(codepoints: &[u32], enc: Encoding) -> alloc::vec::Vec<u8> {
    use alloc::vec::Vec;
    let mut result: Vec<u8> = Vec::new();

    match enc {
        Encoding::Ascii => {
            for &cp in codepoints {
                if cp < 128 {
                    result.push(cp as u8);
                } else {
                    result.push(b'?'); // Replacement for non-ASCII
                }
            }
        }
        Encoding::Latin1 => {
            for &cp in codepoints {
                if cp < 256 {
                    result.push(cp as u8);
                } else {
                    result.push(b'?'); // Replacement for non-Latin1
                }
            }
        }
        Encoding::Utf8 => {
            for &cp in codepoints {
                if cp < 0x80 {
                    result.push(cp as u8);
                } else if cp < 0x800 {
                    result.push(0xC0 | ((cp >> 6) as u8));
                    result.push(0x80 | ((cp & 0x3F) as u8));
                } else if cp < 0x10000 {
                    result.push(0xE0 | ((cp >> 12) as u8));
                    result.push(0x80 | (((cp >> 6) & 0x3F) as u8));
                    result.push(0x80 | ((cp & 0x3F) as u8));
                } else if cp < 0x110000 {
                    result.push(0xF0 | ((cp >> 18) as u8));
                    result.push(0x80 | (((cp >> 12) & 0x3F) as u8));
                    result.push(0x80 | (((cp >> 6) & 0x3F) as u8));
                    result.push(0x80 | ((cp & 0x3F) as u8));
                }
            }
        }
    }

    result
}

/// tsort - topological sort (POSIX compliant)
///
/// POSIX: Reads pairs of strings from stdin or file, outputs topological order.
/// Each pair "a b" means "a" depends on "b" (b must come before a).
/// Detects cycles and reports them to stderr.
#[cfg(feature = "alloc")]
pub fn tsort(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::collections::BTreeMap;
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    // Open input file or use stdin
    // SAFETY: argv is valid for argc elements, passed from main
    let fd = if argc > 1 {
        match unsafe { get_arg(argv, 1) } {
            Some(filename) if filename == b"-" => 0,
            Some(filename) => {
                let fd = io::open(filename, libc::O_RDONLY, 0);
                if fd < 0 {
                    io::write_str(2, b"tsort: ");
                    io::write_all(2, filename);
                    io::write_str(2, b": No such file or directory\n");
                    return 1;
                }
                fd
            }
            None => 0,
        }
    } else {
        0 // stdin
    };

    // Read all input
    let content = io::read_all(fd);
    if fd > 0 {
        io::close(fd);
    }

    // Parse pairs and build graph
    // Graph: node -> set of nodes it depends on (must come before it)
    let mut nodes: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut edges: BTreeMap<Vec<u8>, BTreeSet<Vec<u8>>> = BTreeMap::new(); // node -> predecessors
    let mut successors: BTreeMap<Vec<u8>, BTreeSet<Vec<u8>>> = BTreeMap::new(); // node -> what depends on it

    let mut i = 0;
    while i < content.len() {
        // Skip whitespace
        while i < content.len() && (content[i] == b' ' || content[i] == b'\t' || content[i] == b'\n' || content[i] == b'\r') {
            i += 1;
        }
        if i >= content.len() {
            break;
        }

        // Read first word
        let start1 = i;
        while i < content.len() && content[i] != b' ' && content[i] != b'\t' && content[i] != b'\n' {
            i += 1;
        }
        let word1: Vec<u8> = content[start1..i].to_vec();

        // Skip whitespace between words
        while i < content.len() && (content[i] == b' ' || content[i] == b'\t') {
            i += 1;
        }

        // Read second word (if present on same line)
        let start2 = i;
        while i < content.len() && content[i] != b' ' && content[i] != b'\t' && content[i] != b'\n' {
            i += 1;
        }
        let word2: Vec<u8> = content[start2..i].to_vec();

        // Add nodes
        if !word1.is_empty() {
            nodes.insert(word1.clone());
            if !edges.contains_key(&word1) {
                edges.insert(word1.clone(), BTreeSet::new());
            }
            if !successors.contains_key(&word1) {
                successors.insert(word1.clone(), BTreeSet::new());
            }
        }
        if !word2.is_empty() {
            nodes.insert(word2.clone());
            if !edges.contains_key(&word2) {
                edges.insert(word2.clone(), BTreeSet::new());
            }
            if !successors.contains_key(&word2) {
                successors.insert(word2.clone(), BTreeSet::new());
            }
        }

        // Add edge: word1 depends on word2 (word2 must come before word1)
        if !word1.is_empty() && !word2.is_empty() && word1 != word2 {
            edges.get_mut(&word1).unwrap().insert(word2.clone());
            successors.get_mut(&word2).unwrap().insert(word1.clone());
        }
    }

    // Kahn's algorithm for topological sort
    let mut result: Vec<Vec<u8>> = Vec::new();
    let mut in_degree: BTreeMap<Vec<u8>, usize> = BTreeMap::new();

    // Calculate in-degrees
    for node in &nodes {
        let deg = edges.get(node).map(|s| s.len()).unwrap_or(0);
        in_degree.insert(node.clone(), deg);
    }

    // Find all nodes with no dependencies
    let mut queue: Vec<Vec<u8>> = Vec::new();
    for node in &nodes {
        if *in_degree.get(node).unwrap_or(&0) == 0 {
            queue.push(node.clone());
        }
    }

    // Sort queue for deterministic output
    queue.sort();

    while !queue.is_empty() {
        // Take first (alphabetically) for deterministic output
        let node = queue.remove(0);
        result.push(node.clone());

        // Remove this node from graph
        if let Some(succs) = successors.get(&node) {
            for succ in succs.iter() {
                if let Some(deg) = in_degree.get_mut(succ) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        // Insert in sorted position for determinism
                        let pos = queue.iter().position(|x| x > succ).unwrap_or(queue.len());
                        queue.insert(pos, succ.clone());
                    }
                }
            }
        }
    }

    // Check for cycles
    let has_cycle = result.len() != nodes.len();

    // Output results
    for node in &result {
        io::write_all(1, node);
        io::write_str(1, b"\n");
    }

    if has_cycle {
        io::write_str(2, b"tsort: input contains a loop\n");
        // Output remaining nodes (in cycle)
        for node in &nodes {
            if !result.contains(node) {
                io::write_all(1, node);
                io::write_str(1, b"\n");
            }
        }
        return 1;
    }

    0
}

#[cfg(not(feature = "alloc"))]
pub fn tsort(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"tsort: requires alloc feature\n");
    1
}

/// getopt - parse command options (POSIX compliant)
///
/// POSIX: Parses command-line options and outputs them in normalized form.
/// Usage: getopt optstring parameters...
/// Outputs: normalized options, then "--", then non-option arguments.
#[cfg(feature = "alloc")]
pub fn getopt(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    if argc < 2 {
        io::write_str(2, b"getopt: missing optstring\n");
        return 1;
    }

    // SAFETY: argv is valid for argc elements
    let optstring = match unsafe { get_arg(argv, 1) } {
        Some(s) => s,
        None => return 1,
    };

    let mut options: Vec<Vec<u8>> = Vec::new();
    let mut operands: Vec<&[u8]> = Vec::new();
    let mut i = 2;
    let mut error = false;

    while i < argc as usize {
        let arg = match unsafe { get_arg(argv, i as i32) } {
            Some(a) => a,
            None => break,
        };

        if arg == b"--" {
            // End of options
            i += 1;
            while i < argc as usize {
                if let Some(a) = unsafe { get_arg(argv, i as i32) } {
                    operands.push(a);
                }
                i += 1;
            }
            break;
        } else if arg.starts_with(b"-") && arg.len() > 1 && arg[1] != b'-' {
            // Short options
            let mut j = 1;
            while j < arg.len() {
                let opt = arg[j];
                let opt_pos = optstring.iter().position(|&c| c == opt);

                match opt_pos {
                    Some(pos) => {
                        // Check if option takes an argument
                        let takes_arg = pos + 1 < optstring.len() && optstring[pos + 1] == b':';

                        if takes_arg {
                            if j + 1 < arg.len() {
                                // Argument is attached: -oARG
                                let mut opt_str = Vec::with_capacity(3);
                                opt_str.push(b'-');
                                opt_str.push(opt);
                                options.push(opt_str);
                                let arg_val: Vec<u8> = arg[j + 1..].to_vec();
                                options.push(arg_val);
                                break;
                            } else {
                                // Argument is next parameter
                                i += 1;
                                let mut opt_str = Vec::with_capacity(3);
                                opt_str.push(b'-');
                                opt_str.push(opt);
                                options.push(opt_str);

                                if let Some(next_arg) = unsafe { get_arg(argv, i as i32) } {
                                    options.push(next_arg.to_vec());
                                } else {
                                    io::write_str(2, b"getopt: option requires an argument -- ");
                                    io::write_all(2, &[opt]);
                                    io::write_str(2, b"\n");
                                    error = true;
                                }
                                break;
                            }
                        } else {
                            // Option without argument
                            let mut opt_str = Vec::with_capacity(3);
                            opt_str.push(b'-');
                            opt_str.push(opt);
                            options.push(opt_str);
                        }
                    }
                    None => {
                        io::write_str(2, b"getopt: invalid option -- ");
                        io::write_all(2, &[opt]);
                        io::write_str(2, b"\n");
                        error = true;
                    }
                }
                j += 1;
            }
        } else {
            // Non-option argument
            operands.push(arg);
        }
        i += 1;
    }

    // Output options
    let mut first = true;
    for opt in &options {
        if !first {
            io::write_str(1, b" ");
        }
        // Quote if contains spaces
        if opt.contains(&b' ') || opt.contains(&b'\t') {
            io::write_str(1, b"'");
            io::write_all(1, opt);
            io::write_str(1, b"'");
        } else {
            io::write_all(1, opt);
        }
        first = false;
    }

    // Output separator
    if !first {
        io::write_str(1, b" ");
    }
    io::write_str(1, b"--");

    // Output operands
    for op in &operands {
        io::write_str(1, b" ");
        if op.contains(&b' ') || op.contains(&b'\t') {
            io::write_str(1, b"'");
            io::write_all(1, op);
            io::write_str(1, b"'");
        } else {
            io::write_all(1, op);
        }
    }

    io::write_str(1, b"\n");

    if error { 1 } else { 0 }
}

#[cfg(not(feature = "alloc"))]
pub fn getopt(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"getopt: requires alloc feature\n");
    1
}

/// count - count bytes from stdin
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

/// unicode - display Unicode character information
///
/// Usage: unicode [codepoint...]
/// Shows information about Unicode codepoints (decimal, hex, or U+XXXX notation)
pub fn unicode(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"Usage: unicode <codepoint>...\n");
        io::write_str(2, b"Examples: unicode 65, unicode 0x41, unicode U+0041\n");
        return 1;
    }

    for i in 1..argc as usize {
        // SAFETY: argv is valid for argc elements
        let arg = match unsafe { get_arg(argv, i as i32) } {
            Some(a) => a,
            None => continue,
        };

        let codepoint = parse_codepoint(arg);

        if let Some(cp) = codepoint {
            // Print codepoint info
            let mut buf = [0u8; 16];

            io::write_str(1, b"U+");
            let hex = sys::format_hex(cp as u64, &mut buf);
            for _ in 0..(4usize.saturating_sub(hex.len())) {
                io::write_str(1, b"0");
            }
            io::write_all(1, hex);

            io::write_str(1, b" (");
            let dec = sys::format_u64(cp as u64, &mut buf);
            io::write_all(1, dec);
            io::write_str(1, b") ");

            // Print the character if printable
            if cp < 0x110000 {
                let mut utf8 = [0u8; 4];
                let len = encode_utf8(cp, &mut utf8);
                if len > 0 {
                    io::write_str(1, b"'");
                    io::write_all(1, &utf8[..len]);
                    io::write_str(1, b"'");
                }
            }

            io::write_str(1, b"\n");
        } else {
            io::write_str(2, b"unicode: invalid codepoint: ");
            io::write_all(2, arg);
            io::write_str(2, b"\n");
        }
    }

    0
}

/// Parse codepoint from various formats
fn parse_codepoint(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }

    // U+XXXX format
    if s.len() > 2 && (s[0] == b'U' || s[0] == b'u') && s[1] == b'+' {
        return parse_hex(&s[2..]);
    }

    // 0xXXXX format
    if s.len() > 2 && s[0] == b'0' && (s[1] == b'x' || s[1] == b'X') {
        return parse_hex(&s[2..]);
    }

    // Decimal
    sys::parse_u64(s).map(|n| n as u32)
}

/// Parse hex number
fn parse_hex(s: &[u8]) -> Option<u32> {
    let mut result: u32 = 0;
    for &c in s {
        let digit = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => return None,
        };
        result = result.checked_mul(16)?.checked_add(digit as u32)?;
    }
    Some(result)
}

/// Encode Unicode codepoint to UTF-8
fn encode_utf8(cp: u32, buf: &mut [u8; 4]) -> usize {
    if cp < 0x80 {
        buf[0] = cp as u8;
        1
    } else if cp < 0x800 {
        buf[0] = 0xC0 | ((cp >> 6) as u8);
        buf[1] = 0x80 | ((cp & 0x3F) as u8);
        2
    } else if cp < 0x10000 {
        buf[0] = 0xE0 | ((cp >> 12) as u8);
        buf[1] = 0x80 | (((cp >> 6) & 0x3F) as u8);
        buf[2] = 0x80 | ((cp & 0x3F) as u8);
        3
    } else if cp < 0x110000 {
        buf[0] = 0xF0 | ((cp >> 18) as u8);
        buf[1] = 0x80 | (((cp >> 12) & 0x3F) as u8);
        buf[2] = 0x80 | (((cp >> 6) & 0x3F) as u8);
        buf[3] = 0x80 | ((cp & 0x3F) as u8);
        4
    } else {
        0
    }
}

/// ts - timestamp input lines
///
/// Usage: ts [-s] [format]
/// Prepends each line from stdin with a timestamp.
/// -s: Use time since start instead of wall clock
#[cfg(feature = "alloc")]
pub fn ts(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    let mut use_elapsed = false;
    let mut format: Option<&[u8]> = None;

    // Parse arguments
    for i in 1..argc as usize {
        let arg = match unsafe { get_arg(argv, i as i32) } {
            Some(a) => a,
            None => continue,
        };

        if arg == b"-s" {
            use_elapsed = true;
        } else if arg == b"-i" {
            // Incremental mode - ignore for now
        } else if !arg.starts_with(b"-") {
            format = Some(arg);
        }
    }

    let start_time = unsafe { libc::time(core::ptr::null_mut()) };
    let mut line = Vec::new();
    let mut buf = [0u8; 1];

    loop {
        // Read a line
        line.clear();
        loop {
            let n = io::read(0, &mut buf);
            if n <= 0 {
                if line.is_empty() {
                    return 0;
                }
                break;
            }
            if buf[0] == b'\n' {
                break;
            }
            line.push(buf[0]);
        }

        // Print timestamp
        let now = unsafe { libc::time(core::ptr::null_mut()) };

        if use_elapsed {
            let elapsed = (now - start_time) as u64;
            let hours = elapsed / 3600;
            let mins = (elapsed % 3600) / 60;
            let secs = elapsed % 60;

            let mut num_buf = [0u8; 8];

            let h = sys::format_u64(hours, &mut num_buf);
            if h.len() < 2 { io::write_str(1, b"0"); }
            io::write_all(1, h);
            io::write_str(1, b":");

            let m = sys::format_u64(mins, &mut num_buf);
            if m.len() < 2 { io::write_str(1, b"0"); }
            io::write_all(1, m);
            io::write_str(1, b":");

            let s = sys::format_u64(secs, &mut num_buf);
            if s.len() < 2 { io::write_str(1, b"0"); }
            io::write_all(1, s);
        } else {
            // Format wall clock time
            format_timestamp(now, format);
        }

        io::write_str(1, b" ");
        io::write_all(1, &line);
        io::write_str(1, b"\n");
    }
}

#[cfg(not(feature = "alloc"))]
pub fn ts(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"ts: requires alloc feature\n");
    1
}

/// Format a timestamp
#[cfg(feature = "alloc")]
fn format_timestamp(time: libc::time_t, _format: Option<&[u8]>) {
    // Simple ISO-ish format: YYYY-MM-DD HH:MM:SS
    let mut tm: libc::tm = unsafe { core::mem::zeroed() };
    unsafe {
        libc::localtime_r(&time, &mut tm);
    }

    let mut buf = [0u8; 8];

    // Year
    let year = sys::format_u64((tm.tm_year + 1900) as u64, &mut buf);
    io::write_all(1, year);
    io::write_str(1, b"-");

    // Month
    let month = sys::format_u64((tm.tm_mon + 1) as u64, &mut buf);
    if month.len() < 2 { io::write_str(1, b"0"); }
    io::write_all(1, month);
    io::write_str(1, b"-");

    // Day
    let day = sys::format_u64(tm.tm_mday as u64, &mut buf);
    if day.len() < 2 { io::write_str(1, b"0"); }
    io::write_all(1, day);
    io::write_str(1, b" ");

    // Hour
    let hour = sys::format_u64(tm.tm_hour as u64, &mut buf);
    if hour.len() < 2 { io::write_str(1, b"0"); }
    io::write_all(1, hour);
    io::write_str(1, b":");

    // Minute
    let min = sys::format_u64(tm.tm_min as u64, &mut buf);
    if min.len() < 2 { io::write_str(1, b"0"); }
    io::write_all(1, min);
    io::write_str(1, b":");

    // Second
    let sec = sys::format_u64(tm.tm_sec as u64, &mut buf);
    if sec.len() < 2 { io::write_str(1, b"0"); }
    io::write_all(1, sec);
}

/// uuidgen - generate UUID
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

/// mcookie - generate magic cookie
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

/// pwgen - generate password
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

/// uuencode - encode binary to ASCII (POSIX compliant)
///
/// POSIX: Converts binary file to ASCII representation using uuencoding.
/// Format: "begin MODE FILENAME", encoded lines (up to 45 bytes per line), "end"
#[cfg(feature = "alloc")]
pub fn uuencode(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    let mut mode = 0o644u32;
    let mut decode_name: Option<&[u8]> = None;
    let mut input_file: Option<&[u8]> = None;
    let mut use_base64 = false;
    let mut i = 1;

    // Parse arguments
    while i < argc as usize {
        // SAFETY: argv is valid for argc elements
        let arg = match unsafe { get_arg(argv, i as i32) } {
            Some(a) => a,
            None => break,
        };

        if arg == b"-m" {
            use_base64 = true;
        } else if decode_name.is_none() {
            decode_name = Some(arg);
        } else if input_file.is_none() {
            input_file = Some(decode_name.unwrap());
            decode_name = Some(arg);
        }
        i += 1;
    }

    let name = match decode_name {
        Some(n) => n,
        None => {
            io::write_str(2, b"uuencode: missing operand\n");
            io::write_str(2, b"Usage: uuencode [-m] [file] name\n");
            return 1;
        }
    };

    // Open input file or use stdin
    let fd = match input_file {
        Some(path) => {
            // Get file mode if it exists
            let mut stat_buf = io::stat_zeroed();
            if io::stat(path, &mut stat_buf) == 0 {
                mode = stat_buf.st_mode & 0o777;
            }
            let fd = io::open(path, libc::O_RDONLY, 0);
            if fd < 0 {
                io::write_str(2, b"uuencode: ");
                io::write_all(2, path);
                io::write_str(2, b": No such file or directory\n");
                return 1;
            }
            fd
        }
        None => 0, // stdin
    };

    // Read all input
    let content = io::read_all(fd);
    if fd > 0 {
        io::close(fd);
    }

    if use_base64 {
        // Base64 encoding (MIME format)
        io::write_str(1, b"begin-base64 ");
        let mut mode_buf = [0u8; 8];
        let mode_str = sys::format_octal(mode, &mut mode_buf);
        io::write_all(1, mode_str);
        io::write_str(1, b" ");
        io::write_all(1, name);
        io::write_str(1, b"\n");

        encode_base64(&content);

        io::write_str(1, b"====\n");
    } else {
        // Traditional uuencoding
        io::write_str(1, b"begin ");
        let mut mode_buf = [0u8; 8];
        let mode_str = sys::format_octal(mode, &mut mode_buf);
        io::write_all(1, mode_str);
        io::write_str(1, b" ");
        io::write_all(1, name);
        io::write_str(1, b"\n");

        encode_uuencode(&content);

        io::write_str(1, b"`\nend\n");
    }

    0
}

#[cfg(not(feature = "alloc"))]
pub fn uuencode(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"uuencode: requires alloc feature\n");
    1
}

/// Encode a byte as uuencode character
#[cfg(feature = "alloc")]
fn uu_encode_char(b: u8) -> u8 {
    if b == 0 {
        b'`' // Backtick for zero
    } else {
        b + 32 // Add space offset
    }
}

/// Encode data using traditional uuencoding
#[cfg(feature = "alloc")]
fn encode_uuencode(data: &[u8]) {
    const LINE_BYTES: usize = 45; // 45 bytes per line (60 encoded chars)

    let mut pos = 0;
    while pos < data.len() {
        let chunk_len = core::cmp::min(LINE_BYTES, data.len() - pos);
        let chunk = &data[pos..pos + chunk_len];

        // Line length character
        io::write_all(1, &[uu_encode_char(chunk_len as u8)]);

        // Encode in groups of 3 bytes -> 4 characters
        let mut i = 0;
        while i < chunk_len {
            let b0 = chunk[i];
            let b1 = if i + 1 < chunk_len { chunk[i + 1] } else { 0 };
            let b2 = if i + 2 < chunk_len { chunk[i + 2] } else { 0 };

            let c0 = (b0 >> 2) & 0x3F;
            let c1 = ((b0 << 4) | (b1 >> 4)) & 0x3F;
            let c2 = ((b1 << 2) | (b2 >> 6)) & 0x3F;
            let c3 = b2 & 0x3F;

            io::write_all(1, &[
                uu_encode_char(c0),
                uu_encode_char(c1),
                uu_encode_char(c2),
                uu_encode_char(c3),
            ]);

            i += 3;
        }

        io::write_str(1, b"\n");
        pos += chunk_len;
    }
}

/// Encode data using base64
#[cfg(feature = "alloc")]
fn encode_base64(data: &[u8]) {
    const BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    const LINE_WIDTH: usize = 76;

    let mut line_pos = 0;
    let mut pos = 0;

    while pos < data.len() {
        let b0 = data[pos];
        let b1 = if pos + 1 < data.len() { data[pos + 1] } else { 0 };
        let b2 = if pos + 2 < data.len() { data[pos + 2] } else { 0 };

        let c0 = BASE64[((b0 >> 2) & 0x3F) as usize];
        let c1 = BASE64[(((b0 << 4) | (b1 >> 4)) & 0x3F) as usize];
        let c2 = if pos + 1 < data.len() {
            BASE64[(((b1 << 2) | (b2 >> 6)) & 0x3F) as usize]
        } else {
            b'='
        };
        let c3 = if pos + 2 < data.len() {
            BASE64[(b2 & 0x3F) as usize]
        } else {
            b'='
        };

        io::write_all(1, &[c0, c1, c2, c3]);
        line_pos += 4;

        if line_pos >= LINE_WIDTH {
            io::write_str(1, b"\n");
            line_pos = 0;
        }

        pos += 3;
    }

    if line_pos > 0 {
        io::write_str(1, b"\n");
    }
}

/// uudecode - decode uuencoded file (POSIX compliant)
///
/// POSIX: Decodes uuencoded (or base64) files back to binary.
/// Parses header for filename and mode, writes decoded output.
#[cfg(feature = "alloc")]
pub fn uudecode(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    let mut output_file: Option<&[u8]> = None;
    let mut input_file: Option<&[u8]> = None;
    let mut i = 1;

    // Parse arguments
    while i < argc as usize {
        // SAFETY: argv is valid for argc elements
        let arg = match unsafe { get_arg(argv, i as i32) } {
            Some(a) => a,
            None => break,
        };

        if arg == b"-o" {
            i += 1;
            output_file = unsafe { get_arg(argv, i as i32) };
        } else if input_file.is_none() {
            input_file = Some(arg);
        }
        i += 1;
    }

    // Open input file or use stdin
    let fd = match input_file {
        Some(path) => {
            let fd = io::open(path, libc::O_RDONLY, 0);
            if fd < 0 {
                io::write_str(2, b"uudecode: ");
                io::write_all(2, path);
                io::write_str(2, b": No such file or directory\n");
                return 1;
            }
            fd
        }
        None => 0,
    };

    // Read all input
    let content = io::read_all(fd);
    if fd > 0 {
        io::close(fd);
    }

    // Find "begin" line
    let mut pos = 0;
    let mut is_base64 = false;
    let mut mode = 0o644u32;
    let mut filename: Vec<u8> = Vec::new();

    // Skip to "begin" or "begin-base64" line
    while pos < content.len() {
        let line_start = pos;
        while pos < content.len() && content[pos] != b'\n' {
            pos += 1;
        }
        let line = &content[line_start..pos];
        if pos < content.len() {
            pos += 1; // Skip newline
        }

        if line.starts_with(b"begin-base64 ") {
            is_base64 = true;
            // Parse mode and filename
            let rest = &line[13..];
            if let Some((m, f)) = parse_begin_line(rest) {
                mode = m;
                filename = f;
            }
            break;
        } else if line.starts_with(b"begin ") {
            // Parse mode and filename
            let rest = &line[6..];
            if let Some((m, f)) = parse_begin_line(rest) {
                mode = m;
                filename = f;
            }
            break;
        }
    }

    if filename.is_empty() {
        io::write_str(2, b"uudecode: no 'begin' line found\n");
        return 1;
    }

    // Use output file override if provided
    let out_name = match output_file {
        Some(name) => name,
        None => &filename,
    };

    // Decode content
    let mut decoded: Vec<u8> = Vec::new();

    if is_base64 {
        // Base64 decoding
        while pos < content.len() {
            let line_start = pos;
            while pos < content.len() && content[pos] != b'\n' {
                pos += 1;
            }
            let line = &content[line_start..pos];
            if pos < content.len() {
                pos += 1;
            }

            if line.starts_with(b"====") {
                break;
            }

            decode_base64_line(line, &mut decoded);
        }
    } else {
        // Traditional uudecoding
        while pos < content.len() {
            let line_start = pos;
            while pos < content.len() && content[pos] != b'\n' {
                pos += 1;
            }
            let line = &content[line_start..pos];
            if pos < content.len() {
                pos += 1;
            }

            if line.is_empty() || line == b"`" || line == b"end" {
                break;
            }

            decode_uuencode_line(line, &mut decoded);
        }
    }

    // Write output
    if out_name == b"-" || out_name == b"/dev/stdout" {
        io::write_all(1, &decoded);
    } else {
        let fd = io::open(out_name, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, mode);
        if fd < 0 {
            io::write_str(2, b"uudecode: ");
            io::write_all(2, out_name);
            io::write_str(2, b": cannot create\n");
            return 1;
        }
        io::write_all(fd, &decoded);
        io::close(fd);
    }

    0
}

#[cfg(not(feature = "alloc"))]
pub fn uudecode(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"uudecode: requires alloc feature\n");
    1
}

/// Parse "MODE FILENAME" from begin line
#[cfg(feature = "alloc")]
fn parse_begin_line(s: &[u8]) -> Option<(u32, alloc::vec::Vec<u8>)> {
    use alloc::vec::Vec;

    let mut pos = 0;
    // Parse octal mode
    let mut mode = 0u32;
    while pos < s.len() && s[pos] >= b'0' && s[pos] <= b'7' {
        mode = mode * 8 + (s[pos] - b'0') as u32;
        pos += 1;
    }

    // Skip space
    while pos < s.len() && s[pos] == b' ' {
        pos += 1;
    }

    // Rest is filename
    let filename: Vec<u8> = s[pos..].to_vec();
    if filename.is_empty() {
        return None;
    }

    Some((mode, filename))
}

/// Decode uuencode character
#[cfg(feature = "alloc")]
fn uu_decode_char(c: u8) -> u8 {
    if c == b'`' || c == b' ' {
        0
    } else {
        (c - 32) & 0x3F
    }
}

/// Decode a single uuencode line
#[cfg(feature = "alloc")]
fn decode_uuencode_line(line: &[u8], output: &mut alloc::vec::Vec<u8>) {
    if line.is_empty() {
        return;
    }

    let len = uu_decode_char(line[0]) as usize;
    if len == 0 || line.len() < 2 {
        return;
    }

    let mut pos = 1;
    let mut decoded = 0;

    while decoded < len && pos + 3 < line.len() {
        let c0 = uu_decode_char(line[pos]);
        let c1 = uu_decode_char(line[pos + 1]);
        let c2 = uu_decode_char(line[pos + 2]);
        let c3 = uu_decode_char(line[pos + 3]);

        let b0 = (c0 << 2) | (c1 >> 4);
        let b1 = (c1 << 4) | (c2 >> 2);
        let b2 = (c2 << 6) | c3;

        if decoded < len {
            output.push(b0);
            decoded += 1;
        }
        if decoded < len {
            output.push(b1);
            decoded += 1;
        }
        if decoded < len {
            output.push(b2);
            decoded += 1;
        }

        pos += 4;
    }
}

/// Decode base64 character
#[cfg(feature = "alloc")]
fn base64_decode_char(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        b'=' => None, // Padding
        _ => None,
    }
}

/// Decode a single base64 line
#[cfg(feature = "alloc")]
fn decode_base64_line(line: &[u8], output: &mut alloc::vec::Vec<u8>) {
    let mut pos = 0;

    while pos + 3 < line.len() {
        let c0 = match base64_decode_char(line[pos]) {
            Some(v) => v,
            None => { pos += 1; continue; }
        };
        let c1 = match base64_decode_char(line[pos + 1]) {
            Some(v) => v,
            None => break,
        };

        output.push((c0 << 2) | (c1 >> 4));

        if line[pos + 2] != b'=' {
            if let Some(c2) = base64_decode_char(line[pos + 2]) {
                output.push((c1 << 4) | (c2 >> 2));

                if line[pos + 3] != b'=' {
                    if let Some(c3) = base64_decode_char(line[pos + 3]) {
                        output.push((c2 << 6) | c3);
                    }
                }
            }
        }

        pos += 4;
    }
}

/// help - show help
pub fn help(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"armybox - BusyBox/Toybox compatible multi-call binary\n");
    io::write_str(1, b"Usage: armybox [APPLET] [ARGS]\n");
    0
}

/// memeater - memory stress test
///
/// Usage: memeater [size_in_MB]
/// Allocates and touches memory to stress test the system.
#[cfg(feature = "alloc")]
pub fn memeater(argc: i32, argv: *const *const u8) -> i32 {
    use alloc::vec::Vec;

    let size_mb = if argc > 1 {
        match unsafe { get_arg(argv, 1) }.and_then(|s| sys::parse_u64(s)) {
            Some(n) => n as usize,
            None => 100,
        }
    } else {
        100 // Default 100MB
    };

    io::write_str(1, b"memeater: allocating ");
    let mut buf = [0u8; 16];
    let size_str = sys::format_u64(size_mb as u64, &mut buf);
    io::write_all(1, size_str);
    io::write_str(1, b" MB...\n");

    let total_bytes = size_mb * 1024 * 1024;
    let mut memory: Vec<u8> = Vec::with_capacity(total_bytes);

    // Fill memory to ensure it's actually allocated
    for i in 0..total_bytes {
        memory.push((i & 0xFF) as u8);
    }

    io::write_str(1, b"memeater: allocated. Press Ctrl+C to exit.\n");

    // Keep the memory allocated until interrupted
    loop {
        // Touch memory periodically to prevent it from being swapped out
        for i in (0..memory.len()).step_by(4096) {
            let _ = memory[i];
        }
        unsafe {
            libc::sleep(1);
        }
    }
}

#[cfg(not(feature = "alloc"))]
pub fn memeater(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"memeater: requires alloc feature\n");
    1
}

/// mix - audio mixer interface
///
/// Usage: mix [channel] [volume]
/// Interacts with ALSA/OSS audio mixer.
/// Note: Full ALSA support requires libasound. This provides basic OSS compatibility.
#[cfg(target_os = "linux")]
pub fn mix(argc: i32, argv: *const *const u8) -> i32 {
    const SOUND_MIXER_VOLUME: libc::c_ulong = 0x80044D00;
    const SOUND_MIXER_READ_VOLUME: libc::c_ulong = 0x80044D00;
    const SOUND_MIXER_WRITE_VOLUME: libc::c_ulong = 0xC0044D00;

    // Try to open OSS mixer
    let fd = io::open(b"/dev/mixer", libc::O_RDWR, 0);
    if fd < 0 {
        io::write_str(2, b"mix: cannot open /dev/mixer\n");
        io::write_str(2, b"Note: This system may use ALSA instead of OSS.\n");
        io::write_str(2, b"Try: amixer or alsamixer instead.\n");
        return 1;
    }

    if argc < 2 {
        // Show current volume
        let mut vol: i32 = 0;
        let ret = unsafe {
            libc::ioctl(fd, SOUND_MIXER_READ_VOLUME, &mut vol as *mut i32)
        };
        io::close(fd);

        if ret < 0 {
            io::write_str(2, b"mix: cannot read volume\n");
            return 1;
        }

        let left = vol & 0xFF;
        let right = (vol >> 8) & 0xFF;

        io::write_str(1, b"Master volume: ");
        let mut buf = [0u8; 8];
        let l = sys::format_u64(left as u64, &mut buf);
        io::write_all(1, l);
        io::write_str(1, b"% / ");
        let r = sys::format_u64(right as u64, &mut buf);
        io::write_all(1, r);
        io::write_str(1, b"%\n");
    } else {
        // Set volume
        let vol_arg = match unsafe { get_arg(argv, 1) } {
            Some(v) => v,
            None => {
                io::close(fd);
                return 1;
            }
        };

        let volume = match sys::parse_u64(vol_arg) {
            Some(v) => core::cmp::min(v, 100) as i32,
            None => {
                io::write_str(2, b"mix: invalid volume\n");
                io::close(fd);
                return 1;
            }
        };

        let vol = volume | (volume << 8); // Same for left and right

        let ret = unsafe {
            libc::ioctl(fd, SOUND_MIXER_WRITE_VOLUME, &vol as *const i32)
        };
        io::close(fd);

        if ret < 0 {
            io::write_str(2, b"mix: cannot set volume\n");
            return 1;
        }

        io::write_str(1, b"Volume set to ");
        let mut buf = [0u8; 8];
        let v = sys::format_u64(volume as u64, &mut buf);
        io::write_all(1, v);
        io::write_str(1, b"%\n");
    }

    0
}

#[cfg(not(target_os = "linux"))]
pub fn mix(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(2, b"mix: only available on Linux\n");
    1
}

/// mkpasswd - generate password hash
///
/// Usage: mkpasswd [-m method] [-S salt] [password]
/// Generates a password hash. Currently outputs a simple hash format.
/// For production use, use the system's mkpasswd or openssl.
pub fn mkpasswd(argc: i32, argv: *const *const u8) -> i32 {
    let mut method = b"sha512" as &[u8];
    let mut salt: Option<&[u8]> = None;
    let mut password: Option<&[u8]> = None;
    let mut i = 1;

    // Parse arguments
    while i < argc as usize {
        let arg = match unsafe { get_arg(argv, i as i32) } {
            Some(a) => a,
            None => break,
        };

        if arg == b"-m" || arg == b"--method" {
            i += 1;
            if let Some(m) = unsafe { get_arg(argv, i as i32) } {
                method = m;
            }
        } else if arg == b"-S" || arg == b"--salt" {
            i += 1;
            salt = unsafe { get_arg(argv, i as i32) };
        } else if !arg.starts_with(b"-") {
            password = Some(arg);
        }
        i += 1;
    }

    let salt_str = salt.unwrap_or(b"randomsalt");
    let pwd = password.unwrap_or(b"password");

    // Generate a simple hash (not cryptographically secure - for demo purposes)
    // In real usage, users should use system mkpasswd or openssl passwd
    let prefix: &[u8] = match method {
        b"md5" | b"1" => b"$1$",
        b"sha256" | b"5" => b"$5$",
        b"sha512" | b"6" => b"$6$",
        _ => b"$6$",
    };

    // Simple hash: XOR password bytes with salt, then format as hex
    let mut hash: u64 = 0x5381; // djb2 seed
    for &b in pwd {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    for &b in salt_str {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }

    io::write_all(1, prefix);
    io::write_all(1, salt_str);
    io::write_str(1, b"$");

    // Output hash as base64-ish characters (like crypt output)
    const B64: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut hash_chars = [0u8; 86];
    let mut h = hash;
    for i in 0..86 {
        hash_chars[i] = B64[(h & 0x3F) as usize];
        h = h.wrapping_mul(0x5DEECE66D).wrapping_add(11);
    }
    io::write_all(1, &hash_chars);
    io::write_str(1, b"\n");

    0
}

/// toybox - toybox compatibility
pub fn toybox(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"armybox (toybox compatible)\n");
    0
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() { return release; }
        manifest_dir.join("target/debug/armybox")
    }

    #[test]
    fn test_help() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["help"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("armybox"));
    }

    #[test]
    fn test_toybox() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["toybox"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_uuidgen() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["uuidgen"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("-"));
    }

    #[test]
    fn test_pwgen() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["pwgen"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.trim().len() >= 8);
    }
}
