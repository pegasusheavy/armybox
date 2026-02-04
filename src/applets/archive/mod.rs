//! Archive utilities

use alloc::vec::Vec;
use alloc::vec;
use crate::io;
use super::{get_arg, has_opt};

// Helper functions for file operations
fn open_read(path: &[u8]) -> i32 {
    io::open(path, libc::O_RDONLY, 0)
}

fn open_write_create(path: &[u8], mode: i32) -> i32 {
    io::open(path, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, mode as u32)
}

/// USTAR tar header structure (512 bytes)
#[repr(C)]
struct TarHeader {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    checksum: [u8; 8],
    typeflag: u8,
    linkname: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    uname: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    prefix: [u8; 155],
    _pad: [u8; 12],
}

impl TarHeader {
    fn new() -> Self {
        Self {
            name: [0; 100],
            mode: [0; 8],
            uid: [0; 8],
            gid: [0; 8],
            size: [0; 12],
            mtime: [0; 12],
            checksum: [0; 8],
            typeflag: 0,
            linkname: [0; 100],
            magic: [0; 6],
            version: [0; 2],
            uname: [0; 32],
            gname: [0; 32],
            devmajor: [0; 8],
            devminor: [0; 8],
            prefix: [0; 155],
            _pad: [0; 12],
        }
    }

    fn is_ustar(&self) -> bool {
        &self.magic[..5] == b"ustar"
    }

    fn is_empty(&self) -> bool {
        self.name[0] == 0
    }

    fn get_name(&self) -> &[u8] {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(100);
        &self.name[..len]
    }

    fn get_size(&self) -> u64 {
        parse_octal(&self.size)
    }

    fn get_mode(&self) -> u32 {
        parse_octal(&self.mode) as u32
    }

    fn compute_checksum(&self) -> u32 {
        let bytes = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, 512)
        };
        let mut sum = 0u32;
        for (i, &b) in bytes.iter().enumerate() {
            // Treat checksum field (bytes 148-155) as spaces
            if i >= 148 && i < 156 {
                sum += b' ' as u32;
            } else {
                sum += b as u32;
            }
        }
        sum
    }

    fn set_checksum(&mut self) {
        // Fill with spaces first
        self.checksum = *b"        ";
        let sum = self.compute_checksum();
        // Write checksum as octal with null terminator and space
        write_octal(&mut self.checksum[..6], sum as u64);
        self.checksum[6] = 0;
        self.checksum[7] = b' ';
    }
}

fn parse_octal(bytes: &[u8]) -> u64 {
    let mut result = 0u64;
    for &b in bytes {
        if b == 0 || b == b' ' { break; }
        if b >= b'0' && b <= b'7' {
            result = result * 8 + (b - b'0') as u64;
        }
    }
    result
}

fn write_octal(buf: &mut [u8], mut val: u64) {
    let len = buf.len();
    for i in (0..len).rev() {
        buf[i] = b'0' + (val & 7) as u8;
        val >>= 3;
    }
}

pub fn tar(argc: i32, argv: *const *const u8) -> i32 {
    let mut create = false;
    let mut extract = false;
    let mut list = false;
    let mut verbose = false;
    let mut archive_file: Option<&[u8]> = None;
    let mut files: Vec<&[u8]> = Vec::new();

    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.starts_with(b"-") || (i == 1 && !arg.starts_with(b"/")) {
                // Options (possibly without leading dash for first arg)
                let opts = if arg.starts_with(b"-") { &arg[1..] } else { arg };
                for &c in opts {
                    match c {
                        b'c' => create = true,
                        b'x' => extract = true,
                        b't' => list = true,
                        b'v' => verbose = true,
                        b'f' => {
                            // Next arg is the archive file
                            i += 1;
                            if let Some(f) = unsafe { get_arg(argv, i) } {
                                archive_file = Some(f);
                            }
                        }
                        b'z' | b'j' | b'J' => {
                            // Compression flags - we'll handle uncompressed only for now
                        }
                        _ => {}
                    }
                }
            } else {
                files.push(arg);
            }
        }
        i += 1;
    }

    let archive = match archive_file {
        Some(f) => f,
        None => {
            io::write_str(2, b"tar: -f archive required\n");
            return 1;
        }
    };

    if create {
        tar_create(archive, &files, verbose)
    } else if extract {
        tar_extract(archive, &files, verbose)
    } else if list {
        tar_list(archive, verbose)
    } else {
        io::write_str(2, b"tar: specify -c, -x, or -t\n");
        1
    }
}

fn tar_create(archive: &[u8], files: &[&[u8]], verbose: bool) -> i32 {
    let fd = open_write_create(archive, 0o644);
    if fd < 0 {
        io::write_str(2, b"tar: cannot create archive\n");
        return 1;
    }

    for &file in files {
        if tar_add_file(fd, file, verbose) != 0 {
            io::close(fd);
            return 1;
        }
    }

    // Write two empty blocks as end marker
    let empty = [0u8; 512];
    io::write_all(fd, &empty);
    io::write_all(fd, &empty);

    io::close(fd);
    0
}

fn tar_add_file(fd: i32, path: &[u8], verbose: bool) -> i32 {
    // Get file info
    let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
    let mut path_z = [0u8; 256];
    let len = path.len().min(255);
    path_z[..len].copy_from_slice(&path[..len]);

    if unsafe { libc::stat(path_z.as_ptr() as *const i8, &mut stat_buf) } != 0 {
        io::write_str(2, b"tar: cannot stat ");
        io::write_all(2, path);
        io::write_str(2, b"\n");
        return 1;
    }

    let mode = stat_buf.st_mode;
    let is_dir = (mode & libc::S_IFMT) == libc::S_IFDIR;
    let is_link = (mode & libc::S_IFMT) == libc::S_IFLNK;

    if is_dir {
        // Add directory and recurse
        tar_add_dir_entry(fd, path, &stat_buf, verbose);
        // Recursively add directory contents
        let dir = unsafe { libc::opendir(path_z.as_ptr() as *const i8) };
        if !dir.is_null() {
            loop {
                let entry = unsafe { libc::readdir(dir) };
                if entry.is_null() { break; }
                let name = unsafe { io::cstr_to_slice((*entry).d_name.as_ptr() as *const u8) };
                if name == b"." || name == b".." { continue; }

                // Build full path
                let mut full_path = Vec::new();
                full_path.extend_from_slice(path);
                if !path.ends_with(b"/") {
                    full_path.push(b'/');
                }
                full_path.extend_from_slice(name);

                tar_add_file(fd, &full_path, verbose);
            }
            unsafe { libc::closedir(dir) };
        }
    } else if is_link {
        tar_add_link_entry(fd, path, &stat_buf, verbose);
    } else {
        tar_add_file_entry(fd, path, &stat_buf, verbose);
    }

    0
}

