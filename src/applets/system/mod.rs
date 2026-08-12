//! System utilities

use super::get_arg;

// Individual utility modules (batch 1: core system info)
#[cfg(feature = "arch")]
mod arch;
#[cfg(feature = "groups")]
mod groups;
#[cfg(feature = "hostname")]
mod hostname;
#[cfg(feature = "id")]
mod id;
#[cfg(feature = "uname")]
mod uname;
#[cfg(feature = "users")]
mod users;
#[cfg(feature = "w")]
mod w;
#[cfg(feature = "who")]
mod who;
#[cfg(feature = "whoami")]
mod whoami;

// Batch 2: date/time/environment
#[cfg(feature = "date")]
mod date;
#[cfg(feature = "env")]
mod env;
#[cfg(feature = "free")]
mod free;
#[cfg(feature = "printenv")]
mod printenv;
#[cfg(feature = "sleep")]
mod sleep;
#[cfg(feature = "tty")]
mod tty;
#[cfg(feature = "uptime")]
mod uptime;
#[cfg(feature = "usleep")]
mod usleep;

// Batch 3: filesystem and power
#[cfg(feature = "df")]
mod df;
#[cfg(feature = "dmesg")]
mod dmesg;
#[cfg(feature = "du")]
mod du;
#[cfg(feature = "halt")]
mod halt;
#[cfg(feature = "mount")]
mod mount;
#[cfg(feature = "mountpoint")]
mod mountpoint;
#[cfg(feature = "poweroff")]
mod poweroff;
#[cfg(feature = "reboot")]
mod reboot;
#[cfg(feature = "umount")]
mod umount;

// Batch 4: system info and control
#[cfg(feature = "chroot")]
mod chroot;
#[cfg(feature = "flock")]
mod flock;
#[cfg(feature = "hostid")]
mod hostid;
#[cfg(feature = "logger")]
mod logger;
#[cfg(feature = "logname")]
mod logname;
#[cfg(feature = "nproc")]
mod nproc;

// Batch 5: system/kernel utilities
#[cfg(feature = "blkid")]
mod blkid;
#[cfg(feature = "fsync")]
mod fsync_cmd;
#[cfg(feature = "insmod")]
mod insmod;
#[cfg(feature = "losetup")]
mod losetup;
#[cfg(feature = "lsmod")]
mod lsmod;
#[cfg(feature = "modprobe")]
mod modprobe;
#[cfg(feature = "rmmod")]
mod rmmod;
#[cfg(feature = "swapoff")]
mod swapoff;
#[cfg(feature = "swapon")]
mod swapon;
#[cfg(feature = "sysctl")]
mod sysctl;
#[cfg(feature = "vmstat")]
mod vmstat;

// Batch 6: misc system utilities
#[cfg(feature = "acpi")]
mod acpi;
#[cfg(feature = "cal")]
mod cal;
#[cfg(feature = "fallocate")]
mod fallocate;
#[cfg(feature = "hwclock")]
mod hwclock;
#[cfg(feature = "mkswap")]
mod mkswap;
#[cfg(feature = "nologin")]
mod nologin;
#[cfg(feature = "shuf")]
mod shuf;
#[cfg(feature = "watch")]
mod watch;

// Batch 7: console/hardware utilities
#[cfg(feature = "chvt")]
mod chvt;
#[cfg(feature = "dnsdomainname")]
mod dnsdomainname;
#[cfg(feature = "fgconsole")]
mod fgconsole;
#[cfg(feature = "lspci")]
mod lspci;
#[cfg(feature = "lsusb")]
mod lsusb;
#[cfg(feature = "pivot_root")]
mod pivot_root;
#[cfg(feature = "rfkill")]
mod rfkill;

// Batch 8: device control and resource limits
#[cfg(feature = "blkdiscard")]
mod blkdiscard;
#[cfg(feature = "blockdev")]
mod blockdev;
#[cfg(feature = "eject")]
mod eject;
#[cfg(feature = "login")]
pub(crate) mod login;
#[cfg(feature = "modinfo")]
mod modinfo;
#[cfg(feature = "readahead")]
mod readahead_cmd;
#[cfg(feature = "rtcwake")]
mod rtcwake;
#[cfg(feature = "su")]
mod su;
#[cfg(feature = "ulimit")]
mod ulimit;

