//! Text editors

use crate::io;
use super::get_arg;

pub fn vi(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"vi: stub\n"); 0 }
pub fn view(argc: i32, argv: *const *const u8) -> i32 { vi(argc, argv) }
pub fn hexedit(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"hexedit: stub\n"); 0 }