fn tar_add_dir_entry(fd: i32, path: &[u8], stat: &libc::stat, verbose: bool) {
    let mut header = TarHeader::new();

    // Name (with trailing slash for directories)
    let len = path.len().min(99);
    header.name[..len].copy_from_slice(&path[..len]);
    if len < 99 && !path.ends_with(b"/") {
        header.name[len] = b'/';
    }

    // Mode
    write_octal(&mut header.mode[..7], (stat.st_mode & 0o7777) as u64);

    // UID/GID
    write_octal(&mut header.uid[..7], stat.st_uid as u64);
    write_octal(&mut header.gid[..7], stat.st_gid as u64);

    // Size (0 for directories)
    write_octal(&mut header.size[..11], 0);

    // Mtime
    write_octal(&mut header.mtime[..11], stat.st_mtime as u64);

    // Type flag - directory
    header.typeflag = b'5';

    // USTAR magic
    header.magic = *b"ustar ";
    header.version = *b" \0";

    header.set_checksum();

    let header_bytes = unsafe {
        core::slice::from_raw_parts(&header as *const _ as *const u8, 512)
    };
    io::write_all(fd, header_bytes);

    if verbose {
        io::write_all(1, path);
        io::write_str(1, b"/\n");
    }
}

fn tar_add_link_entry(fd: i32, path: &[u8], stat: &libc::stat, verbose: bool) {
    let mut header = TarHeader::new();

    // Name
    let len = path.len().min(100);
    header.name[..len].copy_from_slice(&path[..len]);

    // Mode
    write_octal(&mut header.mode[..7], (stat.st_mode & 0o7777) as u64);

    // UID/GID
    write_octal(&mut header.uid[..7], stat.st_uid as u64);
    write_octal(&mut header.gid[..7], stat.st_gid as u64);

    // Size (0 for symlinks)
    write_octal(&mut header.size[..11], 0);

    // Mtime
    write_octal(&mut header.mtime[..11], stat.st_mtime as u64);

    // Read link target
    let mut path_z = [0u8; 256];
    let plen = path.len().min(255);
    path_z[..plen].copy_from_slice(&path[..plen]);

    let mut linkbuf = [0u8; 100];
    let link_len = unsafe {
        libc::readlink(path_z.as_ptr() as *const i8, linkbuf.as_mut_ptr() as *mut i8, 100)
    };
    if link_len > 0 {
        header.linkname[..link_len as usize].copy_from_slice(&linkbuf[..link_len as usize]);
    }

    // Type flag - symlink
    header.typeflag = b'2';

    // USTAR magic
    header.magic = *b"ustar ";
    header.version = *b" \0";

    header.set_checksum();

    let header_bytes = unsafe {
        core::slice::from_raw_parts(&header as *const _ as *const u8, 512)
    };
    io::write_all(fd, header_bytes);

    if verbose {
        io::write_all(1, path);
        io::write_str(1, b"\n");
    }
}

fn tar_add_file_entry(fd: i32, path: &[u8], stat: &libc::stat, verbose: bool) {
    let mut header = TarHeader::new();

    // Name
    let len = path.len().min(100);
    header.name[..len].copy_from_slice(&path[..len]);

    // Mode
    write_octal(&mut header.mode[..7], (stat.st_mode & 0o7777) as u64);

    // UID/GID
    write_octal(&mut header.uid[..7], stat.st_uid as u64);
    write_octal(&mut header.gid[..7], stat.st_gid as u64);

    // Size
    let size = stat.st_size as u64;
    write_octal(&mut header.size[..11], size);

    // Mtime
    write_octal(&mut header.mtime[..11], stat.st_mtime as u64);

    // Type flag - regular file
    header.typeflag = b'0';

    // USTAR magic
    header.magic = *b"ustar ";
    header.version = *b" \0";

    header.set_checksum();

    let header_bytes = unsafe {
        core::slice::from_raw_parts(&header as *const _ as *const u8, 512)
    };
    io::write_all(fd, header_bytes);

    // Write file content
    let mut path_z = [0u8; 256];
    let plen = path.len().min(255);
    path_z[..plen].copy_from_slice(&path[..plen]);

    let file_fd = open_read(&path_z);
    if file_fd >= 0 {
        let mut buf = [0u8; 512];
        let mut remaining = size;
        while remaining > 0 {
            let to_read = remaining.min(512) as usize;
            let n = io::read(file_fd, &mut buf[..to_read]);
            if n <= 0 { break; }
            // Pad last block to 512 bytes
            if (n as usize) < 512 {
                for i in (n as usize)..512 {
                    buf[i] = 0;
                }
            }
            io::write_all(fd, &buf);
            remaining -= n as u64;
        }
        io::close(file_fd);
    }

    if verbose {
        io::write_all(1, path);
        io::write_str(1, b"\n");
    }
}

fn tar_extract(archive: &[u8], files: &[&[u8]], verbose: bool) -> i32 {
    let fd = open_read(archive);
    if fd < 0 {
        io::write_str(2, b"tar: cannot open archive\n");
        return 1;
    }

    let mut header_buf = [0u8; 512];

    loop {
        if io::read(fd, &mut header_buf) != 512 {
            break;
        }

        let header = unsafe { &*(header_buf.as_ptr() as *const TarHeader) };

        if header.is_empty() {
            break;
        }

        let name = header.get_name();
        let size = header.get_size();
        let mode = header.get_mode();

        // Check if this file matches the filter (if any)
        let should_extract = files.is_empty() || files.iter().any(|&f| {
            name.starts_with(f) || f == name
        });

        if should_extract {
            match header.typeflag {
                b'5' | b'\0' if name.ends_with(b"/") => {
                    // Directory
                    let mut path_z = [0u8; 256];
                    let len = name.len().min(255);
                    path_z[..len].copy_from_slice(&name[..len]);
                    unsafe { libc::mkdir(path_z.as_ptr() as *const i8, mode) };
                    if verbose {
                        io::write_all(1, name);
                        io::write_str(1, b"\n");
                    }
                }
                b'2' => {
                    // Symlink
                    let target = {
                        let len = header.linkname.iter().position(|&c| c == 0).unwrap_or(100);
                        &header.linkname[..len]
                    };
                    let mut name_z = [0u8; 256];
                    let mut target_z = [0u8; 256];
                    let nlen = name.len().min(255);
                    let tlen = target.len().min(255);
                    name_z[..nlen].copy_from_slice(&name[..nlen]);
                    target_z[..tlen].copy_from_slice(&target[..tlen]);
                    unsafe { libc::symlink(target_z.as_ptr() as *const i8, name_z.as_ptr() as *const i8) };
                    if verbose {
                        io::write_all(1, name);
                        io::write_str(1, b"\n");
                    }
                }
                b'0' | b'\0' => {
                    // Regular file
                    extract_file(name, fd, size, mode, verbose);
                }
                _ => {
                    // Skip unknown types
                }
            }
        }

        // Skip file content blocks
        if size > 0 && (header.typeflag == b'0' || header.typeflag == 0) {
            let blocks = (size + 511) / 512;
            if !should_extract {
                for _ in 0..blocks {
                    io::read(fd, &mut header_buf);
                }
            }
        }
    }

    io::close(fd);
    0
}