// Batch 9: VT, memory, GPIO utilities
#[cfg(feature = "deallocvt")]
mod deallocvt;
#[cfg(feature = "devmem")]
mod devmem;
#[cfg(feature = "freeramdisk")]
mod freeramdisk;
#[cfg(feature = "fsfreeze")]
mod fsfreeze;
#[cfg(feature = "gpiodetect")]
mod gpiodetect;
#[cfg(feature = "gpiofind")]
mod gpiofind;
#[cfg(feature = "gpioget")]
mod gpioget;
#[cfg(feature = "gpioinfo")]
mod gpioinfo;
#[cfg(feature = "gpioset")]
mod gpioset;
#[cfg(feature = "i2cdetect")]
mod i2cdetect;

// Batch 10: I2C and remaining utilities
#[cfg(feature = "i2cdump")]
mod i2cdump;
#[cfg(feature = "i2cget")]
mod i2cget;
#[cfg(feature = "i2cset")]
mod i2cset;
#[cfg(feature = "i2ctransfer")]
mod i2ctransfer;
#[cfg(feature = "inotifyd")]
mod inotifyd;
#[cfg(feature = "linux32")]
mod linux32;
#[cfg(feature = "openvt")]
mod openvt;
#[cfg(feature = "partprobe")]
mod partprobe;

// Re-export batch 1 utilities
#[cfg(feature = "arch")]
pub use arch::arch;
#[cfg(feature = "groups")]
pub use groups::groups;
#[cfg(feature = "hostname")]
pub use hostname::hostname;
#[cfg(feature = "id")]
pub use id::id;
#[cfg(feature = "uname")]
pub use uname::uname;
#[cfg(feature = "users")]
pub use users::users;
#[cfg(feature = "w")]
pub use w::w;
#[cfg(feature = "who")]
pub use who::who;
#[cfg(feature = "whoami")]
pub use whoami::whoami;

// Re-export batch 2 utilities
#[cfg(feature = "date")]
pub use date::date;
#[cfg(feature = "env")]
pub use env::env;
#[cfg(feature = "free")]
pub use free::free;
#[cfg(feature = "printenv")]
pub use printenv::printenv;
#[cfg(feature = "sleep")]
pub use sleep::sleep;
#[cfg(feature = "tty")]
pub use tty::tty;
#[cfg(feature = "uptime")]
pub use uptime::uptime;
#[cfg(feature = "usleep")]
pub use usleep::usleep;

// Re-export batch 3 utilities
#[cfg(feature = "df")]
pub use df::df;
#[cfg(feature = "dmesg")]
pub use dmesg::dmesg;
#[cfg(feature = "du")]
pub use du::du;
#[cfg(feature = "halt")]
pub use halt::halt;
#[cfg(feature = "mount")]
pub use mount::mount;
#[cfg(feature = "mountpoint")]
pub use mountpoint::mountpoint;
#[cfg(feature = "poweroff")]
pub use poweroff::poweroff;
#[cfg(feature = "reboot")]
pub use reboot::reboot;
#[cfg(feature = "umount")]
pub use umount::umount;

// Re-export batch 4 utilities
#[cfg(feature = "chroot")]
pub use chroot::chroot;
#[cfg(feature = "flock")]
pub use flock::flock;
#[cfg(feature = "hostid")]
pub use hostid::hostid;
#[cfg(feature = "logger")]
pub use logger::logger;
#[cfg(feature = "logname")]
pub use logname::logname;
#[cfg(feature = "nproc")]
pub use nproc::nproc;

