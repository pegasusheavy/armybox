//! Process utilities for ArmyBox
//!
//! Process management, monitoring, and control utilities.

use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

// Process management and monitoring
mod kill;
mod killall;
mod killall5;
mod pgrep;
mod pkill;
mod pidof;
mod ps;
mod pmap;
mod pwdx;
mod top;
mod iotop;

// Process execution control
mod nice;
mod renice;
mod nohup;
mod chrt;
mod ionice;
mod iorenice;
mod taskset;
mod timeout;
mod setsid;
mod prlimit;
mod uclampset;

// Namespace utilities
mod nsenter;
mod unshare;

// Re-export process management utilities
pub use kill::kill;
pub use killall::killall;
pub use killall5::killall5;
pub use pgrep::pgrep;
pub use pkill::pkill;
pub use pidof::pidof;
pub use ps::ps;
pub use pmap::pmap;
pub use pwdx::pwdx;
pub use top::top;
pub use iotop::iotop;

// Re-export process execution control utilities
pub use nice::nice;
pub use renice::renice;
pub use nohup::nohup;
pub use chrt::chrt;
pub use ionice::ionice;
pub use iorenice::iorenice;
pub use taskset::taskset;
pub use timeout::timeout;
pub use setsid::setsid;
pub use prlimit::prlimit;
pub use uclampset::uclampset;

// Re-export namespace utilities
pub use nsenter::nsenter;
pub use unshare::unshare;