fn extract_file(name: &[u8], archive_fd: i32, size: u64, mode: u32, verbose: bool) {
    // Create parent directories if needed
    create_parent_dirs(name);

    let mut path_z = [0u8; 256];
    let len = name.len().min(255);
    path_z[..len].copy_from_slice(&name[..len]);

    let fd = open_write_create(&path_z, mode as i32);
    if fd < 0 {
        io::write_str(2, b"tar: cannot create ");
        io::write_all(2, name);
        io::write_str(2, b"\n");
        // Still need to skip the blocks
        let blocks = (size + 511) / 512;
        let mut skip_buf = [0u8; 512];
        for _ in 0..blocks {
            io::read(archive_fd, &mut skip_buf);
        }
        return;
    }

    let mut remaining = size;
    let mut buf = [0u8; 512];
    while remaining > 0 {
        if io::read(archive_fd, &mut buf) != 512 {
            break;
        }
        let to_write = remaining.min(512) as usize;
        io::write_all(fd, &buf[..to_write]);
        remaining -= to_write as u64;
    }

    io::close(fd);

    if verbose {
        io::write_all(1, name);
        io::write_str(1, b"\n");
    }
}

fn create_parent_dirs(path: &[u8]) {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path[..len]);

    // Find each / and create directories
    for i in 0..len {
        if buf[i] == b'/' {
            buf[i] = 0;
            unsafe { libc::mkdir(buf.as_ptr() as *const i8, 0o755) };
            buf[i] = b'/';
        }
    }
}

fn tar_list(archive: &[u8], verbose: bool) -> i32 {
    let fd = open_read(archive);
    if fd < 0 {
        io::write_str(2, b"tar: cannot open archive\n");
        return 1;
    }

    let mut header_buf = [0u8; 512];

    loop {
        if io::read(fd, &mut header_buf) != 512 {
            break;
        }

        let header = unsafe { &*(header_buf.as_ptr() as *const TarHeader) };

        if header.is_empty() {
            break;
        }

        let name = header.get_name();
        let size = header.get_size();

        if verbose {
            // Show mode, size, name
            let mode = header.get_mode();
            let type_char = match header.typeflag {
                b'5' => b'd',
                b'2' => b'l',
                _ => b'-',
            };
            io::write_all(1, &[type_char]);
            io::write_all(1, &mode_string(mode));
            io::write_str(1, b" ");
            io::write_num(1, size);
            io::write_str(1, b" ");
        }
        io::write_all(1, name);
        io::write_str(1, b"\n");

        // Skip file content blocks
        if size > 0 && (header.typeflag == b'0' || header.typeflag == 0) {
            let blocks = (size + 511) / 512;
            for _ in 0..blocks {
                io::read(fd, &mut header_buf);
            }
        }
    }

    io::close(fd);
    0
}

fn mode_string(mode: u32) -> [u8; 9] {
    let mut s = [b'-'; 9];
    if mode & 0o400 != 0 { s[0] = b'r'; }
    if mode & 0o200 != 0 { s[1] = b'w'; }
    if mode & 0o100 != 0 { s[2] = b'x'; }
    if mode & 0o040 != 0 { s[3] = b'r'; }
    if mode & 0o020 != 0 { s[4] = b'w'; }
    if mode & 0o010 != 0 { s[5] = b'x'; }
    if mode & 0o004 != 0 { s[6] = b'r'; }
    if mode & 0o002 != 0 { s[7] = b'w'; }
    if mode & 0o001 != 0 { s[8] = b'x'; }
    s
}

// ============= GZIP (DEFLATE) =============

// DEFLATE fixed Huffman codes - simplified implementation
// For a full implementation, we'd need dynamic Huffman coding

pub fn gzip(argc: i32, argv: *const *const u8) -> i32 {
    let mut decompress = false;
    let mut keep = false;
    let mut stdout_mode = false;
    let mut files: Vec<&[u8]> = Vec::new();

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.starts_with(b"-") {
                for &c in &arg[1..] {
                    match c {
                        b'd' => decompress = true,
                        b'k' => keep = true,
                        b'c' => stdout_mode = true,
                        _ => {}
                    }
                }
            } else {
                files.push(arg);
            }
        }
    }

    if files.is_empty() {
        // Read from stdin, write to stdout
        if decompress {
            gunzip_stream(0, 1)
        } else {
            gzip_stream(0, 1)
        }
    } else {
        for &file in &files {
            if stdout_mode {
                let fd = open_read(file);
                if fd < 0 {
                    io::write_str(2, b"gzip: cannot open file\n");
                    return 1;
                }
                let result = if decompress {
                    gunzip_stream(fd, 1)
                } else {
                    gzip_stream(fd, 1)
                };
                io::close(fd);
                if result != 0 { return result; }
            } else {
                if decompress {
                    if gunzip_file(file, keep) != 0 { return 1; }
                } else {
                    if gzip_file(file, keep) != 0 { return 1; }
                }
            }
        }
        0
    }
}

pub fn gunzip(argc: i32, argv: *const *const u8) -> i32 {
    // gunzip is gzip -d
    let mut new_argv: Vec<*const u8> = Vec::new();
    new_argv.push(b"gunzip\0".as_ptr());
    new_argv.push(b"-d\0".as_ptr());
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            new_argv.push(arg.as_ptr());
        }
    }
    gzip(new_argv.len() as i32, new_argv.as_ptr())
}

