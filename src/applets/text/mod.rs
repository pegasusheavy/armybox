//! Text processing applets
//!
//! POSIX.1-2017 compliant text manipulation utilities.

// Individual utility modules
mod awk;
mod comm;
mod cut;
mod dos2unix;
mod echo;
mod expand;
mod fmt;
mod fold;
mod grep;
mod head;
mod nl;
mod paste;
mod printf;
mod rev;
mod sed;
mod seq;
mod sort;
mod strings;
mod tac;
mod tail;
mod tee;
mod tr;
mod unexpand;
mod uniq;
mod unix2dos;
mod wc;
mod yes;

// Re-export utilities
pub use awk::awk;
pub use comm::comm;
pub use cut::cut;
pub use dos2unix::dos2unix;
pub use echo::echo;
pub use expand::expand;
pub use fmt::fmt;
pub use fold::fold;
pub use grep::{grep, egrep, fgrep};
pub use head::head;
pub use nl::nl;
pub use paste::paste;
pub use printf::printf;
pub use rev::rev;
pub use sed::sed;
pub use seq::seq;
pub use sort::sort;
pub use strings::strings;
pub use tac::tac;
pub use tail::tail;
pub use tee::tee;
pub use tr::tr;
pub use unexpand::unexpand;
pub use uniq::uniq;
pub use unix2dos::unix2dos;
pub use wc::wc;
pub use yes::yes;
