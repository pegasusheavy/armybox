//! Miscellaneous utilities
//!
//! Various utility commands that don't fit into other categories.

use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

// Extracted modules
#[cfg(feature = "base64")]
mod base64;
#[cfg(feature = "boolean")]
mod boolean;
#[cfg(feature = "cmp")]
mod cmp;
#[cfg(feature = "diff")]
mod diff;
#[cfg(feature = "expr")]
mod expr;
#[cfg(feature = "factor")]
mod factor;
#[cfg(feature = "getconf")]
mod getconf;
#[cfg(feature = "hash")]
mod hash;
#[cfg(feature = "hexdump")]
mod hexdump;
#[cfg(feature = "mesg")]
mod mesg;
#[cfg(feature = "readelf")]
mod readelf;
#[cfg(feature = "screen")]
mod screen;
#[cfg(any(
    feature = "ascii",
    feature = "iconv",
    feature = "tsort",
    feature = "getopt",
    feature = "count",
    feature = "unicode",
    feature = "ts",
    feature = "uuidgen",
    feature = "mcookie",
    feature = "pwgen",
    feature = "uuencode",
    feature = "uudecode",
    feature = "help",
    feature = "memeater",
    feature = "mix",
    feature = "mkpasswd",
    feature = "toybox"
))]
mod stubs;
#[cfg(feature = "terminal")]
mod terminal;
#[cfg(feature = "test")]
mod test;
#[cfg(feature = "time")]
mod time;
#[cfg(feature = "which")]
mod which;

// Re-export all utilities
#[cfg(feature = "base64")]
pub use base64::{base64, base32};
#[cfg(feature = "boolean")]
pub use boolean::{r#true, r#false, colon};
#[cfg(feature = "cmp")]
pub use cmp::cmp;
#[cfg(feature = "diff")]
pub use diff::diff;
#[cfg(feature = "expr")]
pub use expr::expr;
#[cfg(feature = "factor")]
pub use factor::factor;
#[cfg(feature = "getconf")]
pub use getconf::getconf;
#[cfg(feature = "hash")]
pub use hash::{md5sum, sha1sum, sha256sum, sha224sum, sha384sum, sha512sum, sha3sum, cksum, crc32};
#[cfg(feature = "hexdump")]
pub use hexdump::{hexdump, hd, xxd, od};
#[cfg(feature = "mesg")]
pub use mesg::mesg;
#[cfg(feature = "readelf")]
pub use readelf::readelf;
#[cfg(feature = "screen")]
pub use screen::screen;
#[cfg(feature = "ascii")]
pub use stubs::ascii;
#[cfg(feature = "iconv")]
pub use stubs::iconv;
#[cfg(feature = "tsort")]
pub use stubs::tsort;
#[cfg(feature = "getopt")]
pub use stubs::getopt;
#[cfg(feature = "count")]
pub use stubs::count;
#[cfg(feature = "unicode")]
pub use stubs::unicode;
#[cfg(feature = "ts")]
pub use stubs::ts;
#[cfg(feature = "uuidgen")]
pub use stubs::uuidgen;
#[cfg(feature = "mcookie")]
pub use stubs::mcookie;
#[cfg(feature = "pwgen")]
pub use stubs::pwgen;
#[cfg(feature = "uuencode")]
pub use stubs::uuencode;
#[cfg(feature = "uudecode")]
pub use stubs::uudecode;
#[cfg(feature = "help")]
pub use stubs::help;
#[cfg(feature = "memeater")]
pub use stubs::memeater;
#[cfg(feature = "mix")]
pub use stubs::mix;
#[cfg(feature = "mkpasswd")]
pub use stubs::mkpasswd;
#[cfg(feature = "toybox")]
pub use stubs::toybox;
#[cfg(feature = "terminal")]
pub use terminal::{clear, reset};
#[cfg(feature = "test")]
pub use test::{test, bracket};
#[cfg(feature = "time")]
pub use time::time;
#[cfg(feature = "which")]
pub use which::which;