pub fn zcat(argc: i32, argv: *const *const u8) -> i32 {
    // zcat is gzip -dc
    let mut new_argv: Vec<*const u8> = Vec::new();
    new_argv.push(b"zcat\0".as_ptr());
    new_argv.push(b"-dc\0".as_ptr());
    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            new_argv.push(arg.as_ptr());
        }
    }
    gzip(new_argv.len() as i32, new_argv.as_ptr())
}

fn gzip_stream(input_fd: i32, output_fd: i32) -> i32 {
    // Read all input
    let mut data = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(input_fd, &mut buf);
        if n <= 0 { break; }
        data.extend_from_slice(&buf[..n as usize]);
    }

    // Compute CRC32
    let crc = crc32(&data);
    let size = data.len() as u32;

    // Write gzip header
    let header = [
        0x1f, 0x8b,  // Magic
        0x08,        // Compression method (deflate)
        0x00,        // Flags
        0, 0, 0, 0,  // Mtime
        0x00,        // Extra flags
        0xff,        // OS (unknown)
    ];
    io::write_all(output_fd, &header);

    // Write DEFLATE compressed data (using stored blocks for simplicity)
    deflate_stored(&data, output_fd);

    // Write CRC32 and original size
    io::write_all(output_fd, &crc.to_le_bytes());
    io::write_all(output_fd, &size.to_le_bytes());

    0
}

fn deflate_stored(data: &[u8], fd: i32) {
    // Use stored blocks (no compression) - valid DEFLATE but not efficient
    // This is a simplified implementation
    let mut offset = 0;
    while offset < data.len() {
        let remaining = data.len() - offset;
        let block_size = remaining.min(65535);
        let is_final = offset + block_size >= data.len();

        // Block header: BFINAL (1 bit) + BTYPE=00 (2 bits) = stored block
        let header_byte = if is_final { 0x01 } else { 0x00 };
        io::write_all(fd, &[header_byte]);

        // LEN and NLEN (little-endian)
        let len = block_size as u16;
        let nlen = !len;
        io::write_all(fd, &len.to_le_bytes());
        io::write_all(fd, &nlen.to_le_bytes());

        // Data
        io::write_all(fd, &data[offset..offset + block_size]);

        offset += block_size;
    }
}

fn gunzip_stream(input_fd: i32, output_fd: i32) -> i32 {
    // Read gzip header
    let mut header = [0u8; 10];
    if io::read(input_fd, &mut header) != 10 {
        io::write_str(2, b"gzip: truncated header\n");
        return 1;
    }

    // Verify magic
    if header[0] != 0x1f || header[1] != 0x8b {
        io::write_str(2, b"gzip: not gzip format\n");
        return 1;
    }

    // Verify compression method is deflate
    if header[2] != 0x08 {
        io::write_str(2, b"gzip: unsupported compression method\n");
        return 1;
    }

    let flags = header[3];

    // Skip optional fields
    if flags & 0x04 != 0 {
        // FEXTRA
        let mut len_buf = [0u8; 2];
        io::read(input_fd, &mut len_buf);
        let len = u16::from_le_bytes(len_buf) as usize;
        let mut skip = vec![0u8; len];
        io::read(input_fd, &mut skip);
    }
    if flags & 0x08 != 0 {
        // FNAME - skip null-terminated string
        let mut b = [0u8; 1];
        loop {
            io::read(input_fd, &mut b);
            if b[0] == 0 { break; }
        }
    }
    if flags & 0x10 != 0 {
        // FCOMMENT - skip null-terminated string
        let mut b = [0u8; 1];
        loop {
            io::read(input_fd, &mut b);
            if b[0] == 0 { break; }
        }
    }
    if flags & 0x02 != 0 {
        // FHCRC
        let mut crc16 = [0u8; 2];
        io::read(input_fd, &mut crc16);
    }

    // Read remaining compressed data
    let mut compressed = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(input_fd, &mut buf);
        if n <= 0 { break; }
        compressed.extend_from_slice(&buf[..n as usize]);
    }

    // The last 8 bytes are CRC32 and original size
    if compressed.len() < 8 {
        io::write_str(2, b"gzip: truncated file\n");
        return 1;
    }

    let trailer_start = compressed.len() - 8;
    let _expected_crc = u32::from_le_bytes([
        compressed[trailer_start],
        compressed[trailer_start + 1],
        compressed[trailer_start + 2],
        compressed[trailer_start + 3],
    ]);
    let _expected_size = u32::from_le_bytes([
        compressed[trailer_start + 4],
        compressed[trailer_start + 5],
        compressed[trailer_start + 6],
        compressed[trailer_start + 7],
    ]);

    compressed.truncate(trailer_start);

    // Decompress DEFLATE data
    let decompressed = inflate(&compressed);

    io::write_all(output_fd, &decompressed);

    0
}

fn gzip_file(path: &[u8], keep: bool) -> i32 {
    let fd = open_read(path);
    if fd < 0 {
        io::write_str(2, b"gzip: cannot open ");
        io::write_all(2, path);
        io::write_str(2, b"\n");
        return 1;
    }

    // Create output path with .gz extension
    let mut out_path = Vec::new();
    out_path.extend_from_slice(path);
    out_path.extend_from_slice(b".gz\0");

    let out_fd = open_write_create(&out_path, 0o644);
    if out_fd < 0 {
        io::write_str(2, b"gzip: cannot create output\n");
        io::close(fd);
        return 1;
    }

    let result = gzip_stream(fd, out_fd);

    io::close(fd);
    io::close(out_fd);

    if result == 0 && !keep {
        let mut path_z = [0u8; 256];
        let len = path.len().min(255);
        path_z[..len].copy_from_slice(&path[..len]);
        unsafe { libc::unlink(path_z.as_ptr() as *const i8) };
    }

    result
}

fn gunzip_file(path: &[u8], keep: bool) -> i32 {
    let fd = open_read(path);
    if fd < 0 {
        io::write_str(2, b"gzip: cannot open ");
        io::write_all(2, path);
        io::write_str(2, b"\n");
        return 1;
    }

    // Create output path without .gz extension
    let mut out_path = Vec::new();
    if path.ends_with(b".gz") {
        out_path.extend_from_slice(&path[..path.len() - 3]);
    } else {
        out_path.extend_from_slice(path);
        out_path.extend_from_slice(b".out");
    }
    out_path.push(0);

    let out_fd = open_write_create(&out_path, 0o644);
    if out_fd < 0 {
        io::write_str(2, b"gzip: cannot create output\n");
        io::close(fd);
        return 1;
    }

    let result = gunzip_stream(fd, out_fd);

    io::close(fd);
    io::close(out_fd);

    if result == 0 && !keep {
        let mut path_z = [0u8; 256];
        let len = path.len().min(255);
        path_z[..len].copy_from_slice(&path[..len]);
        unsafe { libc::unlink(path_z.as_ptr() as *const i8) };
    }

    result
}

