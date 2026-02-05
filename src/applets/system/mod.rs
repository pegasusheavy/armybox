//! System utilities

use alloc::vec::Vec;
use crate::io;
use crate::sys;
use super::{get_arg, has_opt};

// Individual utility modules (batch 1: core system info)
mod arch;
mod groups;
mod hostname;
mod id;
mod uname;
mod users;
mod w;
mod who;
mod whoami;

// Batch 2: date/time/environment
mod date;
mod env;
mod free;
mod printenv;
mod sleep;
mod tty;
mod uptime;
mod usleep;

// Batch 3: filesystem and power
mod df;
mod dmesg;
mod du;
mod halt;
mod mount;
mod mountpoint;
mod poweroff;
mod reboot;
mod umount;

// Batch 4: system info and control
mod chroot;
mod flock;
mod hostid;
mod logger;
mod logname;
mod nproc;

// Batch 5: system/kernel utilities
mod blkid;
mod fsync_cmd;
mod insmod;
mod losetup;
mod lsmod;
mod modprobe;
mod rmmod;
mod swapoff;
mod swapon;
mod sysctl;
mod vmstat;

// Batch 6: misc system utilities
mod acpi;
mod cal;
mod fallocate;
mod hwclock;
mod mkswap;
mod nologin;
mod shuf;
mod watch;

// Batch 7: console/hardware utilities
mod chvt;
mod dnsdomainname;
mod fgconsole;
mod lspci;
mod lsusb;
mod pivot_root;
mod rfkill;

// Batch 8: device control and resource limits
mod blkdiscard;
mod blockdev;
mod eject;
mod login;
mod modinfo;
mod readahead_cmd;
mod rtcwake;
mod su;
mod ulimit;

// Batch 9: VT, memory, GPIO utilities
mod deallocvt;
mod devmem;
mod freeramdisk;
mod fsfreeze;
mod gpiodetect;
mod gpiofind;
mod gpioget;
mod gpioinfo;
mod gpioset;
mod i2cdetect;

// Batch 10: I2C and remaining utilities
mod i2cdump;
mod i2cget;
mod i2cset;
mod i2ctransfer;
mod inotifyd;
mod linux32;
mod openvt;
mod partprobe;

// Re-export batch 1 utilities
pub use arch::arch;
pub use groups::groups;
pub use hostname::hostname;
pub use id::id;
pub use uname::uname;
pub use users::users;
pub use w::w;
pub use who::who;
pub use whoami::whoami;

// Re-export batch 2 utilities
pub use date::date;
pub use env::env;
pub use free::free;
pub use printenv::printenv;
pub use sleep::sleep;
pub use tty::tty;
pub use uptime::uptime;
pub use usleep::usleep;

// Re-export batch 3 utilities
pub use df::df;
pub use dmesg::dmesg;
pub use du::du;
pub use halt::halt;
pub use mount::mount;
pub use mountpoint::mountpoint;
pub use poweroff::poweroff;
pub use reboot::reboot;
pub use umount::umount;

// Re-export batch 4 utilities
pub use chroot::chroot;
pub use flock::flock;
pub use hostid::hostid;
pub use logger::logger;
pub use logname::logname;
pub use nproc::nproc;

// Re-export batch 5 utilities
pub use blkid::blkid;
pub use fsync_cmd::fsync_cmd;
pub use insmod::insmod;
pub use losetup::losetup;
pub use lsmod::lsmod;
pub use modprobe::modprobe;
pub use rmmod::rmmod;
pub use swapoff::swapoff;
pub use swapon::swapon;
pub use sysctl::sysctl;
pub use vmstat::vmstat;

// Re-export batch 6 utilities
pub use acpi::acpi;
pub use cal::cal;
pub use fallocate::fallocate;
pub use hwclock::hwclock;
pub use mkswap::mkswap;
pub use nologin::nologin;
pub use shuf::shuf;
pub use watch::watch;

// Re-export batch 7 utilities
pub use chvt::chvt;
pub use dnsdomainname::dnsdomainname;
pub use fgconsole::fgconsole;
pub use lspci::lspci;
pub use lsusb::lsusb;
pub use pivot_root::pivot_root;
pub use rfkill::rfkill;

// Re-export batch 8 utilities
pub use blkdiscard::blkdiscard;
pub use blockdev::blockdev;
pub use eject::eject;
pub use login::login;
pub use modinfo::modinfo;
pub use readahead_cmd::readahead_cmd;
pub use rtcwake::rtcwake;
pub use su::su;
pub use ulimit::ulimit;

// Re-export batch 9 utilities
pub use deallocvt::deallocvt;
pub use devmem::devmem;
pub use freeramdisk::freeramdisk;
pub use fsfreeze::fsfreeze;
pub use gpiodetect::gpiodetect;
pub use gpiofind::gpiofind;
pub use gpioget::gpioget;
pub use gpioinfo::gpioinfo;
pub use gpioset::gpioset;
pub use i2cdetect::i2cdetect;

// Re-export batch 10 utilities
pub use i2cdump::i2cdump;
pub use i2cget::i2cget;
pub use i2cset::i2cset;
pub use i2ctransfer::i2ctransfer;
pub use inotifyd::inotifyd;
pub use linux32::linux32;
pub use openvt::openvt;
pub use partprobe::partprobe;
