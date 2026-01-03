#!/bin/bash
#
# Run ArmyLinux in QEMU
#
# Provides a quick way to test the distribution in a VM.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISTRO_DIR="$(dirname "$SCRIPT_DIR")"
ISO_DIR="$DISTRO_DIR/iso"
ROOTFS_DIR="$DISTRO_DIR/rootfs"

ISO_NAME="${ISO_NAME:-armylinux-0.1.0-x86_64.iso}"
MEMORY="${MEMORY:-512}"
CPUS="${CPUS:-2}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}==>${NC} $*"; }
warn() { echo -e "${YELLOW}WARNING:${NC} $*"; }
error() { echo -e "${RED}ERROR:${NC} $*" >&2; exit 1; }

# Run from ISO
run_iso() {
    local iso="$ISO_DIR/$ISO_NAME"

    if [[ ! -f "$iso" ]]; then
        error "ISO not found: $iso"
        error "Run build-iso.sh first"
    fi

    log "Starting QEMU from ISO..."
    log "Memory: ${MEMORY}MB, CPUs: $CPUS"

    qemu-system-x86_64 \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -cdrom "$iso" \
        -boot d \
        -enable-kvm \
        -nographic \
        -serial mon:stdio \
        -append "console=ttyS0"
}

# Run with direct kernel boot (faster for testing)
run_direct() {
    log "Starting QEMU with direct kernel boot..."

    # Check for kernel
    local kernel="${KERNEL:-/boot/vmlinuz-linux}"
    if [[ ! -f "$kernel" ]]; then
        # Try Alpine kernel path
        kernel="/boot/vmlinuz-lts"
        if [[ ! -f "$kernel" ]]; then
            error "No kernel found. Set KERNEL=/path/to/vmlinuz"
        fi
    fi

    # Create a simple initramfs with just armybox
    local initramfs="/tmp/armylinux-initramfs.cpio.gz"
    if [[ ! -f "$initramfs" ]] || [[ "$ROOTFS_DIR/bin/armybox" -nt "$initramfs" ]]; then
        log "Creating initramfs..."
        local tmpdir=$(mktemp -d)
        mkdir -p "$tmpdir"/{bin,dev,etc,lib,mnt,proc,sys,run,tmp}
        cp "$ROOTFS_DIR/bin/armybox" "$tmpdir/bin/"
        ln -sf armybox "$tmpdir/bin/sh"

        # Create simple init
        cat > "$tmpdir/init" << 'EOF'
#!/bin/sh
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
exec /bin/sh
EOF
        chmod 755 "$tmpdir/init"

        (cd "$tmpdir" && find . | cpio -H newc -o | gzip > "$initramfs")
        rm -rf "$tmpdir"
    fi

    log "Kernel: $kernel"
    log "Memory: ${MEMORY}MB, CPUs: $CPUS"

    qemu-system-x86_64 \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -kernel "$kernel" \
        -initrd "$initramfs" \
        -append "console=ttyS0 rdinit=/init" \
        -enable-kvm \
        -nographic \
        -serial mon:stdio
}

# Run with 9p virtfs (share rootfs with host)
run_9p() {
    log "Starting QEMU with 9p virtfs..."

    if [[ ! -d "$ROOTFS_DIR/bin" ]]; then
        error "Root filesystem not found. Run build-rootfs.sh first."
    fi

    local kernel="${KERNEL:-/boot/vmlinuz-linux}"
    if [[ ! -f "$kernel" ]]; then
        kernel="/boot/vmlinuz-lts"
        if [[ ! -f "$kernel" ]]; then
            error "No kernel found. Set KERNEL=/path/to/vmlinuz"
        fi
    fi

    # Create minimal initramfs that mounts 9p
    local initramfs="/tmp/armylinux-9p-initramfs.cpio.gz"
    local tmpdir=$(mktemp -d)
    mkdir -p "$tmpdir"/{bin,dev,mnt,proc,sys}
    cp "$ROOTFS_DIR/bin/armybox" "$tmpdir/bin/"
    ln -sf armybox "$tmpdir/bin/sh"
    ln -sf armybox "$tmpdir/bin/mount"
    ln -sf armybox "$tmpdir/bin/switch_root"

    cat > "$tmpdir/init" << 'EOF'
#!/bin/sh
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev
mount -t 9p -o trans=virtio hostfs /mnt
exec switch_root /mnt /sbin/init
EOF
    chmod 755 "$tmpdir/init"

    (cd "$tmpdir" && find . | cpio -H newc -o | gzip > "$initramfs")
    rm -rf "$tmpdir"

    qemu-system-x86_64 \
        -m "$MEMORY" \
        -smp "$CPUS" \
        -kernel "$kernel" \
        -initrd "$initramfs" \
        -append "console=ttyS0 rdinit=/init" \
        -virtfs local,path="$ROOTFS_DIR",mount_tag=hostfs,security_model=passthrough,id=hostfs \
        -enable-kvm \
        -nographic \
        -serial mon:stdio
}

# Print usage
usage() {
    echo "Usage: $0 [iso|direct|9p]"
    echo ""
    echo "Modes:"
    echo "  iso    - Boot from ISO image (default)"
    echo "  direct - Direct kernel boot (faster)"
    echo "  9p     - Boot with 9p virtfs (development)"
    echo ""
    echo "Environment variables:"
    echo "  MEMORY - RAM in MB (default: 512)"
    echo "  CPUS   - Number of CPUs (default: 2)"
    echo "  KERNEL - Path to kernel (for direct/9p modes)"
    echo ""
    echo "In QEMU:"
    echo "  Ctrl+A X - Exit QEMU"
    echo "  Ctrl+A C - QEMU monitor"
}

# Main
main() {
    local mode="${1:-iso}"

    # Check for QEMU
    if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
        error "qemu-system-x86_64 not found"
        error "Install with: sudo apt install qemu-system-x86"
    fi

    case "$mode" in
        iso)
            run_iso
            ;;
        direct)
            run_direct
            ;;
        9p)
            run_9p
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            error "Unknown mode: $mode"
            usage
            exit 1
            ;;
    esac
}

main "$@"
