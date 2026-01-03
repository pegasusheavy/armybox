# Armybox

A `#[no_std]` BusyBox/Toybox clone written in Rust.

## Features

- **131 applets** - core utilities, text processing, system, file ops, process management, vi editor, and POSIX shell
- **Multi-call binary** - single executable providing all utilities
- **Pure Rust 2024** - memory-safe implementation using the latest Rust edition
- **Incredibly tiny** - **173 KB** stripped, **~85 KB** with UPX compression
- **True `#[no_std]`** - no standard library dependency, only `libc` and `alloc`
- **Android-native** - first-class Android/Bionic support, works on Android 5.0+
- **Embedded-ready** - works on systems without full std support
- **Cross-platform** - builds for Linux (glibc/musl), Android, x86_64, ARM64, ARM32

## Binary Size Comparison

| Binary | Size | UPX Size | Applets | Size/Applet |
|--------|------|----------|---------|-------------|
| **Armybox** | 173 KB | ~85 KB | 131 | ~1.3 KB |
| Toybox | ~500 KB | ~200 KB | 200+ | ~2.5 KB |
| BusyBox | 2.4 MB | ~1 MB | 274 | ~9 KB |

Armybox is **9x more efficient per applet** than BusyBox and **2.5x more efficient** than Toybox!

## Current Applets (126)

### File Operations (31)
`basename`, `cat`, `cd`, `chgrp`, `chmod`, `chown`, `cp`, `dd`, `dirname`, `file`, `install`, `link`, `ln`, `ls`, `mkdir`, `mkfifo`, `mknod`, `mktemp`, `mv`, `pwd`, `readlink`, `realpath`, `rm`, `rmdir`, `shred`, `split`, `stat`, `sync`, `touch`, `truncate`, `unlink`

### Text Processing & Editors (29)
`awk`, `comm`, `cut`, `dos2unix`, `echo`, `expand`, `fmt`, `fold`, `grep`, `head`, `nl`, `paste`, `printf`, `rev`, `sed`, `seq`, `sort`, `strings`, `tac`, `tail`, `tee`, `tr`, `unexpand`, `uniq`, `unix2dos`, `vi`, `view`, `wc`, `yes`

### System Utilities (38)
`arch`, `chroot`, `chvt`, `date`, `df`, `dmesg`, `du`, `env`, `fgconsole`, `free`, `groups`, `halt`, `hostid`, `hostname`, `id`, `logger`, `logname`, `lsmod`, `mount`, `mountpoint`, `nice`, `nohup`, `nproc`, `poweroff`, `printenv`, `reboot`, `swapoff`, `swapon`, `sysctl`, `timeout`, `tty`, `umount`, `uname`, `uptime`, `users`, `w`, `who`, `whoami`

### Process Management (11)
`kill`, `killall`, `pgrep`, `pidof`, `pkill`, `ps`, `pwdx`, `renice`, `setsid`

### Shell (3)
`sh`, `ash`, `dash`

### Checksums & Encoding (5)
`base64`, `hexdump`, `md5sum`, `od`, `xxd`

### Miscellaneous (14)
`[`, `clear`, `cmp`, `diff`, `expr`, `factor`, `false`, `find`, `getconf`, `mesg`, `sleep`, `test`, `time`, `true`, `usleep`, `which`

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/PegasusHeavyIndustries/armybox
cd armybox

# Build release binary
cargo build --release

# Binary is at target/release/armybox
```

### Install Symlinks

```bash
# Install symbolic links to /usr/local/bin
sudo ./target/release/armybox --install /usr/local/bin
```

### Compress with UPX

```bash
# Compress to ~64KB
upx --best target/release/armybox
```

### Build for Android

Armybox has native Android support with Bionic libc compatibility.

```bash
# Install Android targets
rustup target add aarch64-linux-android    # ARM64 (most modern devices)
rustup target add armv7-linux-androideabi  # ARMv7 (older 32-bit)
rustup target add x86_64-linux-android     # x86_64 (emulator/Chromebook)

