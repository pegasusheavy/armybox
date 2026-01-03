//! Shell implementation

use crate::io;
use super::get_arg;

pub fn sh(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"sh: stub\n"); 0 }
pub fn ash(argc: i32, argv: *const *const u8) -> i32 { sh(argc, argv) }
pub fn dash(argc: i32, argv: *const *const u8) -> i32 { sh(argc, argv) }
