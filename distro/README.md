# ArmyLinux

An Alpine-compatible Linux distribution powered by **armybox** instead of BusyBox.

## Overview

ArmyLinux is a minimal, security-focused Linux distribution that maintains compatibility with Alpine Linux's package ecosystem while replacing BusyBox with armybox for:

- **Memory safety** - Rust-based userspace eliminates buffer overflows
- **Modern tooling** - Native Rust implementations with optimized performance
- **Full compatibility** - Drop-in replacement for Alpine's busybox package

## Features

- 🦀 **Rust-powered userspace** - armybox provides 303 utilities
- 📦 **Alpine APK compatible** - Use Alpine's package repositories
- 🔒 **musl libc** - Static binaries, small footprint
- 🐳 **Container-ready** - Minimal base images (~8MB)
- ⚡ **Fast boot** - armybox init system with inittab support

## Quick Start

### Build the Distribution

```bash
# Build root filesystem
./scripts/build-rootfs.sh

# Create bootable ISO
./scripts/build-iso.sh

# Build Docker image
./scripts/build-docker.sh
```

### Run in Docker

```bash
docker build -t armylinux .
docker run -it armylinux
```

### Run in QEMU

```bash
./scripts/run-qemu.sh
```

## Directory Structure

```
distro/
├── README.md           # This file
├── Dockerfile          # Docker image definition
├── Makefile            # Build automation
├── config/
│   ├── inittab         # Init configuration
│   ├── fstab           # Filesystem table
│   ├── profile         # Shell profile
│   ├── passwd          # User database
│   ├── group           # Group database
│   ├── shadow          # Password hashes
│   └── repositories    # APK repositories
├── rootfs/
│   └── ...             # Generated root filesystem
├── scripts/
│   ├── build-rootfs.sh # Build root filesystem
│   ├── build-iso.sh    # Create bootable ISO
│   ├── build-docker.sh # Build Docker image
│   ├── run-qemu.sh     # Run in QEMU
│   └── chroot-setup.sh # Setup chroot environment
└── iso/
    └── ...             # Generated ISO files
```

## Compatibility

### Alpine Package Compatibility

ArmyLinux uses Alpine's APK package manager and can install packages from Alpine repositories:

```bash
apk add --no-cache python3 nodejs nginx
```

### BusyBox Applet Mapping

| BusyBox Applet | ArmyLinux Status |
|----------------|------------------|
| Core utilities | ✅ Full support |
| Shell (ash)    | ✅ Compatible |
| Init system    | ✅ Compatible |
| Networking     | ✅ Partial |
| Editors        | ✅ less, more, awk |

### Differences from Alpine

1. **Userspace**: armybox instead of BusyBox
2. **Binary size**: ~6MB vs ~1MB (trade-off for memory safety)
3. **Init**: armybox init (compatible with inittab)

## Building from Source

### Prerequisites

- Rust 1.70+
- Docker (for container builds)
- QEMU (for VM testing)
- Alpine Linux base (for APK tools)

### Build Steps

```bash
# 1. Build armybox static binary
cd ..
make static

# 2. Build root filesystem
cd distro
make rootfs

# 3. Build ISO (optional)
make iso

# 4. Build Docker image
make docker
```

## Configuration

### /etc/inittab

```
::sysinit:/etc/init.d/rcS
::respawn:/sbin/getty 38400 tty1
::respawn:/sbin/getty 38400 tty2
::ctrlaltdel:/sbin/reboot
::shutdown:/bin/umount -a -r
::shutdown:/sbin/swapoff -a
```

### /etc/repositories

```
https://dl-cdn.alpinelinux.org/alpine/v3.19/main
https://dl-cdn.alpinelinux.org/alpine/v3.19/community
```

## Use Cases

### Container Base Image

```dockerfile
FROM armylinux:latest
RUN apk add --no-cache python3
COPY app.py /app/
CMD ["python3", "/app/app.py"]
```

### Embedded Systems

- IoT devices
- Network appliances
- Minimal VMs

### Security-Critical Deployments

- Memory-safe userspace
- Reduced attack surface
- Static binaries

## Roadmap

- [ ] Automated CI/CD builds
- [ ] ARM64/ARM32 images
- [ ] OpenRC compatibility layer
- [ ] apk-tools integration testing
- [ ] Live USB support
- [ ] Installer script

## License

MIT OR Apache-2.0 (same as armybox)

## Related Projects

- [armybox](../) - The BusyBox/Toybox replacement
- [Alpine Linux](https://alpinelinux.org/) - The base distribution
- [musl libc](https://musl.libc.org/) - The C library