# Set up Android NDK (required)
export ANDROID_NDK_HOME=/path/to/android-ndk
export PATH=$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin

# Build for Android ARM64
cargo build --release --target aarch64-linux-android

# Or use the cargo alias
cargo build-android

# Binary is at target/aarch64-linux-android/release/armybox
```

#### Android Deployment

```bash
# Push to Android device via ADB
adb push target/aarch64-linux-android/release/armybox /data/local/tmp/

# Make executable and test
adb shell chmod +x /data/local/tmp/armybox
adb shell /data/local/tmp/armybox --list
```

## Usage

### Direct Invocation
```bash
./armybox ls -la
./armybox cat file.txt
./armybox echo "Hello, World!"
```

### List Available Applets
```bash
./armybox --list
```

### Via Symlinks
After installing symlinks:
```bash
ls -la
cat file.txt
echo "Hello!"
```

## Library Usage

Armybox is a `#[no_std]` library that can be used in embedded environments.

### Cargo.toml

```toml
[dependencies]
armybox = { version = "0.2", default-features = false, features = ["alloc"] }
```

### Features

| Feature | Description | Default |
|---------|-------------|---------|
| `alloc` | Heap allocation (Vec, String) | ✅ |
| `std` | Standard library support | ❌ |

### Example

```rust
#![no_std]
extern crate alloc;

use armybox::{run_applet, applets};

// Check if an applet exists
let exists = armybox::is_applet(b"echo");

// List all applets
for (name, _) in applets::APPLETS {
    // ...
}
```

## Architecture

```
src/
├── lib.rs          # Library entry (no_std compatible)
├── main.rs         # Binary entry (no_std, no_main)
├── io.rs           # Raw I/O via libc
├── sys.rs          # System utilities
└── applets/
    ├── mod.rs      # Applet registry
    ├── file.rs     # File operations
    ├── text.rs     # Text processing
    ├── system.rs   # System utilities
    └── misc.rs     # Miscellaneous
```

## Building

```bash
# Debug build
cargo build

# Release build (optimized for size)
cargo build --release

# Check binary size
ls -lh target/release/armybox
```

## How It Works

Armybox is built entirely with `#[no_std]`:

1. **No Rust Standard Library** - Only uses `core` and `alloc`
2. **Direct libc Calls** - All I/O goes through raw `libc::*` functions
3. **Custom Allocator** - Uses `libc::malloc/free` for heap allocation
4. **Custom Panic Handler** - Minimal panic handling without unwinding
5. **No Main Runtime** - Uses `#[no_main]` with raw C entry point

This results in an incredibly small binary that's perfect for:
- Embedded systems
- Containers (FROM scratch)
- Rescue environments
- Space-constrained systems

## Roadmap

See [TODO.md](TODO.md) for the full feature roadmap.

Key milestones:
1. ✅ Core infrastructure (multi-call binary, applet dispatch)
2. ✅ Basic coreutils (cat, ls, cp, mv, etc.)
3. ✅ Text processing (echo, head, tail, wc, etc.)
4. ✅ System utilities (ps, id, hostname, etc.)
5. ✅ `#[no_std]` compatibility
6. 🔄 Compression utilities (gzip, bzip2)
7. 🔄 Network utilities (ping, wget)
8. 🔄 Shell (ash-compatible)

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please see TODO.md for areas that need work.

## Comparison

| Feature | Armybox | Toybox | BusyBox |
|---------|---------|--------|---------|
| Language | Rust | C | C |
| Memory Safety | ✅ Compile-time | ❌ Manual | ❌ Manual |
| Binary Size | **173 KB** | ~500 KB | 2.4 MB |
| Per-Applet Size | **~1 KB** | ~2.5 KB | ~9 KB |
| `#[no_std]` | ✅ | N/A | N/A |
| Applet Count | 126 | 200+ | 274 |
| License | MIT/Apache-2.0 | 0BSD | GPL v2 |
