//! Native no_std applet implementations
//!
//! All applets implemented using libc directly, no std required.

mod file;
mod text;
mod system;
mod misc;
mod network;
mod archive;
mod editors;
mod init;
mod shell;
#[cfg(feature = "apk")]
mod package;

use crate::io;

/// Number of applets
pub const APPLET_COUNT: usize = 283;

/// Get argument as byte slice
#[inline]
pub unsafe fn get_arg(argv: *const *const u8, idx: i32) -> Option<&'static [u8]> {
    if argv.is_null() {
        return None;
    }
    let ptr = *argv.add(idx as usize);
    if ptr.is_null() {
        return None;
    }
    Some(io::cstr_to_slice(ptr))
}

/// Check if argument has option char
#[inline]
pub fn has_opt(arg: &[u8], opt: u8) -> bool {
    if arg.len() < 2 || arg[0] != b'-' {
        return false;
    }
    arg[1..].contains(&opt)
}

/// Check if argument is exactly this option
#[inline]
pub fn is_opt(arg: &[u8], opt: u8) -> bool {
    arg.len() == 2 && arg[0] == b'-' && arg[1] == opt
}

/// Check if argument looks like an option
#[inline]
pub fn is_option(arg: &[u8]) -> bool {
    arg.len() >= 2 && arg[0] == b'-'
}