// DEFLATE decompression (inflate)
fn inflate(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut bit_pos = 0usize;

    fn get_bits(data: &[u8], bit_pos: &mut usize, count: usize) -> u32 {
        let mut result = 0u32;
        for i in 0..count {
            let byte_idx = *bit_pos / 8;
            let bit_idx = *bit_pos % 8;
            if byte_idx < data.len() {
                if data[byte_idx] & (1 << bit_idx) != 0 {
                    result |= 1 << i;
                }
            }
            *bit_pos += 1;
        }
        result
    }

    loop {
        let bfinal = get_bits(data, &mut bit_pos, 1);
        let btype = get_bits(data, &mut bit_pos, 2);

        match btype {
            0 => {
                // Stored block
                // Skip to byte boundary
                bit_pos = (bit_pos + 7) & !7;
                let len = get_bits(data, &mut bit_pos, 16) as usize;
                let _nlen = get_bits(data, &mut bit_pos, 16);

                let byte_pos = bit_pos / 8;
                if byte_pos + len <= data.len() {
                    output.extend_from_slice(&data[byte_pos..byte_pos + len]);
                }
                bit_pos += len * 8;
            }
            1 => {
                // Fixed Huffman
                inflate_fixed_huffman(data, &mut bit_pos, &mut output);
            }
            2 => {
                // Dynamic Huffman
                inflate_dynamic_huffman(data, &mut bit_pos, &mut output);
            }
            _ => {
                // Invalid block type
                break;
            }
        }

        if bfinal != 0 {
            break;
        }
    }

    output
}

fn inflate_fixed_huffman(data: &[u8], bit_pos: &mut usize, output: &mut Vec<u8>) {
    // Fixed Huffman code lengths:
    // 0-143: 8 bits, 144-255: 9 bits, 256-279: 7 bits, 280-287: 8 bits

    fn get_bits(data: &[u8], bit_pos: &mut usize, count: usize) -> u32 {
        let mut result = 0u32;
        for i in 0..count {
            let byte_idx = *bit_pos / 8;
            let bit_idx = *bit_pos % 8;
            if byte_idx < data.len() {
                if data[byte_idx] & (1 << bit_idx) != 0 {
                    result |= 1 << i;
                }
            }
            *bit_pos += 1;
        }
        result
    }

    fn get_bits_rev(data: &[u8], bit_pos: &mut usize, count: usize) -> u32 {
        let mut result = 0u32;
        for _ in 0..count {
            result <<= 1;
            let byte_idx = *bit_pos / 8;
            let bit_idx = *bit_pos % 8;
            if byte_idx < data.len() {
                if data[byte_idx] & (1 << bit_idx) != 0 {
                    result |= 1;
                }
            }
            *bit_pos += 1;
        }
        result
    }

    let length_bases: [u16; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31,
        35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258
    ];
    let length_extra: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
        3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0
    ];
    let dist_bases: [u16; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193,
        257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577
    ];
    let dist_extra: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6,
        7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13
    ];

    loop {
        // Read code using fixed Huffman
        let mut code = get_bits_rev(data, bit_pos, 7);

        let symbol = if code <= 0b0010111 {
            // 256-279: 7 bits (0000000 - 0010111)
            code + 256
        } else {
            code = (code << 1) | get_bits_rev(data, bit_pos, 1);
            if code <= 0b10111111 {
                // 0-143: 8 bits (00110000 - 10111111)
                code - 0b00110000
            } else if code <= 0b11000111 {
                // 280-287: 8 bits (11000000 - 11000111)
                code - 0b11000000 + 280
            } else {
                code = (code << 1) | get_bits_rev(data, bit_pos, 1);
                // 144-255: 9 bits (110010000 - 111111111)
                code - 0b110010000 + 144
            }
        };

        if symbol < 256 {
            output.push(symbol as u8);
        } else if symbol == 256 {
            break;
        } else {
            // Length/distance pair
            let length_idx = (symbol - 257) as usize;
            if length_idx >= 29 { break; }
            let length = length_bases[length_idx] as usize +
                get_bits(data, bit_pos, length_extra[length_idx] as usize) as usize;

            // Read distance (5 bits, fixed)
            let dist_code = get_bits_rev(data, bit_pos, 5) as usize;
            if dist_code >= 30 { break; }
            let distance = dist_bases[dist_code] as usize +
                get_bits(data, bit_pos, dist_extra[dist_code] as usize) as usize;

            // Copy from output buffer
            let start = if distance > output.len() { 0 } else { output.len() - distance };
            for i in 0..length {
                let idx = start + (i % distance);
                if idx < output.len() {
                    output.push(output[idx]);
                }
            }
        }
    }
}

