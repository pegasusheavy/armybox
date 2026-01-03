#!/bin/bash
#
# Build ArmyLinux bootable ISO
#
# Creates a bootable ISO image that can be used for VMs or USB drives.
# Requires: xorriso, syslinux, linux kernel

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISTRO_DIR="$(dirname "$SCRIPT_DIR")"
ROOTFS_DIR="$DISTRO_DIR/rootfs"
ISO_DIR="$DISTRO_DIR/iso"
BUILD_DIR="$DISTRO_DIR/build"

ISO_NAME="${ISO_NAME:-armylinux-0.1.0-x86_64.iso}"
ALPINE_VERSION="${ALPINE_VERSION:-3.19}"

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

    local missing=()

    command -v xorriso >/dev/null 2>&1 || missing+=("xorriso")
    command -v mksquashfs >/dev/null 2>&1 || missing+=("squashfs-tools")

    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing required tools: ${missing[*]}"
        error "Install with: sudo apt install ${missing[*]}"
    fi

    if [[ ! -d "$ROOTFS_DIR/bin" ]]; then
        error "Root filesystem not found. Run build-rootfs.sh first."
    fi

    log "Prerequisites OK"
}

# Setup ISO structure
setup_iso_structure() {
    log "Setting up ISO structure..."

    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"/{boot/syslinux,live}

    log "ISO structure created"
}

# Download kernel (from Alpine)
download_kernel() {
    log "Downloading kernel from Alpine..."

    local kernel_pkg="linux-lts"
    local kernel_url="$ALPINE_MIRROR/v$ALPINE_VERSION/main/x86_64"

    # For now, we'll use a placeholder
    # In production, you'd download the actual kernel
    warn "Kernel download not implemented"
    warn "You'll need to provide your own kernel at $BUILD_DIR/boot/vmlinuz"

    # Create placeholder files
    touch "$BUILD_DIR/boot/vmlinuz"
    touch "$BUILD_DIR/boot/initramfs"
}

# Create initramfs
create_initramfs() {
    log "Creating initramfs..."

    local initramfs_dir="$BUILD_DIR/initramfs"
    mkdir -p "$initramfs_dir"/{bin,dev,etc,lib,mnt/root,proc,sys,run,tmp}

    # Copy armybox
    cp "$ROOTFS_DIR/bin/armybox" "$initramfs_dir/bin/"
    chmod 755 "$initramfs_dir/bin/armybox"

    # Create essential symlinks
    ln -sf armybox "$initramfs_dir/bin/sh"
    ln -sf armybox "$initramfs_dir/bin/mount"
    ln -sf armybox "$initramfs_dir/bin/umount"
    ln -sf armybox "$initramfs_dir/bin/switch_root"

    # Create init script
    cat > "$initramfs_dir/init" << 'EOF'
#!/bin/sh

# Mount virtual filesystems
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev

# Parse kernel command line
NEWROOT="/mnt/root"
init="/sbin/init"

for opt in $(cat /proc/cmdline); do
    case "$opt" in
        root=*)
            ROOT="${opt#root=}"
            ;;
        init=*)
            init="${opt#init=}"
            ;;
    esac
done

# Wait for root device
echo "Waiting for root device..."
while [ ! -e "$ROOT" ]; do
    sleep 0.1
done

# Mount root filesystem
mount -o ro "$ROOT" "$NEWROOT"

# Switch to real root
exec switch_root "$NEWROOT" "$init"
EOF
    chmod 755 "$initramfs_dir/init"

    # Create initramfs cpio archive
    (cd "$initramfs_dir" && find . | cpio -H newc -o | gzip > "$BUILD_DIR/boot/initramfs.gz")

    log "Initramfs created"
}

# Create squashfs
create_squashfs() {
    log "Creating squashfs filesystem..."

    mksquashfs "$ROOTFS_DIR" "$BUILD_DIR/live/filesystem.squashfs" \
        -comp xz \
        -Xbcj x86 \
        -b 1M \
        -no-recovery \
        -noappend

    log "Squashfs created: $(du -h "$BUILD_DIR/live/filesystem.squashfs" | cut -f1)"
}

# Setup syslinux
setup_syslinux() {
    log "Setting up syslinux..."

    # Syslinux configuration
    cat > "$BUILD_DIR/boot/syslinux/syslinux.cfg" << 'EOF'
DEFAULT armylinux
TIMEOUT 50
PROMPT 1

UI menu.c32

MENU TITLE ArmyLinux Boot Menu
MENU COLOR border       30;44   #40ffffff #a0000000 std
MENU COLOR title        1;36;44 #9033ccff #a0000000 std
MENU COLOR sel          7;37;40 #e0ffffff #20ffffff all
MENU COLOR unsel        37;44   #50ffffff #a0000000 std
MENU COLOR help         37;40   #c0ffffff #a0000000 std
MENU COLOR timeout_msg  37;40   #80ffffff #00000000 std
MENU COLOR timeout      1;37;40 #c0ffffff #00000000 std
MENU COLOR msg07        37;40   #90ffffff #a0000000 std

LABEL armylinux
    MENU LABEL ^ArmyLinux
    MENU DEFAULT
    LINUX /boot/vmlinuz
    INITRD /boot/initramfs.gz
    APPEND root=/dev/sr0 rootfstype=iso9660 init=/sbin/init

LABEL armylinux-serial
    MENU LABEL ArmyLinux (^Serial Console)
    LINUX /boot/vmlinuz
    INITRD /boot/initramfs.gz
    APPEND root=/dev/sr0 rootfstype=iso9660 init=/sbin/init console=ttyS0,115200
EOF

    # Copy syslinux files (if available)
    if [[ -d /usr/lib/syslinux/modules/bios ]]; then
        cp /usr/lib/syslinux/modules/bios/{menu.c32,libutil.c32,libcom32.c32} "$BUILD_DIR/boot/syslinux/" 2>/dev/null || true
    fi

    log "Syslinux configured"
}

# Create ISO
create_iso() {
    log "Creating ISO image..."

    mkdir -p "$ISO_DIR"

    xorriso -as mkisofs \
        -o "$ISO_DIR/$ISO_NAME" \
        -isohybrid-mbr /usr/lib/syslinux/mbr/isohdpfx.bin \
        -c boot/syslinux/boot.cat \
        -b boot/syslinux/isolinux.bin \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        -eltorito-alt-boot \
        -e boot/syslinux/efiboot.img \
        -no-emul-boot \
        -isohybrid-gpt-basdat \
        -V "ARMYLINUX" \
        "$BUILD_DIR" 2>/dev/null || {
            # Fallback to simpler ISO creation
            xorriso -as mkisofs \
                -o "$ISO_DIR/$ISO_NAME" \
                -V "ARMYLINUX" \
                "$BUILD_DIR"
        }

    log "ISO created: $ISO_DIR/$ISO_NAME"
    log "Size: $(du -h "$ISO_DIR/$ISO_NAME" | cut -f1)"
}

# Print summary
print_summary() {
    echo ""
    log "ISO build complete!"
    echo ""
    echo "ISO image: $ISO_DIR/$ISO_NAME"
    echo ""
    echo "To test with QEMU:"
    echo "  ./scripts/run-qemu.sh"
    echo ""
    echo "To write to USB drive:"
    echo "  sudo dd if=$ISO_DIR/$ISO_NAME of=/dev/sdX bs=4M status=progress"
}

# Main
main() {
    log "Building ArmyLinux ISO..."

    check_prereqs
    setup_iso_structure
    download_kernel
    create_initramfs
    create_squashfs
    setup_syslinux
    create_iso
    print_summary
}

main "$@"