/// Find an applet function by name
pub fn find_applet(name: &[u8]) -> Option<fn(i32, *const *const u8) -> i32> {
    // File utilities
    if name == b"cat" { return Some(file::cat); }
    if name == b"cp" { return Some(file::cp); }
    if name == b"mv" { return Some(file::mv); }
    if name == b"rm" { return Some(file::rm); }
    if name == b"mkdir" { return Some(file::mkdir); }
    if name == b"rmdir" { return Some(file::rmdir); }
    if name == b"touch" { return Some(file::touch); }
    if name == b"ln" { return Some(file::ln); }
    if name == b"ls" { return Some(file::ls); }
    if name == b"pwd" { return Some(file::pwd); }
    if name == b"chmod" { return Some(file::chmod); }
    if name == b"chown" { return Some(file::chown); }
    if name == b"chgrp" { return Some(file::chgrp); }
    if name == b"stat" { return Some(file::stat); }
    if name == b"readlink" { return Some(file::readlink); }
    if name == b"realpath" { return Some(file::realpath); }
    if name == b"basename" { return Some(file::basename); }
    if name == b"dirname" { return Some(file::dirname); }
    if name == b"sync" { return Some(file::sync_cmd); }
    if name == b"link" { return Some(file::link); }
    if name == b"unlink" { return Some(file::unlink); }
    if name == b"dd" { return Some(file::dd); }
    if name == b"mktemp" { return Some(file::mktemp); }
    if name == b"mkfifo" { return Some(file::mkfifo); }
    if name == b"mknod" { return Some(file::mknod); }
    if name == b"split" { return Some(file::split); }
    if name == b"install" { return Some(file::install); }
    if name == b"truncate" { return Some(file::truncate); }
    if name == b"shred" { return Some(file::shred); }
    if name == b"file" { return Some(file::file); }
    if name == b"xargs" { return Some(file::xargs); }
    if name == b"patch" { return Some(file::patch); }
    if name == b"find" { return Some(file::find); }
    if name == b"cd" { return Some(file::cd); }

    // Text utilities
    if name == b"echo" { return Some(text::echo); }
    if name == b"printf" { return Some(text::printf); }
    if name == b"head" { return Some(text::head); }
    if name == b"tail" { return Some(text::tail); }
    if name == b"wc" { return Some(text::wc); }
    if name == b"tee" { return Some(text::tee); }
    if name == b"tac" { return Some(text::tac); }
    if name == b"rev" { return Some(text::rev); }
    if name == b"yes" { return Some(text::yes); }
    if name == b"seq" { return Some(text::seq); }
    if name == b"nl" { return Some(text::nl); }
    if name == b"tr" { return Some(text::tr); }
    if name == b"cut" { return Some(text::cut); }
    if name == b"paste" { return Some(text::paste); }
    if name == b"sort" { return Some(text::sort); }
    if name == b"uniq" { return Some(text::uniq); }
    if name == b"grep" { return Some(text::grep); }
    if name == b"egrep" { return Some(text::egrep); }
    if name == b"fgrep" { return Some(text::fgrep); }
    if name == b"sed" { return Some(text::sed); }
    if name == b"awk" { return Some(text::awk); }
    if name == b"comm" { return Some(text::comm); }
    if name == b"expand" { return Some(text::expand); }
    if name == b"unexpand" { return Some(text::unexpand); }
    if name == b"fold" { return Some(text::fold); }
    if name == b"fmt" { return Some(text::fmt); }
    if name == b"strings" { return Some(text::strings); }
    if name == b"dos2unix" { return Some(text::dos2unix); }
    if name == b"unix2dos" { return Some(text::unix2dos); }

    // System utilities
    if name == b"uname" { return Some(system::uname); }
    if name == b"hostname" { return Some(system::hostname); }
    if name == b"whoami" { return Some(system::whoami); }
    if name == b"id" { return Some(system::id); }
    if name == b"groups" { return Some(system::groups); }
    if name == b"who" { return Some(system::who); }
    if name == b"w" { return Some(system::w); }
    if name == b"users" { return Some(system::users); }
    if name == b"arch" { return Some(system::arch); }
    if name == b"date" { return Some(system::date); }
    if name == b"env" { return Some(system::env); }
    if name == b"printenv" { return Some(system::printenv); }
    if name == b"tty" { return Some(system::tty); }
    if name == b"kill" { return Some(system::kill); }
    if name == b"killall" { return Some(system::killall); }
    if name == b"killall5" { return Some(system::killall5); }
    if name == b"ps" { return Some(system::ps); }
    if name == b"pgrep" { return Some(system::pgrep); }
    if name == b"pkill" { return Some(system::pkill); }
    if name == b"pidof" { return Some(system::pidof); }
    if name == b"pwdx" { return Some(system::pwdx); }
    if name == b"sleep" { return Some(system::sleep); }
    if name == b"usleep" { return Some(system::usleep); }
    if name == b"uptime" { return Some(system::uptime); }
    if name == b"free" { return Some(system::free); }
    if name == b"df" { return Some(system::df); }
    if name == b"du" { return Some(system::du); }
    if name == b"mount" { return Some(system::mount); }
    if name == b"umount" { return Some(system::umount); }
    if name == b"mountpoint" { return Some(system::mountpoint); }
    if name == b"dmesg" { return Some(system::dmesg); }
    if name == b"halt" { return Some(system::halt); }
    if name == b"reboot" { return Some(system::reboot); }
    if name == b"poweroff" { return Some(system::poweroff); }
    if name == b"chroot" { return Some(system::chroot); }
    if name == b"nice" { return Some(system::nice); }
    if name == b"renice" { return Some(system::renice); }
    if name == b"nohup" { return Some(system::nohup); }
    if name == b"setsid" { return Some(system::setsid); }
    if name == b"timeout" { return Some(system::timeout); }
    if name == b"logname" { return Some(system::logname); }
    if name == b"logger" { return Some(system::logger); }
    if name == b"dnsdomainname" { return Some(system::dnsdomainname); }
    if name == b"hostid" { return Some(system::hostid); }
    if name == b"nproc" { return Some(system::nproc); }
    if name == b"fgconsole" { return Some(system::fgconsole); }
    if name == b"chvt" { return Some(system::chvt); }
    if name == b"flock" { return Some(system::flock); }
    if name == b"fsync" { return Some(system::fsync_cmd); }
    if name == b"sysctl" { return Some(system::sysctl); }
    if name == b"swapoff" { return Some(system::swapoff); }
    if name == b"swapon" { return Some(system::swapon); }
    if name == b"blkid" { return Some(system::blkid); }
    if name == b"losetup" { return Some(system::losetup); }
    if name == b"insmod" { return Some(system::insmod); }
    if name == b"rmmod" { return Some(system::rmmod); }
    if name == b"modprobe" { return Some(system::modprobe); }
    if name == b"lsmod" { return Some(system::lsmod); }
    if name == b"pivot_root" { return Some(system::pivot_root); }
    if name == b"readahead" { return Some(system::readahead_cmd); }
    if name == b"taskset" { return Some(system::taskset); }
    if name == b"rfkill" { return Some(system::rfkill); }
    if name == b"ionice" { return Some(system::ionice); }
    if name == b"chrt" { return Some(system::chrt); }
    // New toybox applets
    if name == b"acpi" { return Some(system::acpi); }
    if name == b"cal" { return Some(system::cal); }
    if name == b"top" { return Some(system::top); }
    if name == b"vmstat" { return Some(system::vmstat); }
    if name == b"watch" { return Some(system::watch); }
    if name == b"hwclock" { return Some(system::hwclock); }
    if name == b"fallocate" { return Some(system::fallocate); }
    if name == b"shuf" { return Some(system::shuf); }
    if name == b"mkswap" { return Some(system::mkswap); }
    if name == b"nologin" { return Some(system::nologin); }
    if name == b"nsenter" { return Some(system::nsenter); }
    if name == b"unshare" { return Some(system::unshare); }
    if name == b"pmap" { return Some(system::pmap); }
    if name == b"su" { return Some(system::su); }
    if name == b"login" { return Some(system::login); }
    if name == b"eject" { return Some(system::eject); }
    if name == b"blockdev" { return Some(system::blockdev); }
    if name == b"prlimit" { return Some(system::prlimit); }
    if name == b"rtcwake" { return Some(system::rtcwake); }
    if name == b"uclampset" { return Some(system::uclampset); }
    if name == b"ulimit" { return Some(system::ulimit); }

    // Misc utilities
    if name == b"true" { return Some(misc::r#true); }
    if name == b"false" { return Some(misc::r#false); }
    if name == b":" { return Some(misc::colon); }
    if name == b"test" { return Some(misc::test); }
    if name == b"[" { return Some(misc::bracket); }
    if name == b"clear" { return Some(misc::clear); }
    if name == b"reset" { return Some(misc::reset); }
    if name == b"which" { return Some(misc::which); }
    if name == b"expr" { return Some(misc::expr); }
    if name == b"time" { return Some(misc::time); }
    if name == b"mesg" { return Some(misc::mesg); }
    if name == b"getconf" { return Some(misc::getconf); }
    if name == b"factor" { return Some(misc::factor); }
    if name == b"base64" { return Some(misc::base64); }
    if name == b"base32" { return Some(misc::base32); }
    if name == b"cmp" { return Some(misc::cmp); }
    if name == b"diff" { return Some(misc::diff); }
    if name == b"od" { return Some(misc::od); }
    if name == b"hexdump" { return Some(misc::hexdump); }
    if name == b"hd" { return Some(misc::hd); }
    if name == b"xxd" { return Some(misc::xxd); }
    if name == b"md5sum" { return Some(misc::md5sum); }
    if name == b"sha1sum" { return Some(misc::sha1sum); }
    if name == b"sha224sum" { return Some(misc::sha224sum); }
    if name == b"sha256sum" { return Some(misc::sha256sum); }
    if name == b"sha384sum" { return Some(misc::sha384sum); }
    if name == b"sha512sum" { return Some(misc::sha512sum); }
    if name == b"sha3sum" { return Some(misc::sha3sum); }
    if name == b"cksum" { return Some(misc::cksum); }
    if name == b"crc32" { return Some(misc::crc32); }
    if name == b"ascii" { return Some(misc::ascii); }
    if name == b"iconv" { return Some(misc::iconv); }
    if name == b"tsort" { return Some(misc::tsort); }
    if name == b"getopt" { return Some(misc::getopt); }
    if name == b"count" { return Some(misc::count); }
    if name == b"unicode" { return Some(misc::unicode); }
    if name == b"ts" { return Some(misc::ts); }
    if name == b"uuidgen" { return Some(misc::uuidgen); }
    if name == b"mcookie" { return Some(misc::mcookie); }
    if name == b"pwgen" { return Some(misc::pwgen); }
    if name == b"uuencode" { return Some(misc::uuencode); }
    if name == b"uudecode" { return Some(misc::uudecode); }

    // Network utilities
    if name == b"wget" { return Some(network::wget); }
    if name == b"nc" { return Some(network::nc); }
    if name == b"netcat" { return Some(network::netcat); }
    if name == b"ping" { return Some(network::ping); }
    if name == b"ping6" { return Some(network::ping6); }
    if name == b"traceroute" { return Some(network::traceroute); }
    if name == b"traceroute6" { return Some(network::traceroute6); }
    if name == b"host" { return Some(network::host); }
    if name == b"nslookup" { return Some(network::nslookup); }
    if name == b"ifconfig" { return Some(network::ifconfig); }
    if name == b"netstat" { return Some(network::netstat); }
    if name == b"route" { return Some(network::route); }
    if name == b"tftp" { return Some(network::tftp); }
    if name == b"ftpget" { return Some(network::ftpget); }
    if name == b"ftpput" { return Some(network::ftpput); }
    if name == b"ipcalc" { return Some(network::ipcalc); }
    if name == b"brctl" { return Some(network::brctl); }
    if name == b"tunctl" { return Some(network::tunctl); }
    if name == b"ether-wake" { return Some(network::ether_wake); }
    if name == b"ifup" { return Some(network::ifup); }
    if name == b"ifdown" { return Some(network::ifdown); }
    if name == b"ss" { return Some(network::ss); }
    if name == b"arp" { return Some(network::arp); }
    if name == b"arping" { return Some(network::arping); }
    if name == b"ip" { return Some(network::ip); }
    if name == b"ipaddr" { return Some(network::ipaddr); }
    if name == b"iplink" { return Some(network::iplink); }
    if name == b"ipneigh" { return Some(network::ipneigh); }
    if name == b"iproute" { return Some(network::iproute); }
    if name == b"iprule" { return Some(network::iprule); }
    if name == b"nameif" { return Some(network::nameif); }
    if name == b"slattach" { return Some(network::slattach); }
    if name == b"vconfig" { return Some(network::vconfig); }
    if name == b"telnet" { return Some(network::telnet); }
    if name == b"httpd" { return Some(network::httpd); }
    if name == b"sntp" { return Some(network::sntp); }
    if name == b"microcom" { return Some(network::microcom); }

    // Archive utilities
    if name == b"tar" { return Some(archive::tar); }
    if name == b"gzip" { return Some(archive::gzip); }
    if name == b"gunzip" { return Some(archive::gunzip); }
    if name == b"zcat" { return Some(archive::zcat); }
    if name == b"bzip2" { return Some(archive::bzip2); }
    if name == b"bunzip2" { return Some(archive::bunzip2); }
    if name == b"bzcat" { return Some(archive::bzcat); }
    if name == b"xz" { return Some(archive::xz); }
    if name == b"unxz" { return Some(archive::unxz); }
    if name == b"xzcat" { return Some(archive::xzcat); }
    if name == b"cpio" { return Some(archive::cpio); }
    if name == b"unzip" { return Some(archive::unzip); }
    if name == b"compress" { return Some(archive::compress); }
    if name == b"uncompress" { return Some(archive::uncompress); }

    // Editors
    if name == b"vi" { return Some(editors::vi); }
    if name == b"view" { return Some(editors::view); }
    if name == b"hexedit" { return Some(editors::hexedit); }

    // Init system
    if name == b"init" { return Some(init::init); }
    if name == b"telinit" { return Some(init::telinit); }
    if name == b"runlevel" { return Some(init::runlevel); }
    if name == b"getty" { return Some(init::getty); }
    if name == b"sulogin" { return Some(init::sulogin); }
    if name == b"linuxrc" { return Some(init::init); }
    if name == b"oneit" { return Some(init::oneit); }
    if name == b"switch_root" { return Some(init::switch_root); }
    if name == b"watchdog" { return Some(init::watchdog); }

    // Shell
    if name == b"sh" { return Some(shell::sh); }
    if name == b"ash" { return Some(shell::ash); }
    if name == b"dash" { return Some(shell::dash); }

    // APK package manager (feature gated)
    #[cfg(feature = "apk")]
    if name == b"apk" { return Some(package::apk); }

    // Additional toybox applets
    if name == b"blkdiscard" { return Some(system::blkdiscard); }
    if name == b"chattr" { return Some(file::chattr); }
    if name == b"lsattr" { return Some(file::lsattr); }
    if name == b"deallocvt" { return Some(system::deallocvt); }
    if name == b"devmem" { return Some(system::devmem); }
    if name == b"freeramdisk" { return Some(system::freeramdisk); }
    if name == b"fsfreeze" { return Some(system::fsfreeze); }
    if name == b"fstype" { return Some(file::fstype); }
    if name == b"gpiodetect" { return Some(system::gpiodetect); }
    if name == b"gpiofind" { return Some(system::gpiofind); }
    if name == b"gpioget" { return Some(system::gpioget); }
    if name == b"gpioinfo" { return Some(system::gpioinfo); }
    if name == b"gpioset" { return Some(system::gpioset); }
    if name == b"help" { return Some(misc::help); }
    if name == b"i2cdetect" { return Some(system::i2cdetect); }
    if name == b"i2cdump" { return Some(system::i2cdump); }
    if name == b"i2cget" { return Some(system::i2cget); }
    if name == b"i2cset" { return Some(system::i2cset); }
    if name == b"i2ctransfer" { return Some(system::i2ctransfer); }
    if name == b"inotifyd" { return Some(system::inotifyd); }
    if name == b"iorenice" { return Some(system::iorenice); }
    if name == b"iotop" { return Some(system::iotop); }
    if name == b"linux32" { return Some(system::linux32); }
    if name == b"lspci" { return Some(system::lspci); }
    if name == b"lsusb" { return Some(system::lsusb); }
    if name == b"makedevs" { return Some(file::makedevs); }
    if name == b"memeater" { return Some(misc::memeater); }
    if name == b"mix" { return Some(misc::mix); }
    if name == b"mkpasswd" { return Some(misc::mkpasswd); }
    if name == b"modinfo" { return Some(system::modinfo); }
    if name == b"nbd-client" { return Some(network::nbd_client); }
    if name == b"nbd-server" { return Some(network::nbd_server); }
    if name == b"openvt" { return Some(system::openvt); }
    if name == b"partprobe" { return Some(system::partprobe); }
    if name == b"readelf" { return Some(misc::readelf); }
    if name == b"setfattr" { return Some(file::setfattr); }
    if name == b"toybox" { return Some(misc::toybox); }
    if name == b"unicode" { return Some(misc::unicode); }

    None
}

/// List all applet names
pub fn list_applets() {
    io::write_str(1, b"Currently defined applets:\n");
    // Alphabetically sorted list
    let names: &[&[u8]] = &[
        b"acpi", b"arch", b"arp", b"arping", b"ascii", b"ash", b"awk",
        b"base32", b"base64", b"basename", b"blkdiscard", b"blkid", b"blockdev", b"brctl",
        b"bunzip2", b"bzcat", b"bzip2",
        b"cal", b"cat", b"cd", b"chattr", b"chgrp", b"chmod", b"chown", b"chroot", b"chrt", b"chvt",
        b"cksum", b"clear", b"cmp", b"comm", b"compress", b"count", b"cp", b"cpio", b"crc32", b"cut",
        b"dash", b"date", b"dd", b"deallocvt", b"devmem", b"df", b"diff", b"dirname", b"dmesg", b"dnsdomainname", b"dos2unix", b"du",
        b"echo", b"egrep", b"eject", b"env", b"expand", b"expr",
        b"factor", b"fallocate", b"false", b"fgconsole", b"fgrep", b"file", b"find", b"flock", b"fmt", b"fold",
        b"free", b"freeramdisk", b"fsfreeze", b"fstype", b"fsync", b"ftpget", b"ftpput",
        b"getconf", b"getopt", b"getty", b"gpiodetect", b"gpiofind", b"gpioget", b"gpioinfo", b"gpioset",
        b"grep", b"groups", b"gunzip", b"gzip",
        b"halt", b"hd", b"head", b"help", b"hexdump", b"hexedit", b"host", b"hostid", b"hostname", b"httpd", b"hwclock",
        b"i2cdetect", b"i2cdump", b"i2cget", b"i2cset", b"i2ctransfer",
        b"iconv", b"id", b"ifconfig", b"ifdown", b"ifup", b"init", b"inotifyd", b"insmod", b"install", b"ionice", b"iorenice", b"iotop", b"ip",
        b"ipaddr", b"ipcalc", b"iplink", b"ipneigh", b"iproute", b"iprule",
        b"kill", b"killall", b"killall5",
        b"link", b"linux32", b"linuxrc", b"ln", b"logger", b"login", b"logname", b"losetup", b"ls", b"lsattr", b"lsmod", b"lspci", b"lsusb",
        b"makedevs", b"mcookie", b"md5sum", b"memeater", b"mesg", b"microcom", b"mix", b"mkdir", b"mkfifo", b"mknod", b"mkpasswd", b"mkswap", b"mktemp",
        b"modinfo", b"modprobe", b"mount", b"mountpoint", b"mv",
        b"nameif", b"nbd-client", b"nbd-server", b"nc", b"netcat", b"netstat", b"nice", b"nl", b"nohup", b"nologin", b"nproc", b"nsenter", b"nslookup",
        b"od", b"oneit", b"openvt",
        b"partprobe", b"paste", b"patch", b"pgrep", b"pidof", b"ping", b"ping6", b"pivot_root", b"pkill", b"pmap", b"poweroff",
        b"printenv", b"printf", b"prlimit", b"ps", b"pwd", b"pwdx", b"pwgen",
        b"readahead", b"readelf", b"readlink", b"realpath", b"reboot", b"renice", b"reset", b"rev", b"rfkill", b"rm", b"rmdir",
        b"rmmod", b"route", b"rtcwake", b"runlevel",
        b"sed", b"seq", b"setfattr", b"setsid", b"sh", b"sha1sum", b"sha224sum", b"sha256sum", b"sha384sum", b"sha3sum", b"sha512sum",
        b"shred", b"shuf", b"sleep", b"slattach", b"sntp", b"sort", b"split", b"ss", b"stat", b"strings", b"su", b"sulogin",
        b"swapoff", b"swapon", b"switch_root", b"sync", b"sysctl",
        b"tac", b"tail", b"tar", b"taskset", b"tee", b"telinit", b"telnet", b"test", b"tftp", b"time", b"timeout",
        b"top", b"touch", b"toybox", b"tr", b"traceroute", b"traceroute6", b"true", b"truncate", b"ts", b"tsort", b"tty", b"tunctl",
        b"uclampset", b"ulimit", b"umount", b"uname", b"uncompress", b"unexpand", b"unicode", b"uniq", b"unix2dos", b"unlink",
        b"unshare", b"unxz", b"unzip", b"uptime", b"users", b"usleep", b"uudecode", b"uuencode", b"uuidgen",
        b"vconfig", b"vi", b"view", b"vmstat",
        b"w", b"watch", b"watchdog", b"wc", b"wget", b"which", b"who", b"whoami",
        b"xargs", b"xxd", b"xz", b"xzcat",
        b"yes",
        b"zcat",
    ];

    for name in names {
        io::write_all(1, name);
        io::write_str(1, b"\n");
    }
}