fn inflate_dynamic_huffman(data: &[u8], bit_pos: &mut usize, output: &mut Vec<u8>) {
    fn get_bits(data: &[u8], bit_pos: &mut usize, count: usize) -> u32 {
        let mut result = 0u32;
        for i in 0..count {
            let byte_idx = *bit_pos / 8;
            let bit_idx = *bit_pos % 8;
            if byte_idx < data.len() {
                if data[byte_idx] & (1 << bit_idx) != 0 {
                    result |= 1 << i;
                }
            }
            *bit_pos += 1;
        }
        result
    }

    let hlit = get_bits(data, bit_pos, 5) as usize + 257;
    let hdist = get_bits(data, bit_pos, 5) as usize + 1;
    let hclen = get_bits(data, bit_pos, 4) as usize + 4;

    // Read code length code lengths
    let order: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut code_length_lengths = [0u8; 19];
    for i in 0..hclen {
        code_length_lengths[order[i]] = get_bits(data, bit_pos, 3) as u8;
    }

    // Build code length Huffman tree
    let code_length_tree = build_huffman_tree(&code_length_lengths);

    // Read literal/length and distance code lengths
    let mut lengths = vec![0u8; hlit + hdist];
    let mut i = 0;
    while i < hlit + hdist {
        let sym = decode_huffman(data, bit_pos, &code_length_tree);
        match sym {
            0..=15 => {
                lengths[i] = sym as u8;
                i += 1;
            }
            16 => {
                let repeat = get_bits(data, bit_pos, 2) as usize + 3;
                let val = if i > 0 { lengths[i - 1] } else { 0 };
                for _ in 0..repeat {
                    if i < lengths.len() {
                        lengths[i] = val;
                        i += 1;
                    }
                }
            }
            17 => {
                let repeat = get_bits(data, bit_pos, 3) as usize + 3;
                for _ in 0..repeat {
                    if i < lengths.len() {
                        lengths[i] = 0;
                        i += 1;
                    }
                }
            }
            18 => {
                let repeat = get_bits(data, bit_pos, 7) as usize + 11;
                for _ in 0..repeat {
                    if i < lengths.len() {
                        lengths[i] = 0;
                        i += 1;
                    }
                }
            }
            _ => break,
        }
    }

    // Build literal/length and distance trees
    let lit_tree = build_huffman_tree(&lengths[..hlit]);
    let dist_tree = build_huffman_tree(&lengths[hlit..]);

    // Decode data
    let length_bases: [u16; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31,
        35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258
    ];
    let length_extra: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
        3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0
    ];
    let dist_bases: [u16; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193,
        257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577
    ];
    let dist_extra: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6,
        7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13
    ];

    loop {
        let symbol = decode_huffman(data, bit_pos, &lit_tree);

        if symbol < 256 {
            output.push(symbol as u8);
        } else if symbol == 256 {
            break;
        } else {
            let length_idx = (symbol - 257) as usize;
            if length_idx >= 29 { break; }
            let length = length_bases[length_idx] as usize +
                get_bits(data, bit_pos, length_extra[length_idx] as usize) as usize;

            let dist_code = decode_huffman(data, bit_pos, &dist_tree) as usize;
            if dist_code >= 30 { break; }
            let distance = dist_bases[dist_code] as usize +
                get_bits(data, bit_pos, dist_extra[dist_code] as usize) as usize;

            let start = if distance > output.len() { 0 } else { output.len() - distance };
            for i in 0..length {
                let idx = start + (i % distance);
                if idx < output.len() {
                    output.push(output[idx]);
                }
            }
        }
    }
}

// Simple Huffman tree representation
struct HuffmanTree {
    codes: Vec<(u16, u8, u16)>, // (code, length, symbol)
    max_bits: u8,
}

fn build_huffman_tree(lengths: &[u8]) -> HuffmanTree {
    let max_bits = *lengths.iter().max().unwrap_or(&0);
    if max_bits == 0 {
        return HuffmanTree { codes: Vec::new(), max_bits: 0 };
    }

    // Count codes of each length
    let mut bl_count = vec![0u16; max_bits as usize + 1];
    for &len in lengths {
        if len > 0 {
            bl_count[len as usize] += 1;
        }
    }

    // Find numerical value of smallest code for each length
    let mut next_code = vec![0u16; max_bits as usize + 1];
    let mut code = 0u16;
    for bits in 1..=max_bits as usize {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }

    // Assign codes
    let mut codes = Vec::new();
    for (symbol, &len) in lengths.iter().enumerate() {
        if len > 0 {
            let c = next_code[len as usize];
            next_code[len as usize] += 1;
            codes.push((c, len, symbol as u16));
        }
    }

    HuffmanTree { codes, max_bits }
}

fn decode_huffman(data: &[u8], bit_pos: &mut usize, tree: &HuffmanTree) -> u16 {
    if tree.codes.is_empty() {
        return 0;
    }

    let mut code = 0u16;
    for len in 1..=tree.max_bits {
        // Read one bit (MSB first for Huffman)
        let byte_idx = *bit_pos / 8;
        let bit_idx = *bit_pos % 8;
        code <<= 1;
        if byte_idx < data.len() {
            if data[byte_idx] & (1 << bit_idx) != 0 {
                code |= 1;
            }
        }
        *bit_pos += 1;

        // Check if this code matches
        for &(c, l, sym) in &tree.codes {
            if l == len && c == code {
                return sym;
            }
        }
    }

    0 // Not found
}

fn crc32(data: &[u8]) -> u32 {
    static CRC_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut j = 0;
            while j < 8 {
                if c & 1 != 0 {
                    c = 0xedb88320 ^ (c >> 1);
                } else {
                    c >>= 1;
                }
                j += 1;
            }
            table[i] = c;
            i += 1;
        }
        table
    };

    let mut crc = 0xffffffff_u32;
    for &b in data {
        crc = CRC_TABLE[((crc ^ b as u32) & 0xff) as usize] ^ (crc >> 8);
    }
    !crc
}

// ============= CPIO =============

pub fn cpio(argc: i32, argv: *const *const u8) -> i32 {
    let mut mode = 0u8; // 'i' = extract, 'o' = create, 't' = list
    let mut verbose = false;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg.starts_with(b"-") {
                for &c in &arg[1..] {
                    match c {
                        b'i' => mode = b'i',
                        b'o' => mode = b'o',
                        b't' => mode = b't',
                        b'v' => verbose = true,
                        _ => {}
                    }
                }
            }
        }
    }

    match mode {
        b'i' => cpio_extract(verbose),
        b'o' => cpio_create(verbose),
        b't' => cpio_list(verbose),
        _ => {
            io::write_str(2, b"cpio: specify -i, -o, or -t\n");
            1
        }
    }
}

// CPIO newc (SVR4) format header
#[repr(C)]
struct CpioHeader {
    magic: [u8; 6],      // "070701" or "070702"
    ino: [u8; 8],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    nlink: [u8; 8],
    mtime: [u8; 8],
    filesize: [u8; 8],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    rdevmajor: [u8; 8],
    rdevminor: [u8; 8],
    namesize: [u8; 8],
    check: [u8; 8],
}

fn parse_hex_8(bytes: &[u8]) -> u32 {
    let mut result = 0u32;
    for &b in bytes {
        result <<= 4;
        if b >= b'0' && b <= b'9' {
            result |= (b - b'0') as u32;
        } else if b >= b'a' && b <= b'f' {
            result |= (b - b'a' + 10) as u32;
        } else if b >= b'A' && b <= b'F' {
            result |= (b - b'A' + 10) as u32;
        }
    }
    result
}

fn write_hex_8(buf: &mut [u8], val: u32) {
    const HEX: &[u8] = b"0123456789ABCDEF";
    let mut v = val;
    for i in (0..8).rev() {
        buf[i] = HEX[(v & 0xf) as usize];
        v >>= 4;
    }
}

