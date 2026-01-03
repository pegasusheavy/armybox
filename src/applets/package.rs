//! Package management (feature-gated)

use crate::io;
use super::get_arg;

pub fn apk(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"apk: stub\n"); 0 }