// Re-export batch 5 utilities
#[cfg(feature = "blkid")]
pub use blkid::blkid;
#[cfg(feature = "fsync")]
pub use fsync_cmd::fsync_cmd;
#[cfg(feature = "insmod")]
pub use insmod::insmod;
#[cfg(feature = "losetup")]
pub use losetup::losetup;
#[cfg(feature = "lsmod")]
pub use lsmod::lsmod;
#[cfg(feature = "modprobe")]
pub use modprobe::modprobe;
#[cfg(feature = "rmmod")]
pub use rmmod::rmmod;
#[cfg(feature = "swapoff")]
pub use swapoff::swapoff;
#[cfg(feature = "swapon")]
pub use swapon::swapon;
#[cfg(feature = "sysctl")]
pub use sysctl::sysctl;
#[cfg(feature = "vmstat")]
pub use vmstat::vmstat;

// Re-export batch 6 utilities
#[cfg(feature = "acpi")]
pub use acpi::acpi;
#[cfg(feature = "cal")]
pub use cal::cal;
#[cfg(feature = "fallocate")]
pub use fallocate::fallocate;
#[cfg(feature = "hwclock")]
pub use hwclock::hwclock;
#[cfg(feature = "mkswap")]
pub use mkswap::mkswap;
#[cfg(feature = "nologin")]
pub use nologin::nologin;
#[cfg(feature = "shuf")]
pub use shuf::shuf;
#[cfg(feature = "watch")]
pub use watch::watch;

// Re-export batch 7 utilities
#[cfg(feature = "chvt")]
pub use chvt::chvt;
#[cfg(feature = "dnsdomainname")]
pub use dnsdomainname::dnsdomainname;
#[cfg(feature = "fgconsole")]
pub use fgconsole::fgconsole;
#[cfg(feature = "lspci")]
pub use lspci::lspci;
#[cfg(feature = "lsusb")]
pub use lsusb::lsusb;
#[cfg(feature = "pivot_root")]
pub use pivot_root::pivot_root;
#[cfg(feature = "rfkill")]
pub use rfkill::rfkill;

// Re-export batch 8 utilities
#[cfg(feature = "blkdiscard")]
pub use blkdiscard::blkdiscard;
#[cfg(feature = "blockdev")]
pub use blockdev::blockdev;
#[cfg(feature = "eject")]
pub use eject::eject;
#[cfg(feature = "login")]
pub use login::login;
#[cfg(feature = "modinfo")]
pub use modinfo::modinfo;
#[cfg(feature = "readahead")]
pub use readahead_cmd::readahead_cmd;
#[cfg(feature = "rtcwake")]
pub use rtcwake::rtcwake;
#[cfg(feature = "su")]
pub use su::su;
#[cfg(feature = "ulimit")]
pub use ulimit::ulimit;

// Re-export batch 9 utilities
#[cfg(feature = "deallocvt")]
pub use deallocvt::deallocvt;
#[cfg(feature = "devmem")]
pub use devmem::devmem;
#[cfg(feature = "freeramdisk")]
pub use freeramdisk::freeramdisk;
#[cfg(feature = "fsfreeze")]
pub use fsfreeze::fsfreeze;
#[cfg(feature = "gpiodetect")]
pub use gpiodetect::gpiodetect;
#[cfg(feature = "gpiofind")]
pub use gpiofind::gpiofind;
#[cfg(feature = "gpioget")]
pub use gpioget::gpioget;
#[cfg(feature = "gpioinfo")]
pub use gpioinfo::gpioinfo;
#[cfg(feature = "gpioset")]
pub use gpioset::gpioset;
#[cfg(feature = "i2cdetect")]
pub use i2cdetect::i2cdetect;

// Re-export batch 10 utilities
#[cfg(feature = "i2cdump")]
pub use i2cdump::i2cdump;
#[cfg(feature = "i2cget")]
pub use i2cget::i2cget;
#[cfg(feature = "i2cset")]
pub use i2cset::i2cset;
#[cfg(feature = "i2ctransfer")]
pub use i2ctransfer::i2ctransfer;
#[cfg(feature = "inotifyd")]
pub use inotifyd::inotifyd;
#[cfg(feature = "linux32")]
pub use linux32::linux32;
#[cfg(feature = "openvt")]
pub use openvt::openvt;
#[cfg(feature = "partprobe")]
pub use partprobe::partprobe;
