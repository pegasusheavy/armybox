# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-01-03

### Added

#### Terminal Multiplexer
- `screen` - GNU screen-compatible terminal multiplexer
- `tmux` - Alias for screen functionality
- Session management (create, list, attach, detach)
- PTY-based terminal emulation
- Ctrl+A prefix for in-session commands
- Window creation and navigation

#### Documentation & SEO
- Comprehensive documentation website at pegasusheavy.github.io/armybox
- Schema.org structured data for SEO/AEO
- FAQ schema with common questions
- Sitemap and robots.txt
- Open Graph and Twitter Card meta tags
- GitHub Pages deployment workflow

### Changed
- Updated applet count to **293** (from 291)
- Updated all documentation with latest statistics
- Improved comparison tables with terminal multiplexer support

### Fixed
- Minor code cleanup and organization

## [0.2.0] - 2026-01-02

### Added

#### 100% Toybox Compatibility
- All 238 Toybox commands now supported
- 53 additional utilities beyond Toybox
- Drop-in replacement for Toybox deployments

#### POSIX Compliance Audit
- Complete POSIX.1-2017 compliance for core utilities
- `sed` - Added d, p, a, i, c commands and address ranges
- `awk` - Full pattern processing implementation
- `sort` - Added -k (key field) and -t (field separator)
- `cut` - Added -b (bytes) and -c (characters)
- `cp` - Added -i (interactive) and -p (preserve)
- `rm` - Added -i (interactive)

#### Networking Stack (35+ commands)
- `wget`, `nc`/`netcat`, `ping`, `ping6`, `traceroute`, `traceroute6`
- `host`, `nslookup`, `ifconfig`, `netstat`, `route`
- `ss`, `arp`, `arping`, `ip`, `ipaddr`, `iplink`, `ipneigh`, `iproute`, `iprule`
- `nameif`, `slattach`, `vconfig`, `telnet`, `brctl`, `tunctl`, `ether-wake`
- `ifup`, `ifdown`, `tftp`, `ftpget`, `ftpput`, `ipcalc`
- `nbd-client`, `nbd-server`, `httpd`, `sntp`

#### Init System
- `init` - PID 1 system initialization
- `telinit` - Change runlevel
- `runlevel` - Print current runlevel
- `getty` - Open terminal for login
- `sulogin` - Single-user login
- `linuxrc` - Init for initramfs
- `/etc/inittab` parsing
- Runlevel management (0-6, S)

#### Vi Editor
- Full modal editing (normal, insert, command-line modes)
- Movement commands (h, j, k, l, w, b, e, 0, $, gg, G)
- Editing commands (i, a, o, O, x, dd, yy, p, P, u)
- Search and replace (/, ?, n, N, :s)
- File operations (:w, :q, :wq, :q!, :e)
- Visual selection (v, V)

#### Archive Utilities
- `bunzip2`, `bzcat`, `bzip2`
- `cpio` - Copy files to/from archives
- `gunzip`, `gzip`, `zcat`
- `tar` - Full create, extract, list support
- `unxz`, `xz`, `xzcat`
- `unzip`
- `compress`, `uncompress` - LZW compression

#### System Utilities
- `acpi`, `cal`, `top`, `blkdiscard`, `blockdev`
- `chattr`, `cksum`, `crc32`, `deallocvt`, `devmem`
- `egrep`, `eject`, `fallocate`, `fgrep`, `freeramdisk`
- `fsfreeze`, `fstype`, `getopt`
- `gpiodetect`, `gpiofind`, `gpioget`, `gpioinfo`, `gpioset`
- `hd`, `help`, `hexedit`, `httpd`, `hwclock`
- `i2cdetect`, `i2cdump`, `i2cget`, `i2cset`, `i2ctransfer`
- `iconv`, `inotifyd`, `iorenice`, `iotop`
- `killall5`, `linux32`, `login`, `lsattr`, `lspci`, `lsusb`
- `makedevs`, `mcookie`, `memeater`, `microcom`, `mix`
- `mkpasswd`, `mkswap`, `modinfo`
- `nbd-client`, `nbd-server`, `nologin`, `nsenter`
- `oneit`, `openvt`, `partprobe`, `pmap`, `prlimit`
- `pwgen`, `readelf`, `reset`, `rtcwake`, `setfattr`
- `sha1sum`, `sha224sum`, `sha256sum`, `sha384sum`, `sha3sum`, `sha512sum`
- `shuf`, `sntp`, `su`, `switch_root`, `ts`, `tsort`
- `uclampset`, `ulimit`, `unicode`, `unshare`
- `uudecode`, `uuencode`, `uuidgen`, `vmstat`
- `watch`, `watchdog`

#### APK Package Manager (Optional)
- Alpine Linux package management support
- Feature-gated with `apk` feature flag

### Changed
- Binary size reduced to **108 KB** (from 220 KB)
- Per-applet efficiency improved to **~380 bytes** (from ~1.3 KB)
- Updated GitHub repository to pegasusheavy/armybox

### Performance
- Criterion-based benchmarking suite
- Dispatch overhead benchmarks
- Throughput benchmarks for I/O-bound commands
- Text processing utility benchmarks

## [0.1.0] - 2026-01-01

### Added

#### Core Features
- Multi-call binary architecture with 163+ applets
- Applet dispatch system with O(1) static lookup
- High-performance native implementations for hot-path applets
- Cross-compilation support for musl, ARM, MIPS, RISC-V

#### Shell
- ash-compatible shell with POSIX compliance
- Pipes, redirections (`>`, `>>`, `<`, `2>`, `&>`)
- Variables (`$var`, `${var}`, `${var:-default}`)
- Control structures (`if`, `while`, `for`, `case`)
- Arithmetic expansion (`$((expr))`)
- Command substitution (`$(cmd)`, backticks)
- Background jobs (`&`) and subshells (`()`)
- 25+ shell builtins

#### Coreutils
- File operations: `cat`, `cp`, `mv`, `rm`, `mkdir`, `rmdir`, `touch`, `chmod`, `chown`, `ln`, `ls`, etc.
- Text processing: `head`, `tail`, `wc`, `sort`, `uniq`, `cut`, `paste`, `tr`, `tee`, etc.
- System info: `uname`, `whoami`, `id`, `groups`, `hostname`, `env`, `date`, etc.
- Checksums: `md5sum`, `sha1sum`, `sha256sum`, `sha512sum`, `sha3sum`

#### Text Processing
- `grep`/`egrep`/`fgrep` with regex support
- `find` with multiple predicates
- `sed` with basic commands
- `awk` interpreter

#### System Utilities
- `ps`, `pgrep`, `pkill`
- `free`, `watch`
- `mount`, `umount`
- `dmesg`, `lsmod`
- `halt`, `reboot`, `poweroff`

#### Extra Utilities
- `dd`, `hexdump`
- `diff`, `cmp`
- `chroot`, `install`
- `dos2unix`, `unix2dos`
- `rev`, `tac`
- `which`, `usleep`

### Packaging
- Static musl builds for all architectures
- Docker images (scratch and Alpine-based)
- Debian/Ubuntu .deb packages
- RPM packages for Fedora/RHEL
- Arch Linux PKGBUILD
- Alpine Linux APKBUILD
- Universal install script

### Documentation
- Comprehensive README
- Benchmark documentation
- Docker usage guide
- Packaging instructions

[Unreleased]: https://github.com/pegasusheavy/armybox/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/pegasusheavy/armybox/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/pegasusheavy/armybox/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/pegasusheavy/armybox/releases/tag/v0.1.0