fn cpio_extract(verbose: bool) -> i32 {
    let mut header_buf = [0u8; 110];

    loop {
        if io::read(0, &mut header_buf) != 110 {
            break;
        }

        // Check magic
        if &header_buf[0..6] != b"070701" && &header_buf[0..6] != b"070702" {
            io::write_str(2, b"cpio: bad magic\n");
            return 1;
        }

        let mode = parse_hex_8(&header_buf[14..22]);
        let filesize = parse_hex_8(&header_buf[54..62]) as usize;
        let namesize = parse_hex_8(&header_buf[94..102]) as usize;

        // Read filename
        let mut name = vec![0u8; namesize];
        io::read(0, &mut name);

        // Skip padding to 4-byte boundary
        let header_total = 110 + namesize;
        let padding = (4 - (header_total % 4)) % 4;
        let mut skip = [0u8; 4];
        if padding > 0 {
            io::read(0, &mut skip[..padding]);
        }

        // Remove null terminator from name
        let name_str = if namesize > 0 && name[namesize - 1] == 0 {
            &name[..namesize - 1]
        } else {
            &name[..]
        };

        // Check for trailer
        if name_str == b"TRAILER!!!" {
            break;
        }

        if verbose {
            io::write_all(1, name_str);
            io::write_str(1, b"\n");
        }

        let is_dir = (mode & 0o170000) == 0o040000;
        let is_file = (mode & 0o170000) == 0o100000;

        if is_dir {
            // Create directory
            let mut path_z = vec![0u8; name_str.len() + 1];
            path_z[..name_str.len()].copy_from_slice(name_str);
            unsafe { libc::mkdir(path_z.as_ptr() as *const i8, (mode & 0o7777) as u32) };
        } else if is_file {
            // Create file
            create_parent_dirs(name_str);
            let mut path_z = vec![0u8; name_str.len() + 1];
            path_z[..name_str.len()].copy_from_slice(name_str);

            let fd = open_write_create(&path_z, (mode & 0o7777) as i32);
            if fd >= 0 {
                let mut remaining = filesize;
                let mut buf = [0u8; 4096];
                while remaining > 0 {
                    let to_read = remaining.min(4096);
                    let n = io::read(0, &mut buf[..to_read]);
                    if n <= 0 { break; }
                    io::write_all(fd, &buf[..n as usize]);
                    remaining -= n as usize;
                }
                io::close(fd);
            } else {
                // Skip file content
                let mut remaining = filesize;
                let mut buf = [0u8; 4096];
                while remaining > 0 {
                    let to_read = remaining.min(4096);
                    io::read(0, &mut buf[..to_read]);
                    remaining -= to_read;
                }
            }
        } else {
            // Skip unknown types
            let mut remaining = filesize;
            let mut buf = [0u8; 4096];
            while remaining > 0 {
                let to_read = remaining.min(4096);
                io::read(0, &mut buf[..to_read]);
                remaining -= to_read;
            }
        }

        // Skip data padding
        let data_padding = (4 - (filesize % 4)) % 4;
        if data_padding > 0 {
            io::read(0, &mut skip[..data_padding]);
        }
    }

    0
}

fn cpio_create(verbose: bool) -> i32 {
    // Read filenames from stdin, write cpio to stdout
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    let mut ino = 0u32;

    loop {
        line.clear();
        loop {
            if io::read(0, &mut byte) != 1 {
                if line.is_empty() {
                    // Write trailer
                    cpio_write_entry(b"TRAILER!!!", 0, 0, 0, &[], verbose);
                    return 0;
                }
                break;
            }
            if byte[0] == b'\n' { break; }
            line.push(byte[0]);
        }

        if line.is_empty() { continue; }

        // Stat the file
        let mut path_z = vec![0u8; line.len() + 1];
        path_z[..line.len()].copy_from_slice(&line);

        let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
        if unsafe { libc::lstat(path_z.as_ptr() as *const i8, &mut stat_buf) } != 0 {
            io::write_str(2, b"cpio: cannot stat ");
            io::write_all(2, &line);
            io::write_str(2, b"\n");
            continue;
        }

        ino += 1;
        let mode = stat_buf.st_mode;
        let is_file = (mode & libc::S_IFMT) == libc::S_IFREG;

        if is_file {
            // Read file content
            let fd = open_read(&path_z);
            if fd >= 0 {
                let mut content = Vec::new();
                let mut buf = [0u8; 4096];
                loop {
                    let n = io::read(fd, &mut buf);
                    if n <= 0 { break; }
                    content.extend_from_slice(&buf[..n as usize]);
                }
                io::close(fd);
                cpio_write_entry(&line, ino, mode as u32, stat_buf.st_mtime as u32, &content, verbose);
            }
        } else {
            cpio_write_entry(&line, ino, mode as u32, stat_buf.st_mtime as u32, &[], verbose);
        }
    }
}

fn cpio_write_entry(name: &[u8], ino: u32, mode: u32, mtime: u32, data: &[u8], verbose: bool) {
    let mut header = [0u8; 110];

    // Magic
    header[0..6].copy_from_slice(b"070701");

    // ino
    write_hex_8(&mut header[6..14], ino);

    // mode
    write_hex_8(&mut header[14..22], mode);

    // uid, gid
    write_hex_8(&mut header[22..30], 0);
    write_hex_8(&mut header[30..38], 0);

    // nlink
    write_hex_8(&mut header[38..46], 1);

    // mtime
    write_hex_8(&mut header[46..54], mtime);

    // filesize
    write_hex_8(&mut header[54..62], data.len() as u32);

    // devmajor, devminor, rdevmajor, rdevminor
    write_hex_8(&mut header[62..70], 0);
    write_hex_8(&mut header[70..78], 0);
    write_hex_8(&mut header[78..86], 0);
    write_hex_8(&mut header[86..94], 0);

    // namesize (including null terminator)
    write_hex_8(&mut header[94..102], (name.len() + 1) as u32);

    // check
    write_hex_8(&mut header[102..110], 0);

    // Write header
    io::write_all(1, &header);

    // Write name with null terminator
    io::write_all(1, name);
    io::write_all(1, &[0]);

    // Padding to 4-byte boundary
    let header_total = 110 + name.len() + 1;
    let padding = (4 - (header_total % 4)) % 4;
    for _ in 0..padding {
        io::write_all(1, &[0]);
    }

    // Write data
    if !data.is_empty() {
        io::write_all(1, data);

        // Data padding
        let data_padding = (4 - (data.len() % 4)) % 4;
        for _ in 0..data_padding {
            io::write_all(1, &[0]);
        }
    }

    if verbose {
        io::write_all(2, name);
        io::write_str(2, b"\n");
    }
}

