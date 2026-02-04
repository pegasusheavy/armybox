//! File operation applets
//!
//! POSIX.1-2017 compliant file manipulation utilities.

use crate::io;
use crate::sys;
use super::{get_arg, has_opt, is_opt};

// Individual utility modules
mod basename;
mod cat;
mod cd;
mod chgrp;
mod chmod;
mod chown;
mod cp;
mod dd;
mod dirname;
mod file_cmd;
mod find_cmd;
mod install;
mod link_cmd;
mod ln;
mod ls;
mod mkdir;
mod mkfifo;
mod mknod;
mod mktemp;
mod mv;
mod patch;
mod pwd;
mod readlink;
mod realpath;
mod rm;
mod rmdir;
mod shred;
mod split;
mod stat;
mod stubs;
mod sync_cmd;
mod touch;
mod truncate;
mod unlink;
mod xargs;

// Re-export utilities
pub use basename::basename;
pub use cat::cat;
pub use cd::cd;
pub use chgrp::chgrp;
pub use chmod::chmod;
pub use chown::chown;
pub use cp::cp;
pub use dd::dd;
pub use dirname::dirname;
pub use file_cmd::file;
pub use find_cmd::find;
pub use install::install;
pub use link_cmd::link;
pub use ln::ln;
pub use ls::ls;
pub use mkdir::mkdir;
pub use mkfifo::mkfifo;
pub use mknod::mknod;
pub use mktemp::mktemp;
pub use mv::mv;
pub use patch::patch;
pub use pwd::pwd;
pub use readlink::readlink;
pub use realpath::realpath;
pub use rm::rm;
pub use rmdir::rmdir;
pub use shred::shred;
pub use split::split;
pub use stat::stat;
pub use stubs::{chattr, lsattr, fstype, makedevs, setfattr};
pub use sync_cmd::sync_cmd;
pub use touch::touch;
pub use truncate::truncate;
pub use unlink::unlink;
pub use xargs::xargs;

// Re-export helpers for use by install
pub(crate) use cp::copy_file;
pub(crate) use mkdir::mkdir_parents;
