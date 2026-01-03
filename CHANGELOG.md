# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-02

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

#### Init System
- PID 1 init process
- `/etc/inittab` parsing
- Runlevel management (0-6, S)
- Service spawning and respawning
- Signal handling (SIGCHLD, SIGTERM, SIGINT, SIGHUP)
- Essential filesystem mounting

#### Coreutils (via uutils + native)
- File operations: `cat`, `cp`, `mv`, `rm`, `mkdir`, `rmdir`, `touch`, `chmod`, `chown`, `ln`, `ls`, etc.
- Text processing: `head`, `tail`, `wc`, `sort`, `uniq`, `cut`, `paste`, `tr`, `tee`, etc.
- System info: `uname`, `whoami`, `id`, `groups`, `hostname`, `env`, `date`, etc.
- Checksums: `md5sum`, `sha1sum`, `sha256sum`, `sha512sum`, `sha3sum`

#### Compression & Archives
- gzip/gunzip/zcat
- bzip2/bunzip2/bzcat
- xz/unxz/xzcat
- tar (create, extract, list)
- zip/unzip
- cpio

#### Text Processing
- `grep`/`egrep`/`fgrep` with regex support
- `find` with multiple predicates
- `sed` with basic commands
- `awk` interpreter
- `less`/`more` pagers
- `strings`, `xargs`

#### System Utilities
- `ps`, `pgrep`, `pkill`
- `free`, `top`, `watch`
- `mount`, `umount`
- `dmesg`, `lsmod`
- `halt`, `reboot`, `poweroff`

#### Networking
- `ping`, `traceroute`
- `netstat`, `ss`
- `ifconfig`, `ip`
- `wget`, `nc`/`netcat`
- `host`, `nslookup`, `dig`
- `arp`, `arping`, `route`
- `telnet`, `dnsdomainname`

#### Extra Utilities
- `dd`, `hexdump`, `xxd`
- `diff`, `cmp`
- `chroot`, `install`
- `dos2unix`, `unix2dos`
- `rev`, `tac`
- `which`, `usleep`
- `uuencode`, `uudecode`

### Performance
- 256KB I/O buffers for throughput
- `memchr` SIMD-optimized string search
- Regex caching with LRU eviction
- Memory-mapped file I/O where beneficial
- Native fast implementations bypass clap overhead

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
- Angular documentation website

[0.1.0]: https://github.com/PegasusHeavyIndustries/armybox/releases/tag/v0.1.0
