//! Archive utilities

use crate::io;
use super::get_arg;

pub fn tar(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"tar: stub\n"); 0 }
pub fn gzip(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"gzip: stub\n"); 0 }
pub fn gunzip(argc: i32, argv: *const *const u8) -> i32 { gzip(argc, argv) }
pub fn zcat(argc: i32, argv: *const *const u8) -> i32 { gzip(argc, argv) }
pub fn bzip2(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"bzip2: stub\n"); 0 }
pub fn bunzip2(argc: i32, argv: *const *const u8) -> i32 { bzip2(argc, argv) }
pub fn bzcat(argc: i32, argv: *const *const u8) -> i32 { bzip2(argc, argv) }
pub fn xz(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"xz: stub\n"); 0 }
pub fn unxz(argc: i32, argv: *const *const u8) -> i32 { xz(argc, argv) }
pub fn xzcat(argc: i32, argv: *const *const u8) -> i32 { xz(argc, argv) }
pub fn cpio(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"cpio: stub\n"); 0 }
pub fn unzip(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"unzip: stub\n"); 0 }
pub fn compress(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"compress: stub\n"); 0 }
pub fn uncompress(argc: i32, argv: *const *const u8) -> i32 { compress(argc, argv) }
