#!/bin/bash
#
# Build ArmyLinux root filesystem
#
# This script creates a minimal Alpine-compatible root filesystem
# with armybox replacing BusyBox.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISTRO_DIR="$(dirname "$SCRIPT_DIR")"
ARMYBOX_DIR="$(dirname "$DISTRO_DIR")"
ROOTFS_DIR="$DISTRO_DIR/rootfs"
CONFIG_DIR="$DISTRO_DIR/config"

ALPINE_VERSION="${ALPINE_VERSION:-3.19}"
ALPINE_MIRROR="${ALPINE_MIRROR:-https://dl-cdn.alpinelinux.org/alpine}"
ARCH="${ARCH:-x86_64}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}==>${NC} $*"; }
warn() { echo -e "${YELLOW}WARNING:${NC} $*"; }
error() { echo -e "${RED}ERROR:${NC} $*" >&2; exit 1; }

# Check prerequisites
check_prereqs() {
    log "Checking prerequisites..."

    # Check for root (needed for chroot operations)
    if [[ $EUID -ne 0 ]]; then
        warn "Not running as root. Some operations may fail."
        warn "Consider running with: sudo $0"
    fi

    # Check for armybox binary
    ARMYBOX_BIN="$ARMYBOX_DIR/target/x86_64-unknown-linux-musl/release/armybox"
    if [[ ! -f "$ARMYBOX_BIN" ]]; then
        error "armybox binary not found at $ARMYBOX_BIN"
        error "Run 'make static' in $ARMYBOX_DIR first"
    fi

    # Verify it's statically linked
    if ! file "$ARMYBOX_BIN" | grep -q "statically linked"; then
        warn "armybox may not be statically linked"
    fi

    log "Prerequisites OK"
}

# Create directory structure
create_dirs() {
    log "Creating directory structure..."

    mkdir -p "$ROOTFS_DIR"/{bin,sbin,usr/{bin,sbin,lib,share},lib,etc/{init.d,apk},dev,proc,sys,run,tmp,var/{log,cache,lib/apk},root,home,mnt,opt,srv}

    # Set permissions
    chmod 1777 "$ROOTFS_DIR/tmp"
    chmod 0700 "$ROOTFS_DIR/root"

    log "Directory structure created"
}

# Install armybox
install_armybox() {
    log "Installing armybox..."

    # Copy binary
    cp "$ARMYBOX_BIN" "$ROOTFS_DIR/bin/armybox"
    chmod 755 "$ROOTFS_DIR/bin/armybox"

    # Create symlinks
    log "Creating applet symlinks..."

    # Get list of applets
    APPLETS=$("$ARMYBOX_BIN" --list 2>/dev/null || true)

    if [[ -z "$APPLETS" ]]; then
        error "Failed to get applet list from armybox"
    fi

    for applet in $APPLETS; do
        # Determine target directory based on applet type
        case "$applet" in
            init|halt|reboot|poweroff|shutdown|sulogin|getty|telinit|runlevel|\
            ifconfig|route|arp|mount|umount|swapon|swapoff|mdev|modprobe|insmod|rmmod|\
            sysctl|hwclock|fdisk|mkfs*|fsck*|blkid)
                target="$ROOTFS_DIR/sbin/$applet"
                ;;
            *)
                target="$ROOTFS_DIR/bin/$applet"
                ;;
        esac

        # Don't overwrite armybox itself
        if [[ "$applet" != "armybox" ]]; then
            ln -sf armybox "$target" 2>/dev/null || true
        fi
    done

    # Ensure /bin/sh exists
    ln -sf armybox "$ROOTFS_DIR/bin/sh"

    log "armybox installed with $(echo "$APPLETS" | wc -w) applets"
}

# Install configuration files
install_config() {
    log "Installing configuration files..."

    # Core config files
    cp "$CONFIG_DIR/inittab" "$ROOTFS_DIR/etc/"
    cp "$CONFIG_DIR/fstab" "$ROOTFS_DIR/etc/"
    cp "$CONFIG_DIR/profile" "$ROOTFS_DIR/etc/"
    cp "$CONFIG_DIR/passwd" "$ROOTFS_DIR/etc/"
    cp "$CONFIG_DIR/group" "$ROOTFS_DIR/etc/"
    cp "$CONFIG_DIR/shadow" "$ROOTFS_DIR/etc/"
    cp "$CONFIG_DIR/sysctl.conf" "$ROOTFS_DIR/etc/"

    # Set proper permissions on shadow
    chmod 0640 "$ROOTFS_DIR/etc/shadow"

    # APK repositories
    mkdir -p "$ROOTFS_DIR/etc/apk"
    cp "$CONFIG_DIR/repositories" "$ROOTFS_DIR/etc/apk/"

    # Create hostname
    echo "armylinux" > "$ROOTFS_DIR/etc/hostname"

    # Create hosts file
    cat > "$ROOTFS_DIR/etc/hosts" << 'EOF'
127.0.0.1       localhost localhost.localdomain
::1             localhost localhost.localdomain
127.0.1.1       armylinux armylinux.localdomain
EOF

    # Create resolv.conf
    cat > "$ROOTFS_DIR/etc/resolv.conf" << 'EOF'
nameserver 8.8.8.8
nameserver 8.8.4.4
nameserver 1.1.1.1
EOF

    # Create nsswitch.conf
    cat > "$ROOTFS_DIR/etc/nsswitch.conf" << 'EOF'
passwd:     files
group:      files
shadow:     files
hosts:      files dns
networks:   files
protocols:  files
services:   files
ethers:     files
rpc:        files
EOF

    # Create os-release
    cat > "$ROOTFS_DIR/etc/os-release" << EOF
NAME="ArmyLinux"
ID=armylinux
VERSION_ID=0.1.0
PRETTY_NAME="ArmyLinux 0.1.0"
HOME_URL="https://github.com/PegasusHeavyIndustries/armybox"
BUG_REPORT_URL="https://github.com/PegasusHeavyIndustries/armybox/issues"
EOF

    # Create motd
    cat > "$ROOTFS_DIR/etc/motd" << 'EOF'

Welcome to ArmyLinux!

Powered by armybox - a memory-safe BusyBox replacement written in Rust.

 * Documentation: https://github.com/PegasusHeavyIndustries/armybox
 * Package repos: Alpine Linux v3.19

EOF

    log "Configuration files installed"
}

