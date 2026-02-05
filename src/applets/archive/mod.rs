//! Archive utilities

use crate::io;
use super::get_arg;

// Extracted modules
#[cfg(feature = "bzip2")]
mod bzip2;
#[cfg(feature = "compress")]
mod compress;
#[cfg(feature = "cpio")]
mod cpio;
#[cfg(feature = "gzip")]
mod gzip;
#[cfg(feature = "tar")]
mod tar;
#[cfg(feature = "unzip")]
mod unzip;
#[cfg(feature = "xz")]
mod xz;

// Re-export all utilities
#[cfg(feature = "bzip2")]
pub use bzip2::{bzip2, bunzip2, bzcat};
#[cfg(feature = "compress")]
pub use compress::{compress, uncompress};
#[cfg(feature = "cpio")]
pub use cpio::cpio;
#[cfg(feature = "gzip")]
pub use gzip::{gzip, gunzip, zcat};
#[cfg(feature = "tar")]
pub use tar::tar;
#[cfg(feature = "unzip")]
pub use unzip::unzip;
#[cfg(feature = "xz")]
pub use xz::{xz, unxz, xzcat, lzma, unlzma, lzcat};

// Shared helper functions used by multiple archive utilities

#[cfg(any(feature = "tar", feature = "cpio", feature = "unzip"))]
pub(crate) fn open_read(path: &[u8]) -> i32 {
    io::open(path, libc::O_RDONLY, 0)
}

#[cfg(any(feature = "tar", feature = "cpio", feature = "unzip"))]
pub(crate) fn open_write_create(path: &[u8], mode: i32) -> i32 {
    io::open(path, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, mode as u32)
}

#[cfg(any(feature = "tar", feature = "cpio", feature = "unzip"))]
pub(crate) fn create_parent_dirs(path: &[u8]) {
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

#[cfg(any(feature = "tar", feature = "cpio"))]
pub(crate) fn mode_string(mode: u32) -> [u8; 9] {
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
