//! Miscellaneous utilities
//!
//! Various utility commands that don't fit into other categories.

use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

// Extracted modules
mod base64;
mod boolean;
mod cmp;
mod diff;
mod expr;
mod factor;
mod getconf;
mod hash;
mod hexdump;
mod mesg;
mod readelf;
mod screen;
mod stubs;
mod terminal;
mod test;
mod time;
mod which;

// Re-export all utilities
pub use base64::{base64, base32};
pub use boolean::{r#true, r#false, colon};
pub use cmp::cmp;
pub use diff::diff;
pub use expr::expr;
pub use factor::factor;
pub use getconf::getconf;
pub use hash::{md5sum, sha1sum, sha256sum, sha224sum, sha384sum, sha512sum, sha3sum, cksum, crc32};
pub use hexdump::{hexdump, hd, xxd, od};
pub use mesg::mesg;
pub use readelf::readelf;
pub use screen::screen;
pub use stubs::{ascii, iconv, tsort, getopt, count, unicode, ts, uuidgen, mcookie, pwgen, uuencode, uudecode, help, memeater, mix, mkpasswd, toybox};
pub use terminal::{clear, reset};
pub use test::{test, bracket};
pub use time::time;
pub use which::which;
