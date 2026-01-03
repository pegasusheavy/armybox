#!/bin/bash
# build-magisk-module.sh - Build Magisk module for armybox
#
# Prerequisites:
#   - Android NDK installed
#   - cargo-ndk installed: cargo install cargo-ndk
#   - Rust Android targets: rustup target add aarch64-linux-android armv7-linux-androideabi

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
VERSION=$(grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)
OUTPUT_DIR="$PROJECT_DIR/dist/android"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Check prerequisites
check_prereqs() {
    if ! command -v cargo-ndk &>/dev/null; then
        log_error "cargo-ndk not found. Install with: cargo install cargo-ndk"
        exit 1
    fi

    if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
        log_error "ANDROID_NDK_HOME not set"
        exit 1
    fi
}

# Build for Android
build_android() {
    local arch="$1"
    local target="$2"

    log_info "Building for $arch ($target)..."

    cd "$PROJECT_DIR"
    cargo ndk -t "$arch" build --release

    log_success "Built for $arch"
}

# Create Magisk module
create_module() {
    log_info "Creating Magisk module..."

    local module_dir="$OUTPUT_DIR/magisk-module"
    rm -rf "$module_dir"
    mkdir -p "$module_dir/system/bin"
    mkdir -p "$module_dir/META-INF/com/google/android"

    # Copy module files
    cp "$SCRIPT_DIR/magisk/module.prop" "$module_dir/"
    cp "$SCRIPT_DIR/magisk/customize.sh" "$module_dir/"
    cp "$SCRIPT_DIR/magisk/post-fs-data.sh" "$module_dir/"
    cp "$SCRIPT_DIR/magisk/service.sh" "$module_dir/"

    # Update version in module.prop
    sed -i "s/^version=.*/version=v${VERSION}/" "$module_dir/module.prop"

    # Create update-binary for Magisk
    cat > "$module_dir/META-INF/com/google/android/update-binary" << 'EOF'
#!/sbin/sh

#################
# Initialization
#################

umask 022

# Global vars
TMPDIR=/dev/tmp
PERSISTDIR=/sbin/.magisk/mirror/persist

rm -rf $TMPDIR 2>/dev/null
mkdir -p $TMPDIR

# echo before loading util_functions
ui_print() { echo "$1"; }

require_new_magisk() {
  ui_print "*******************************"
  ui_print " Please install Magisk v20.4+! "
  ui_print "*******************************"
  exit 1
}

##############
# Environment
##############

OUTFD=$2
ZIPFILE=$3

mount /data 2>/dev/null

# Load utility functions
[ -f /data/adb/magisk/util_functions.sh ] || require_new_magisk
. /data/adb/magisk/util_functions.sh
[ $MAGISK_VER_CODE -lt 20400 ] && require_new_magisk

# Preperation for flashable zips
setup_flashable

# Mount partitions
mount_partitions

# Detect version and architecture
api_level_arch_detect

# Setup busybox and binaries
$BOOTMODE && boot_actions || recovery_actions

##############
# Preparation
##############

# Extract prop file
unzip -o "$ZIPFILE" module.prop -d $TMPDIR >&2
[ ! -f $TMPDIR/module.prop ] && abort "! Unable to extract zip file!"

$BOOTMODE && MODDIRNAME=modules_update || MODDIRNAME=modules
MODULEROOT=$NVBASE/$MODDIRNAME
MODID=`grep_prop id $TMPDIR/module.prop`
MODPATH=$MODULEROOT/$MODID
MODNAME=`grep_prop name $TMPDIR/module.prop`

# Create mod paths
rm -rf $MODPATH 2>/dev/null
mkdir -p $MODPATH

##########
# Install
##########

# Extract to module path
ui_print "- Extracting module files"
unzip -o "$ZIPFILE" -x 'META-INF/*' -d $MODPATH >&2

# Default permissions
set_perm_recursive $MODPATH 0 0 0755 0644

# Custom install script
[ -f $MODPATH/customize.sh ] && . $MODPATH/customize.sh

# Handle replace folders
for TARGET in $REPLACE; do
  mktouch $MODPATH$TARGET/.replace
done

# Auto Mount
$SKIPMOUNT && touch $MODPATH/skip_mount

# prop files
$PROPFILE && copy_propsfile

# Module info
cp -af $TMPDIR/module.prop $MODPATH/module.prop

# post-fs-data scripts
$POSTFSDATA && cp -af $TMPDIR/post-fs-data.sh $MODPATH/post-fs-data.sh

# service scripts
$LATESTARTSERVICE && cp -af $TMPDIR/service.sh $MODPATH/service.sh

# Handle addon.d
ADDON_D=false
$ADDON_D && copy_addon_d

ui_print "- Done"
EOF

    cat > "$module_dir/META-INF/com/google/android/updater-script" << 'EOF'
#MAGISK
EOF

    chmod 755 "$module_dir/META-INF/com/google/android/update-binary"

    # Copy ARM64 binary (primary)
    if [[ -f "$PROJECT_DIR/target/aarch64-linux-android/release/armybox" ]]; then
        cp "$PROJECT_DIR/target/aarch64-linux-android/release/armybox" "$module_dir/system/bin/"
        chmod 755 "$module_dir/system/bin/armybox"
    else
        log_error "ARM64 binary not found. Build with: cargo ndk -t arm64-v8a build --release"
        exit 1
    fi

    # Create zip
    cd "$module_dir"
    zip -r "$OUTPUT_DIR/armybox-magisk-v${VERSION}.zip" .

    log_success "Created: $OUTPUT_DIR/armybox-magisk-v${VERSION}.zip"
}

# Main
main() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║    Armybox Magisk Module Builder       ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    check_prereqs

    mkdir -p "$OUTPUT_DIR"

    # Build for ARM64 (most common)
    build_android "arm64-v8a" "aarch64-linux-android"

    # Optionally build for ARM32
    # build_android "armeabi-v7a" "armv7-linux-androideabi"

    create_module

    echo ""
    log_success "Build complete!"
    echo ""
    echo "  Magisk module: $OUTPUT_DIR/armybox-magisk-v${VERSION}.zip"
    echo ""
    echo "  To install:"
    echo "    1. Copy to device: adb push $OUTPUT_DIR/armybox-magisk-v${VERSION}.zip /sdcard/"
    echo "    2. Open Magisk Manager → Modules → Install from storage"
    echo "    3. Select the zip and reboot"
    echo ""
}

main "$@"
