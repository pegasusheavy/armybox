//! Init system utilities

use crate::io;
use super::get_arg;

pub fn init(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"init: stub\n"); 0 }
pub fn telinit(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn runlevel(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"N 3\n"); 0 }
pub fn getty(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn sulogin(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn oneit(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn switch_root(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn watchdog(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