# Create init scripts
create_init_scripts() {
    log "Creating init scripts..."

    # rcS - main init script
    cat > "$ROOTFS_DIR/etc/init.d/rcS" << 'EOF'
#!/bin/sh
#
# rcS - ArmyLinux system initialization script

echo "Starting ArmyLinux..."

# Mount virtual filesystems (if not already mounted by init)
mountpoint -q /proc || mount -t proc proc /proc
mountpoint -q /sys || mount -t sysfs sysfs /sys
mountpoint -q /dev || mount -t devtmpfs devtmpfs /dev

# Create device nodes
mkdir -p /dev/pts /dev/shm
mount -t devpts devpts /dev/pts 2>/dev/null
mount -t tmpfs tmpfs /dev/shm 2>/dev/null

# Mount other filesystems from fstab
mount -a 2>/dev/null

# Set hostname
if [ -f /etc/hostname ]; then
    hostname -F /etc/hostname
fi

# Apply sysctl settings
if [ -f /etc/sysctl.conf ]; then
    sysctl -p /etc/sysctl.conf >/dev/null 2>&1
fi

# Seed random number generator
if [ -f /var/lib/random-seed ]; then
    cat /var/lib/random-seed >/dev/urandom 2>/dev/null
fi
dd if=/dev/urandom of=/var/lib/random-seed count=1 bs=512 2>/dev/null

# Create /var/run symlink if needed
[ -L /var/run ] || ln -sf /run /var/run

# Start syslog (if available)
if command -v syslogd >/dev/null 2>&1; then
    syslogd -n &
fi

# Start klogd (if available)
if command -v klogd >/dev/null 2>&1; then
    klogd -n &
fi

# Run local startup scripts
for script in /etc/init.d/S*; do
    [ -x "$script" ] && "$script" start
done

echo "ArmyLinux started successfully"
EOF
    chmod 755 "$ROOTFS_DIR/etc/init.d/rcS"

    # rcK - shutdown script
    cat > "$ROOTFS_DIR/etc/init.d/rcK" << 'EOF'
#!/bin/sh
#
# rcK - ArmyLinux shutdown script

echo "Shutting down ArmyLinux..."

# Run local shutdown scripts
for script in /etc/init.d/K*; do
    [ -x "$script" ] && "$script" stop
done

# Save random seed
dd if=/dev/urandom of=/var/lib/random-seed count=1 bs=512 2>/dev/null

# Kill all processes
killall5 -15
sleep 1
killall5 -9

echo "ArmyLinux shutdown complete"
EOF
    chmod 755 "$ROOTFS_DIR/etc/init.d/rcK"

    log "Init scripts created"
}

# Create device nodes
create_devices() {
    log "Creating device nodes..."

    # Only create if running as root
    if [[ $EUID -eq 0 ]]; then
        mknod -m 666 "$ROOTFS_DIR/dev/null" c 1 3 2>/dev/null || true
        mknod -m 666 "$ROOTFS_DIR/dev/zero" c 1 5 2>/dev/null || true
        mknod -m 666 "$ROOTFS_DIR/dev/random" c 1 8 2>/dev/null || true
        mknod -m 666 "$ROOTFS_DIR/dev/urandom" c 1 9 2>/dev/null || true
        mknod -m 666 "$ROOTFS_DIR/dev/tty" c 5 0 2>/dev/null || true
        mknod -m 620 "$ROOTFS_DIR/dev/console" c 5 1 2>/dev/null || true
        mknod -m 666 "$ROOTFS_DIR/dev/ptmx" c 5 2 2>/dev/null || true

        for i in 0 1 2 3 4 5 6; do
            mknod -m 620 "$ROOTFS_DIR/dev/tty$i" c 4 $i 2>/dev/null || true
        done

        log "Device nodes created"
    else
        warn "Skipping device node creation (not running as root)"
    fi
}

# Print summary
print_summary() {
    log "Build complete!"
    echo ""
    echo "Root filesystem: $ROOTFS_DIR"
    echo "Size: $(du -sh "$ROOTFS_DIR" | cut -f1)"
    echo ""
    echo "To test with chroot:"
    echo "  sudo chroot $ROOTFS_DIR /bin/sh"
    echo ""
    echo "To create Docker image:"
    echo "  ./scripts/build-docker.sh"
}

# Main
main() {
    log "Building ArmyLinux root filesystem..."
    log "Alpine version: $ALPINE_VERSION"
    log "Architecture: $ARCH"

    check_prereqs
    create_dirs
    install_armybox
    install_config
    create_init_scripts
    create_devices
    print_summary
}

main "$@"
