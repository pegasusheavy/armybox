# Armybox - Busybox/Toybox Clone in Rust

## Project Goal
100% feature compliance with both BusyBox and Toybox, implemented in safe Rust where possible.

## Current Progress
- **Applets Implemented**: 201 (of 300+ BusyBox / 200+ Toybox)
- **Binary Size**: 323 KB (release, stripped, LTO) / ~117 KB (UPX compressed)
- **Size per Applet**: ~1.6 KB
- **Architecture**: Multi-call binary with symlink dispatch ✅
- **Testing**: Property tests, benchmarks, fuzz targets ✅
- **POSIX Compliance**: Full for core utilities ✅

### Benchmark Results (vs BusyBox)
| Applet | Speedup | Notes |
|--------|---------|-------|
| true (startup) | 7.46x faster | Dispatch overhead only |
| cut | 32.87x faster | Delimiter parsing |
| uniq | 3.66x faster | Line deduplication |
| grep -v | 1.71x faster | Inverted match |
| wc | 1.53x faster | Word/line counting |

**Criterion Benchmarks** (run with `cd crates/benchmarks && cargo bench`):
| Benchmark | Time | Throughput |
|-----------|------|------------|
| grep literal (100K) | 20.7ms | 241 MiB/s |
| grep regex (100K) | 20.6ms | 242 MiB/s |
| sed substitute (10K) | 5.2ms | 96 MiB/s |
| sort numeric (10K) | 6.4ms | - |
| awk print (10K) | 3.3ms | - |
| cat (1MB) | 1.0ms | 932 MiB/s |
| wc (1MB) | 1.0ms | 928 MiB/s |

## Legend
- `[ ]` - Not started
- `[~]` - In progress
- `[x]` - Complete
- `[B]` - BusyBox only
- `[T]` - Toybox only
- `[BT]` - Both BusyBox and Toybox

---

## Architecture Tasks
- [x] Multi-call binary infrastructure (symlink/applet dispatch)
- [x] Unified argument parsing framework (clap)
- [x] Common utility library (file ops, string handling, etc.)
- [x] Build system for selective applet compilation (Cargo features)
- [x] Cross-compilation support (musl, ARM, MIPS, etc.)
- [x] Static linking configuration
- [x] Test harness with compliance checking against reference implementations
- [x] Benchmarking infrastructure (criterion)
- [x] Fuzzing infrastructure (cargo-fuzz)
- [x] Property-based testing (proptest)
- [ ] Documentation generation

---

## Applets by Category

### Archive Utilities
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [ ] | ar | [B] | Create/modify/extract archives |
| [x] | bunzip2 | [BT] | Decompress bzip2 files |
| [x] | bzcat | [BT] | Decompress to stdout |
| [x] | bzip2 | [BT] | Compress files with bzip2 |
| [x] | compress | [B] | Compress files (LZW) (stub) |
| [x] | cpio | [BT] | Copy files to/from archives |
| [ ] | dpkg | [B] | Debian package manager |
| [ ] | dpkg-deb | [B] | Debian package archive tool |
| [x] | gunzip | [BT] | Decompress gzip files |
| [x] | gzip | [BT] | Compress files with gzip |
| [ ] | lzcat | [B] | Decompress LZMA to stdout |
| [ ] | lzma | [B] | LZMA compression |
| [ ] | lzop | [B] | LZO compression |
| [ ] | lzopcat | [B] | Decompress LZO to stdout |
| [ ] | rpm | [B] | RPM package operations |
| [ ] | rpm2cpio | [B] | Convert RPM to cpio |
| [x] | tar | [BT] | Archive utility |
| [x] | uncompress | [B] | Decompress .Z files (stub) |
| [ ] | unlzma | [B] | Decompress LZMA files |
| [ ] | unlzop | [B] | Decompress LZO files |
| [x] | unxz | [BT] | Decompress XZ files |
| [x] | unzip | [BT] | Extract ZIP archives |
| [x] | xz | [T] | XZ compression |
| [x] | xzcat | [BT] | Decompress XZ to stdout |
| [x] | zcat | [BT] | Decompress gzip to stdout |
| [ ] | zip | [T] | Create ZIP archives |