fn cpio_list(verbose: bool) -> i32 {
    let mut header_buf = [0u8; 110];

    loop {
        if io::read(0, &mut header_buf) != 110 {
            break;
        }

        if &header_buf[0..6] != b"070701" && &header_buf[0..6] != b"070702" {
            io::write_str(2, b"cpio: bad magic\n");
            return 1;
        }

        let mode = parse_hex_8(&header_buf[14..22]);
        let filesize = parse_hex_8(&header_buf[54..62]) as usize;
        let namesize = parse_hex_8(&header_buf[94..102]) as usize;

        let mut name = vec![0u8; namesize];
        io::read(0, &mut name);

        let header_total = 110 + namesize;
        let padding = (4 - (header_total % 4)) % 4;
        let mut skip = [0u8; 4];
        if padding > 0 {
            io::read(0, &mut skip[..padding]);
        }

        let name_str = if namesize > 0 && name[namesize - 1] == 0 {
            &name[..namesize - 1]
        } else {
            &name[..]
        };

        if name_str == b"TRAILER!!!" {
            break;
        }

        if verbose {
            // Print mode in ls -l format
            let type_char = match mode & 0o170000 {
                0o040000 => b'd',
                0o120000 => b'l',
                _ => b'-',
            };
            io::write_all(1, &[type_char]);
            io::write_all(1, &mode_string((mode & 0o7777) as u32));
            io::write_str(1, b" ");
            io::write_num(1, filesize as u64);
            io::write_str(1, b" ");
        }
        io::write_all(1, name_str);
        io::write_str(1, b"\n");

        // Skip data
        let mut remaining = filesize;
        let mut buf = [0u8; 4096];
        while remaining > 0 {
            let to_read = remaining.min(4096);
            io::read(0, &mut buf[..to_read]);
            remaining -= to_read;
        }

        let data_padding = (4 - (filesize % 4)) % 4;
        if data_padding > 0 {
            io::read(0, &mut skip[..data_padding]);
        }
    }

    0
}

// ============= BZIP2 =============

pub fn bzip2(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    io::write_str(2, b"bzip2: not implemented (complex algorithm)\n");
    1
}

pub fn bunzip2(argc: i32, argv: *const *const u8) -> i32 { bzip2(argc, argv) }
pub fn bzcat(argc: i32, argv: *const *const u8) -> i32 { bzip2(argc, argv) }

// ============= XZ (LZMA) =============

pub fn xz(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    io::write_str(2, b"xz: not implemented (complex algorithm)\n");
    1
}

pub fn unxz(argc: i32, argv: *const *const u8) -> i32 { xz(argc, argv) }
pub fn xzcat(argc: i32, argv: *const *const u8) -> i32 { xz(argc, argv) }

// ============= UNZIP =============

pub fn unzip(argc: i32, argv: *const *const u8) -> i32 {
    let mut list_only = false;
    let mut archive: Option<&[u8]> = None;

    for i in 1..argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-l" {
                list_only = true;
            } else if !arg.starts_with(b"-") && archive.is_none() {
                archive = Some(arg);
            }
        }
    }

    let archive = match archive {
        Some(a) => a,
        None => {
            io::write_str(2, b"unzip: specify archive\n");
            return 1;
        }
    };

    let fd = open_read(archive);
    if fd < 0 {
        io::write_str(2, b"unzip: cannot open archive\n");
        return 1;
    }

    // Read entire archive into memory (simplified approach)
    let mut data = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(fd, &mut buf);
        if n <= 0 { break; }
        data.extend_from_slice(&buf[..n as usize]);
    }
    io::close(fd);

    // Process ZIP local file headers
    let mut offset = 0usize;

    while offset + 30 <= data.len() {
        // Check for local file header signature
        if &data[offset..offset + 4] != &[0x50, 0x4b, 0x03, 0x04] {
            // Might be central directory, stop
            break;
        }

        let compression = u16::from_le_bytes([data[offset + 8], data[offset + 9]]);
        let compressed_size = u32::from_le_bytes([
            data[offset + 18], data[offset + 19], data[offset + 20], data[offset + 21]
        ]) as usize;
        let uncompressed_size = u32::from_le_bytes([
            data[offset + 22], data[offset + 23], data[offset + 24], data[offset + 25]
        ]) as usize;
        let filename_len = u16::from_le_bytes([data[offset + 26], data[offset + 27]]) as usize;
        let extra_len = u16::from_le_bytes([data[offset + 28], data[offset + 29]]) as usize;

        let filename_start = offset + 30;
        let filename_end = filename_start + filename_len;
        let data_start = filename_end + extra_len;
        let data_end = data_start + compressed_size;

        if data_end > data.len() {
            break;
        }

        let filename = &data[filename_start..filename_end];

        if list_only {
            io::write_num(1, uncompressed_size as u64);
            io::write_str(1, b"  ");
            io::write_all(1, filename);
            io::write_str(1, b"\n");
        } else {
            io::write_str(1, b"  inflating: ");
            io::write_all(1, filename);
            io::write_str(1, b"\n");

            // Extract file
            if !filename.ends_with(b"/") {
                create_parent_dirs(filename);
                let mut path_z = vec![0u8; filename.len() + 1];
                path_z[..filename.len()].copy_from_slice(filename);

                let out_fd = open_write_create(&path_z, 0o644);
                if out_fd >= 0 {
                    let compressed_data = &data[data_start..data_end];

                    match compression {
                        0 => {
                            // Stored (no compression)
                            io::write_all(out_fd, compressed_data);
                        }
                        8 => {
                            // DEFLATE
                            let decompressed = inflate(compressed_data);
                            io::write_all(out_fd, &decompressed);
                        }
                        _ => {
                            io::write_str(2, b"unzip: unsupported compression\n");
                        }
                    }
                    io::close(out_fd);
                }
            } else {
                // Directory
                let mut path_z = vec![0u8; filename.len() + 1];
                path_z[..filename.len()].copy_from_slice(filename);
                unsafe { libc::mkdir(path_z.as_ptr() as *const i8, 0o755) };
            }
        }

        offset = data_end;
    }

    0
}

// ============= COMPRESS (LZW) =============

pub fn compress(argc: i32, argv: *const *const u8) -> i32 {
    let _ = argc;
    let _ = argv;
    io::write_str(2, b"compress: not implemented (legacy format)\n");
    1
}

pub fn uncompress(argc: i32, argv: *const *const u8) -> i32 { compress(argc, argv) }