### Coreutils / File Operations
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [x] | basename | [BT] | Strip directory from filename |
| [x] | cat | [BT] | Concatenate files |
| [x] | chgrp | [BT] | Change group ownership |
| [x] | chmod | [BT] | Change file permissions |
| [x] | chown | [BT] | Change file ownership |
| [x] | chroot | [BT] | Run command with different root |
| [x] | cksum | [BT] | CRC checksum and byte count |
| [x] | comm | [BT] | Compare sorted files |
| [x] | cp | [BT] | Copy files |
| [x] | cut | [BT] | Remove sections from lines |
| [x] | date | [BT] | Print/set system date |
| [x] | dd | [BT] | Convert and copy files |
| [x] | df | [BT] | Report filesystem disk space |
| [x] | dirname | [BT] | Strip last component from path |
| [x] | dos2unix | [BT] | Convert DOS line endings |
| [x] | du | [BT] | Estimate file space usage |
| [x] | echo | [BT] | Display a line of text |
| [x] | env | [BT] | Run program in modified environment |
| [x] | expand | [BT] | Convert tabs to spaces |
| [x] | expr | [BT] | Evaluate expressions |
| [x] | factor | [BT] | Factor numbers |
| [x] | false | [BT] | Return false |
| [x] | fmt | [BT] | Simple text formatter |
| [x] | fold | [BT] | Wrap lines to specified width |
| [x] | groups | [BT] | Print group memberships |
| [x] | head | [BT] | Output first part of files |
| [x] | hostid | [BT] | Print host identifier |
| [x] | hostname | [BT] | Get/set hostname |
| [x] | id | [BT] | Print user identity |
| [x] | install | [BT] | Copy files with attributes |
| [x] | link | [BT] | Create hard link |
| [x] | ln | [BT] | Create links |
| [x] | logname | [BT] | Print login name |
| [x] | ls | [BT] | List directory contents |
| [x] | md5sum | [BT] | Compute MD5 checksums |
| [x] | mkdir | [BT] | Create directories |
| [x] | mkfifo | [BT] | Create named pipes |
| [x] | mknod | [BT] | Create special files |
| [x] | mktemp | [BT] | Create temporary file/directory |
| [x] | mv | [BT] | Move/rename files |
| [x] | nice | [BT] | Run with modified scheduling priority |
| [x] | nl | [BT] | Number lines |
| [x] | nohup | [BT] | Run immune to hangups |
| [x] | nproc | [BT] | Print number of processors |
| [x] | od | [BT] | Dump files in octal/other formats |
| [x] | paste | [BT] | Merge lines of files |
| [x] | printenv | [BT] | Print environment |
| [x] | printf | [BT] | Format and print data |
| [x] | pwd | [BT] | Print working directory |
| [x] | readlink | [BT] | Print resolved symbolic link |
| [x] | realpath | [BT] | Print resolved path |
| [x] | rm | [BT] | Remove files |
| [x] | rmdir | [BT] | Remove directories |
| [x] | seq | [BT] | Print sequence of numbers |
| [x] | sha1sum | [BT] | Compute SHA1 checksums |
| [x] | sha256sum | [BT] | Compute SHA256 checksums |
| [x] | sha512sum | [BT] | Compute SHA512 checksums |
| [x] | sha3sum | [T] | Compute SHA3 checksums |
| [x] | shred | [BT] | Overwrite file to hide contents |
| [x] | sleep | [BT] | Delay for specified time |
| [x] | sort | [BT] | Sort lines |
| [x] | split | [BT] | Split file into pieces |
| [x] | stat | [BT] | Display file status |
| [x] | stty | [BT] | Change terminal settings |
| [x] | sum | [B] | Checksum and count blocks |
| [x] | sync | [BT] | Flush filesystem buffers |
| [x] | tac | [BT] | Concatenate in reverse |
| [x] | tail | [BT] | Output last part of files |
| [x] | tee | [BT] | Read stdin, write stdout and files |
| [x] | test | [BT] | Evaluate conditional expression |
| [x] | [ | [BT] | Alias for test |
| [ ] | [[ | [T] | Extended test |
| [x] | timeout | [BT] | Run command with time limit |
| [x] | touch | [BT] | Change file timestamps |
| [x] | tr | [BT] | Translate characters |
| [x] | true | [BT] | Return true |
| [x] | truncate | [BT] | Shrink/extend file size |
| [x] | tty | [BT] | Print terminal name |
| [x] | uname | [BT] | Print system information |
| [x] | unexpand | [BT] | Convert spaces to tabs |
| [x] | uniq | [BT] | Report/omit repeated lines |
| [x] | unix2dos | [BT] | Convert Unix line endings |
| [x] | unlink | [BT] | Remove a single file |
| [x] | uptime | [BT] | Show system uptime |
| [x] | users | [B] | Print logged in users |
| [x] | usleep | [B] | Sleep in microseconds |
| [x] | uudecode | [BT] | Decode uuencoded file |
| [x] | uuencode | [BT] | Encode binary file |
| [x] | wc | [BT] | Word, line, byte count |
| [x] | which | [BT] | Locate command |
| [x] | who | [BT] | Show logged in users |
| [x] | whoami | [BT] | Print effective user ID |
| [x] | yes | [BT] | Output string repeatedly |

### Text Processing / Search
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [x] | awk | [BT] | Pattern scanning language |
| [x] | cmp | [BT] | Compare files byte by byte |
| [x] | diff | [BT] | Compare files line by line |
| [ ] | ed | [B] | Line editor |
| [x] | egrep | [BT] | Extended regex grep |
| [x] | fgrep | [BT] | Fixed string grep |
| [x] | find | [BT] | Search for files |
| [x] | grep | [BT] | Search file patterns |
| [x] | less | [B] | File pager |
| [x] | more | [B] | File pager |
| [x] | patch | [BT] | Apply diff to original |
| [x] | sed | [BT] | Stream editor |
| [x] | vi | [BT] | Visual editor |
| [x] | view | [BT] | Read-only vi |
| [x] | xargs | [BT] | Build command lines from stdin |

### Process Utilities
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [x] | chrt | [BT] | Manipulate real-time attributes |
| [ ] | fuser | [B] | Identify processes using files |
| [x] | ionice | [BT] | Set/get I/O scheduling |
| [x] | kill | [BT] | Send signals to processes |
| [x] | killall | [BT] | Kill processes by name |
| [ ] | killall5 | [B] | Send signal to all processes |
| [ ] | lsof | [T] | List open files |
| [x] | nice | [BT] | Run with modified priority |
| [x] | nohup | [BT] | Run immune to hangups |
| [x] | pgrep | [BT] | Look up processes by name |
| [x] | pidof | [BT] | Find PID of running program |
| [x] | pkill | [BT] | Kill processes by name |
| [ ] | pmap | [B] | Report process memory map |
| [x] | ps | [BT] | Report process status |
| [x] | pwdx | [BT] | Report process working directory |
| [x] | renice | [BT] | Alter process priority |
| [x] | setsid | [BT] | Run program in new session |
| [ ] | start-stop-daemon | [B] | Start/stop system daemons |
| [x] | taskset | [BT] | Set/retrieve CPU affinity |
| [x] | top | [BT] | Display processes |
| [x] | watch | [BT] | Execute program periodically |

### Networking Utilities
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [x] | arp | [B] | Manipulate ARP cache |
| [x] | arping | [B] | Send ARP requests |
| [x] | brctl | [B] | Ethernet bridge admin |
| [ ] | curl | [T] | Transfer data from URLs (partial) |
| [ ] | dhcprelay | [B] | DHCP relay agent |
| [ ] | dnsd | [B] | Small DNS server |
| [x] | dnsdomainname | [B] | Show DNS domain name |
| [x] | ether-wake | [B] | Send Wake-on-LAN packet |
| [ ] | ftpd | [B] | FTP server |
| [x] | ftpget | [B] | Download via FTP |
| [x] | ftpput | [B] | Upload via FTP |
| [x] | host | [T] | DNS lookup utility |
| [x] | hostname | [BT] | Get/set hostname |
| [ ] | httpd | [B] | HTTP server |
| [x] | ifconfig | [BT] | Configure network interface |
| [x] | ifdown | [B] | Deconfigure network interface |
| [ ] | ifenslave | [B] | Attach/detach slave interface |
| [ ] | ifplugd | [B] | Network interface plug daemon |
| [x] | ifup | [B] | Configure network interface |
| [ ] | inetd | [B] | Internet superserver |
| [x] | ip | [BT] | Show/manipulate routing/devices |
| [x] | ipaddr | [B] | IP address management |
| [x] | ipcalc | [B] | IP network calculator |
| [ ] | ipcrm | [BT] | Remove IPC resources |
| [ ] | ipcs | [BT] | Show IPC resources |
| [x] | iplink | [B] | Network device config |
| [x] | ipneigh | [B] | Neighbor/ARP tables |
| [x] | iproute | [B] | Routing table management |
| [x] | iprule | [B] | Routing policy database |
| [ ] | iptunnel | [B] | IP tunneling |
| [x] | nameif | [B] | Name network interfaces |
| [x] | nc | [BT] | Netcat |
| [x] | netcat | [BT] | Netcat alias |
| [x] | netstat | [BT] | Network statistics |
| [x] | nslookup | [B] | Query DNS servers |
| [ ] | ntpd | [B] | NTP daemon |
| [x] | ping | [BT] | Send ICMP ECHO_REQUEST |
| [x] | ping6 | [BT] | Send IPv6 ICMP ECHO_REQUEST |
| [ ] | pscan | [B] | Port scanner |
| [x] | route | [B] | Show/manipulate routing table |
| [x] | slattach | [B] | Attach serial line to network |
| [x] | ss | [BT] | Socket statistics |
| [ ] | ssl_client | [B] | SSL client helper |
| [ ] | tc | [B] | Traffic control |
| [ ] | tcpsvd | [B] | TCP service daemon |
| [x] | telnet | [B] | Telnet client |
| [ ] | telnetd | [B] | Telnet server |
| [x] | tftp | [B] | TFTP client |
| [ ] | tftpd | [B] | TFTP server |
| [x] | traceroute | [BT] | Trace packet route |
| [x] | traceroute6 | [BT] | IPv6 traceroute |
| [x] | tunctl | [B] | Create/delete TUN/TAP devices |
| [ ] | udhcpc | [B] | DHCP client |
| [ ] | udhcpc6 | [B] | DHCPv6 client |
| [ ] | udhcpd | [B] | DHCP server |
| [x] | vconfig | [B] | VLAN configuration |
| [x] | wget | [B] | Non-interactive network downloader |
| [ ] | zcip | [B] | Zero-configuration networking |

### Package Management
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [x] | apk | [AB] | Alpine Package Keeper (optional) |

### System Administration
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [ ] | acpid | [B] | ACPI event daemon |
| [ ] | addgroup | [B] | Add group |
| [ ] | adduser | [B] | Add user |
| [ ] | blkdiscard | [BT] | Discard device sectors |
| [x] | blkid | [BT] | Print block device attributes |
| [ ] | blockdev | [BT] | Call block device ioctls |
| [ ] | chpasswd | [B] | Update passwords in batch |
| [ ] | crond | [B] | Cron daemon |
| [ ] | crontab | [BT] | Manage cron jobs |
| [ ] | cryptpw | [B] | Create password hash |
| [ ] | delgroup | [B] | Delete group |
| [ ] | deluser | [B] | Delete user |
| [ ] | devmem | [BT] | Read/write physical memory |
| [x] | dmesg | [BT] | Print kernel ring buffer |
| [ ] | eject | [BT] | Eject removable media |
| [ ] | freeramdisk | [B] | Free ramdisk memory |
| [ ] | fsck | [B] | Filesystem check wrapper |
| [ ] | fsck.minix | [B] | Minix filesystem check |
| [ ] | fsfreeze | [BT] | Freeze/unfreeze filesystem |
| [ ] | fstrim | [BT] | Discard unused blocks |
| [ ] | getopt | [BT] | Parse command options |
| [ ] | getty | [BT] | Open terminal and set modes |
| [x] | halt | [BT] | Halt the system |
| [ ] | hdparm | [B] | Get/set hard disk parameters |
| [ ] | hwclock | [BT] | Query/set hardware clock |
| [ ] | i2cdetect | [BT] | Detect I2C chips |
| [ ] | i2cdump | [BT] | Dump I2C registers |
| [ ] | i2cget | [BT] | Read I2C registers |
| [ ] | i2cset | [BT] | Set I2C registers |
| [ ] | i2ctransfer | [BT] | Send I2C messages |
| [ ] | ifconfig | [BT] | Configure network interface |
| [ ] | init | [B] | System init |
| [x] | insmod | [BT] | Insert kernel module |
| [ ] | klogd | [B] | Kernel log daemon |
| [ ] | last | [B] | Show listing of last users |
| [ ] | linuxrc | [B] | Init for initrd |
| [ ] | loadfont | [B] | Load console font |
| [ ] | loadkmap | [B] | Load keyboard map |
| [x] | logger | [BT] | Write to syslog |
| [ ] | login | [B] | Begin session |
| [ ] | logread | [B] | Read syslog ring buffer |
| [x] | losetup | [BT] | Set up loop devices |
| [x] | lsmod | [BT] | Show loaded kernel modules |
| [ ] | lspci | [BT] | List PCI devices |
| [ ] | lsusb | [BT] | List USB devices |
| [ ] | makedevs | [B] | Create device files |
| [ ] | mdev | [B] | Mini udev |
| [x] | mesg | [BT] | Control write access to terminal |
| [ ] | mkdosfs | [B] | Create FAT filesystem |
| [ ] | mke2fs | [B] | Create ext2/3/4 filesystem |
| [ ] | mkfs.ext2 | [B] | Create ext2 filesystem |
| [ ] | mkfs.minix | [B] | Create minix filesystem |
| [ ] | mkfs.vfat | [B] | Create FAT filesystem |
| [ ] | mkpasswd | [B] | Create password hash |
| [ ] | mkswap | [BT] | Set up swap area |
| [ ] | modinfo | [BT] | Show kernel module info |
| [x] | modprobe | [BT] | Add/remove kernel modules |
| [x] | mount | [BT] | Mount filesystem |
| [x] | mountpoint | [BT] | Check if directory is mountpoint |
| [ ] | nbd-client | [B] | Connect to NBD server |
| [ ] | nsenter | [BT] | Run program in namespace |
| [ ] | partprobe | [BT] | Inform OS of partition changes |
| [ ] | passwd | [B] | Change password |
| [x] | pivot_root | [BT] | Change root filesystem |
| [x] | poweroff | [BT] | Power off the system |
| [ ] | rdate | [B] | Get date from remote |
| [x] | readahead | [B] | Read files into page cache |
| [x] | reboot | [BT] | Reboot the system |
| [x] | rfkill | [BT] | Enable/disable wireless devices |
| [x] | rmmod | [BT] | Remove kernel module |
| [ ] | run-init | [B] | Switch root and exec init |
| [ ] | runlevel | [B] | Find current runlevel |
| [ ] | setconsole | [B] | Redirect console |
| [ ] | setfont | [B] | Load console font |
| [ ] | setkeycodes | [B] | Load scancode-keycode mappings |
| [ ] | setlogcons | [B] | Redirect kernel messages |
| [ ] | setpriv | [BT] | Run with different privileges |
| [ ] | setserial | [B] | Get/set serial port info |
| [x] | setsid | [BT] | Run program in new session |
| [ ] | showkey | [B] | Show keyboard scancodes |
| [x] | slattach | [B] | Attach serial line |
| [ ] | su | [B] | Run shell as another user |
| [ ] | sulogin | [B] | Single-user login |
| [x] | swapoff | [BT] | Disable swap |
| [x] | swapon | [BT] | Enable swap |
| [ ] | switch_root | [BT] | Switch root filesystem |
| [x] | sysctl | [BT] | Configure kernel parameters |
| [ ] | syslogd | [B] | System logging daemon |
| [ ] | tune2fs | [B] | Adjust ext2/3/4 parameters |
| [ ] | uevent | [B] | Handle uevents |
| [x] | umount | [BT] | Unmount filesystems |
| [ ] | unshare | [BT] | Run program in new namespace |
| [ ] | vlock | [B] | Virtual console lock |
| [ ] | wall | [B] | Write to all users |
| [ ] | watchdog | [B] | Watchdog daemon |

### Shell / Scripting
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [x] | ash | [B] | Almquist shell |
| [ ] | bash | [T] | Bourne-again shell (partial) |
| [ ] | cttyhack | [B] | Give shell a controlling tty |
| [ ] | hush | [B] | Hush shell |
| [x] | sh | [BT] | POSIX shell |
| [x] | dash | [BT] | Debian Almquist shell |
| [ ] | script | [BT] | Record terminal session |
| [ ] | scriptreplay | [BT] | Replay terminal session |

### Init / Service Management
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [ ] | chpst | [B] | Change process state |
| [ ] | envdir | [B] | Run with environment from dir |
| [ ] | envuidgid | [B] | Run with uid/gid from user |
| [x] | getty | [BT] | Open terminal and set modes |
| [x] | init | [B] | System init process |
| [x] | linuxrc | [B] | Init for initrd |
| [ ] | runsv | [B] | Run a service |
| [x] | runlevel | [B] | Find current runlevel |
| [ ] | runsvdir | [B] | Run a directory of services |
| [ ] | setuidgid | [B] | Run with specified uid/gid |
| [ ] | softlimit | [B] | Run with resource limits |
| [x] | sulogin | [B] | Single-user login |
| [ ] | sv | [B] | Control services |
| [ ] | svc | [B] | Control daemontools services |
| [ ] | svlogd | [B] | Service logging daemon |
| [ ] | svok | [B] | Check if service is running |
| [x] | telinit | [B] | Change runlevel |

### SELinux Utilities
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [ ] | chcon | [BT] | Change file security context |
| [ ] | getenforce | [BT] | Get SELinux enforcing mode |
| [ ] | getsebool | [B] | Get SELinux boolean value |
| [ ] | load_policy | [BT] | Load SELinux policy |
| [ ] | matchpathcon | [B] | Get default context for path |
| [ ] | restorecon | [BT] | Restore file security contexts |
| [ ] | runcon | [BT] | Run command in context |
| [ ] | selinuxenabled | [BT] | Check if SELinux is enabled |
| [ ] | setenforce | [BT] | Set SELinux enforcing mode |
| [ ] | setfiles | [B] | Set file security contexts |
| [ ] | setsebool | [B] | Set SELinux boolean value |

### Console Utilities
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [x] | chvt | [BT] | Change virtual terminal |
| [x] | clear | [BT] | Clear screen |
| [ ] | deallocvt | [BT] | Deallocate virtual terminal |
| [ ] | dumpkmap | [B] | Dump keyboard map |
| [x] | fgconsole | [BT] | Print foreground VT number |
| [ ] | fbset | [B] | Show/change framebuffer settings |
| [ ] | fbsplash | [B] | Framebuffer splash |
| [ ] | kbd_mode | [B] | Report/set keyboard mode |
| [ ] | loadfont | [B] | Load console font |
| [ ] | loadkmap | [B] | Load keyboard translation table |
| [ ] | openvt | [BT] | Start program on new VT |
| [x] | reset | [BT] | Reset terminal |
| [ ] | resize | [B] | Set terminal size |
| [ ] | setconsole | [B] | Redirect system console |
| [ ] | setfont | [B] | Load console font |
| [ ] | setkeycodes | [B] | Load scancode-keycode mapping |
| [ ] | setlogcons | [B] | Send kernel messages to console |
| [ ] | showkey | [B] | Show keyboard scancodes |

### Math / Calculator
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [ ] | bc | [B] | Arbitrary precision calculator |
| [ ] | dc | [B] | RPN calculator |
| [ ] | factor | [BT] | Factor integers |

### Toybox-Specific Utilities
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [ ] | acpi | [T] | Show ACPI status |
| [x] | base32 | [T] | Encode/decode base32 |
| [x] | base64 | [BT] | Encode/decode base64 |
| [ ] | count | [T] | Copy stdin to stdout with count |
| [ ] | demo_many_options | [T] | Test applet |
| [ ] | demo_number | [T] | Test applet |
| [ ] | demo_scankey | [T] | Test applet |
| [ ] | demo_utf8towc | [T] | Test applet |
| [ ] | devmem | [BT] | Read/write physical memory |
| [ ] | elf | [T] | Print ELF file info |
| [x] | file | [BT] | Determine file type |
| [ ] | fstype | [T] | Print filesystem type |
| [x] | getconf | [T] | Get configuration values |
| [ ] | getfattr | [T] | Get extended attributes |
| [ ] | help | [BT] | Show help text |
| [ ] | hexedit | [T] | Hex editor |
| [ ] | iconv | [T] | Convert character encoding |
| [ ] | inotifyd | [T] | Inotify daemon |
| [x] | ionice | [BT] | Set I/O scheduling class |
| [ ] | lsattr | [BT] | List file attributes |
| [ ] | makedevs | [BT] | Create device files |
| [ ] | mcookie | [T] | Generate magic cookie |
| [ ] | microcom | [T] | Serial terminal |
| [ ] | mix | [T] | Audio mixer |
| [ ] | mkpasswd | [BT] | Generate password hash |
| [ ] | modinfo | [BT] | Show kernel module info |
| [x] | mountpoint | [BT] | Check if directory is mountpoint |
| [ ] | nbd-client | [T] | Connect to NBD server |
| [x] | nproc | [BT] | Print number of processors |
| [ ] | oneit | [T] | Simple init |
| [ ] | openvt | [BT] | Open new virtual terminal |
| [ ] | partprobe | [BT] | Inform kernel of partition changes |
| [x] | pgrep | [BT] | Find processes by name |
| [x] | pkill | [BT] | Kill processes by name |
| [ ] | pmap | [T] | Report process memory map |
| [x] | printenv | [BT] | Print environment variables |
| [x] | pwdx | [BT] | Print process working directory |
| [x] | readahead | [T] | Preload files into cache |
| [ ] | readelf | [T] | Display ELF file info |
| [x] | rev | [BT] | Reverse lines |
| [ ] | rtcwake | [T] | Enter system sleep state |
| [ ] | sendevent | [T] | Send input events |
| [ ] | setfattr | [T] | Set extended attributes |
| [x] | setsid | [BT] | Run in new session |
| [x] | sha3sum | [T] | SHA3 checksums |
| [x] | shred | [BT] | Securely delete files |
| [ ] | skeleton | [T] | Template applet |
| [ ] | skeleton_alias | [T] | Template applet |
| [ ] | sntp | [T] | Simple NTP client |
| [x] | strings | [BT] | Print printable strings |
| [ ] | strace | [T] | Trace system calls |
| [x] | sysctl | [BT] | Read/write kernel parameters |
| [x] | tac | [BT] | Concatenate in reverse |
| [x] | taskset | [BT] | Get/set CPU affinity |
| [x] | time | [BT] | Time command execution |
| [ ] | toysh | [T] | Toybox shell |
| [ ] | ts | [T] | Timestamp stdin |
| [ ] | ulimit | [BT] | Get/set resource limits |
| [ ] | unicode | [T] | Print unicode characters |
| [x] | unix2dos | [BT] | Convert line endings |
| [ ] | uuidgen | [T] | Generate UUID |
| [x] | vconfig | [T] | VLAN configuration |
| [ ] | vmstat | [BT] | Report virtual memory stats |
| [x] | w | [BT] | Show logged in users |
| [x] | watch | [BT] | Execute program periodically |
| [ ] | watchdog | [T] | Watchdog timer daemon |
| [x] | xxd | [BT] | Hexdump utility |

### Miscellaneous Utilities
| Status | Applet | Source | Description |
|--------|--------|--------|-------------|
| [ ] | adjtimex | [B] | Tune kernel clock |
| [ ] | ascii | [B] | Print ASCII table |
| [ ] | bbconfig | [B] | Print busybox config |
| [ ] | beep | [B] | Beep through PC speaker |
| [ ] | chat | [B] | Modem chat script |
| [ ] | conspy | [B] | See/control virtual consoles |
| [ ] | crond | [B] | Cron daemon |
| [ ] | crontab | [BT] | Manage cron tables |
| [ ] | devfsd | [B] | Devfs management daemon |
| [ ] | dumpleases | [B] | Show DHCP leases |
| [ ] | fbset | [B] | Framebuffer settings |
| [ ] | fdflush | [B] | Force floppy disk sync |
| [x] | flock | [BT] | File locking |
| [x] | fsync | [BT] | Synchronize file to disk |
| [ ] | getopt | [BT] | Parse command options |
| [ ] | hdparm | [B] | Get/set hard disk parameters |
| [x] | hexdump | [BT] | Display file in hex |
| [x] | hd | [BT] | Hexdump alias |
| [ ] | i2cdetect | [BT] | Detect I2C chips |
| [ ] | i2cdump | [BT] | Dump I2C registers |
| [ ] | i2cget | [BT] | Read I2C registers |
| [ ] | i2cset | [BT] | Set I2C registers |
| [ ] | i2ctransfer | [BT] | I2C message transfer |
| [ ] | inotifyd | [BT] | Inotify daemon |
| [ ] | ipcrm | [BT] | Remove IPC resources |
| [ ] | ipcs | [BT] | Show IPC resources |
| [ ] | kbd_mode | [B] | Report/set keyboard mode |
| [ ] | last | [B] | Show last logins |
| [ ] | length | [B] | Print string length |
| [ ] | linux32 | [B] | Uname emulation |
| [ ] | linux64 | [B] | Uname emulation |
| [ ] | lsscsi | [T] | List SCSI devices |
| [ ] | makedevs | [BT] | Create device nodes |
| [ ] | man | [B] | Manual page viewer |
| [ ] | microcom | [BT] | Minicom-like serial terminal |
| [ ] | mim | [B] | Mime helper |
| [x] | mountpoint | [BT] | Check mountpoint |
| [ ] | mt | [B] | Magnetic tape control |
| [ ] | nandwrite | [B] | Write to NAND |
| [ ] | nanddump | [B] | Dump NAND contents |
| [ ] | nbd-client | [BT] | NBD client |
| [ ] | prlimit | [T] | Get/set process resource limits |
| [ ] | raidautorun | [B] | Start RAID autorun |
| [ ] | readprofile | [B] | Read kernel profiling data |
| [ ] | realpath | [BT] | Print absolute path |
| [ ] | reformime | [B] | Parse MIME messages |
| [ ] | setarch | [B] | Set architecture |
| [ ] | setfattr | [BT] | Set file attributes |
| [ ] | setkeycodes | [B] | Load keyboard keycodes |
| [x] | setsid | [BT] | Run in new session |
| [ ] | setuidgid | [B] | Change uid/gid and exec |
| [x] | sha3sum | [T] | SHA3 checksums |
| [x] | strings | [BT] | Find printable strings |
| [ ] | taskset | [BT] | Set CPU affinity |
| [x] | time | [BT] | Time a command |
| [ ] | timeout | [BT] | Run with time limit |
| [ ] | ts | [T] | Timestamp lines |
| [ ] | ttysize | [B] | Print terminal dimensions |
| [ ] | ubi* | [B] | UBI utilities (ubiattach, ubimkvol, etc.) |
| [ ] | ubiattach | [B] | Attach MTD device to UBI |
| [ ] | ubidetach | [B] | Detach UBI device |
| [ ] | ubimkvol | [B] | Create UBI volume |
| [ ] | ubirename | [B] | Rename UBI volumes |
| [ ] | ubirmvol | [B] | Remove UBI volume |
| [ ] | ubirsvol | [B] | Resize UBI volume |
| [ ] | ubiupdatevol | [B] | Update UBI volume |
| [ ] | udhcpc | [B] | DHCP client |
| [ ] | udhcpc6 | [B] | DHCPv6 client |
| [ ] | udhcpd | [B] | DHCP server |
| [ ] | volname | [B] | Print volume name |

---

## Testing Requirements

### Unit Tests
- [~] Unit tests for each applet's core functionality
- [ ] Edge case testing for all options/flags
- [ ] Error handling verification

### Integration Tests
- [~] End-to-end testing against reference implementations
- [ ] Compatibility tests with real-world scripts
- [ ] POSIX compliance verification

### Property-Based Testing
- [x] Property-based tests with proptest (18 tests)
- [x] Roundtrip testing for compression/encoding
- [x] Sort/uniq invariant testing

### Fuzz Testing
- [x] Fuzz testing for input parsing (grep, sed, awk patterns)
- [x] Fuzz testing for file handling (tar, compression)
- [x] Memory safety verification via fuzzing

### Benchmarking
- [x] Criterion benchmarks for core applets
- [x] Text processing benchmarks
- [x] Compression benchmarks

---

## Performance Goals
- [x] Binary size within 2x of C implementations (311 KB vs ~2.4 MB for BusyBox - actually 8x smaller!)
- [x] Runtime performance within 10% of C implementations (6 applets faster, 8+ at parity)
- [x] Memory usage comparable to original utilities

### Performance Optimizations Implemented
- [x] LTO and size optimization in release builds
- [x] Large I/O buffers (64-128KB)
- [x] Fast literal matching with memchr/memmem for grep
- [x] Regex pattern caching
- [x] SIMD byte searching via memchr

---

## Documentation
- [ ] Man pages for all applets
- [x] Usage examples (docs website)
- [x] Build instructions for various targets (docs website)
- [x] Comparison guide with BusyBox/Toybox (docs website)
- [x] Documentation website with Angular/TailwindCSS

---

## Platform Support
- [x] Linux x86_64
- [x] Linux i686
- [x] Linux ARM (32-bit)
- [x] Linux ARM64 (aarch64)
- [ ] Linux MIPS
- [ ] Linux RISC-V
- [x] Android (native Bionic support)
- [x] musl libc support
- [x] glibc support

---

## Milestones

### Phase 1: Core Infrastructure ✅
- [x] Multi-call binary framework
- [x] Basic coreutils (cat, ls, cp, mv, rm, mkdir, echo, etc.) - via uutils
- [x] Shell (ash-compatible) - sh, ash, dash

### Phase 2: System Utilities ✅
- [x] Mount/umount
- [x] Init system - init, telinit, runlevel, getty, sulogin
- [x] Process utilities (ps, kill, top, pgrep, pkill)
- [x] Network basics (ifconfig, ping, netstat)

### Phase 3: Advanced Utilities ✅
- [x] Archive utilities (tar, gzip, bzip2, xz, cpio)
- [x] Text processing (sed, awk, grep, find)
- [x] Full networking stack - ip, route, arp, nslookup, dig, ss, telnet, arping

### Phase 4: Complete Coverage
- [~] All remaining applets (189/300+ implemented, 95% of Toybox)
- [~] 100% compatibility testing
- [x] Performance optimization (6 applets faster than BusyBox!)
- [x] Benchmarking infrastructure
- [x] Fuzzing infrastructure
- [x] POSIX compliance for core utilities

### Phase 5: Distribution & Ecosystem
- [x] ArmyLinux distribution subproject (distro/)
- [x] Alpine-compatible root filesystem
- [x] Docker images (scratch, Alpine-based)
- [ ] Bootable ISO image
- [ ] QEMU testing infrastructure
- [ ] ARM64/ARM32 distribution images
- [ ] Automated CI/CD builds
- [ ] Package repository hosting

---

## ArmyLinux Distribution

An Alpine-compatible Linux distribution powered by armybox.

### Build System
- [x] Makefile with targets
- [x] Root filesystem builder (scripts/build-rootfs.sh)
- [x] Docker image builder (scripts/build-docker.sh)
- [x] ISO builder (scripts/build-iso.sh)
- [x] QEMU runner (scripts/run-qemu.sh)

### Configuration Files
- [x] /etc/inittab
- [x] /etc/fstab
- [x] /etc/profile
- [x] /etc/passwd, group, shadow
- [x] /etc/apk/repositories
- [x] /etc/sysctl.conf

### Init Scripts
- [x] /etc/init.d/rcS (startup)
- [x] /etc/init.d/rcK (shutdown)
- [ ] /etc/init.d/S* service scripts
- [ ] /etc/init.d/K* service scripts

### Features
- [ ] OpenRC compatibility layer
- [x] apk package manager (optional feature)
- [ ] Live USB support
- [ ] Installer script
- [ ] ARM64 support
- [ ] Raspberry Pi support

---

## Notes
- Prioritize memory safety over raw performance
- Use `unsafe` only when absolutely necessary and document reasoning
- Maintain POSIX compliance where applicable
- Support both BusyBox and Toybox command-line argument styles where they differ
