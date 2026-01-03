#!/bin/bash
# build-android.sh - Build armybox for Android
#
# Prerequisites:
#   - Android NDK installed (set ANDROID_NDK_HOME)
#   - cargo-ndk installed: cargo install cargo-ndk
#   - Rust Android targets added

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VERSION=$(grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)
OUTPUT_DIR="$PROJECT_DIR/dist/android"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Targets
TARGETS=(
    "arm64-v8a:aarch64-linux-android"
    "armeabi-v7a:armv7-linux-androideabi"
    "x86_64:x86_64-linux-android"
    "x86:i686-linux-android"
)

# Check prerequisites
check_prereqs() {
    log_info "Checking prerequisites..."

    # Check cargo-ndk
    if ! command -v cargo-ndk &>/dev/null; then
        log_error "cargo-ndk not found"
        echo "Install with: cargo install cargo-ndk"
        exit 1
    fi

    # Check NDK
    if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
        # Try to find NDK
        local ndk_paths=(
            "$HOME/Android/Sdk/ndk"
            "$HOME/Library/Android/sdk/ndk"
            "/opt/android-ndk"
        )

        for path in "${ndk_paths[@]}"; do
            if [[ -d "$path" ]]; then
                # Get latest version
                latest=$(ls -1 "$path" 2>/dev/null | sort -V | tail -1)
                if [[ -n "$latest" ]]; then
                    export ANDROID_NDK_HOME="$path/$latest"
                    log_info "Found NDK at: $ANDROID_NDK_HOME"
                    break
                fi
            fi
        done

        if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
            log_error "Android NDK not found"
            echo "Set ANDROID_NDK_HOME or install NDK via Android Studio"
            exit 1
        fi
    fi

    log_success "Prerequisites OK"
}

# Add Rust targets
add_targets() {
    log_info "Adding Rust targets..."

    for target_pair in "${TARGETS[@]}"; do
        local rust_target="${target_pair#*:}"
        if ! rustup target list --installed | grep -q "$rust_target"; then
            log_info "Adding target: $rust_target"
            rustup target add "$rust_target"
        fi
    done

    log_success "Targets ready"
}

# Build for a specific target
build_target() {
    local abi="$1"
    local rust_target="$2"

    log_info "Building for $abi ($rust_target)..."

    cd "$PROJECT_DIR"

    # Build with cargo-ndk
    cargo ndk -t "$abi" build --release

    # Copy to output
    mkdir -p "$OUTPUT_DIR"
    cp "target/$rust_target/release/armybox" "$OUTPUT_DIR/armybox-$rust_target"

    # Strip if possible
    if command -v "${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" &>/dev/null; then
        "${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" "$OUTPUT_DIR/armybox-$rust_target"
    fi

    log_success "Built: $OUTPUT_DIR/armybox-$rust_target"
}

# Build all targets
build_all() {
    for target_pair in "${TARGETS[@]}"; do
        local abi="${target_pair%:*}"
        local rust_target="${target_pair#*:}"
        build_target "$abi" "$rust_target"
    done
}

# Build only ARM64 (most common)
build_arm64() {
    build_target "arm64-v8a" "aarch64-linux-android"
}

# Show usage
usage() {
    cat << EOF
Build armybox for Android

Usage: $0 [COMMAND]

Commands:
    all         Build for all architectures
    arm64       Build for ARM64 only (most common)
    arm32       Build for ARM32 only
    x86_64      Build for x86_64 only
    help        Show this help

Environment:
    ANDROID_NDK_HOME    Path to Android NDK (auto-detected if not set)

Examples:
    $0 arm64            # Build for modern phones
    $0 all              # Build for all architectures
EOF
}

# Main
main() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║    Armybox Android Builder             ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    check_prereqs
    add_targets

    mkdir -p "$OUTPUT_DIR"

    case "${1:-arm64}" in
        all)
            build_all
            ;;
        arm64)
            build_target "arm64-v8a" "aarch64-linux-android"
            ;;
        arm32)
            build_target "armeabi-v7a" "armv7-linux-androideabi"
            ;;
        x86_64)
            build_target "x86_64" "x86_64-linux-android"
            ;;
        x86)
            build_target "x86" "i686-linux-android"
            ;;
        help|--help|-h)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown command: $1"
            usage
            exit 1
            ;;
    esac

    echo ""
    log_success "Build complete!"
    echo ""
    echo "  Output: $OUTPUT_DIR/"
    ls -lh "$OUTPUT_DIR/"
    echo ""
    echo "  To push to device:"
    echo "    adb push $OUTPUT_DIR/armybox-aarch64-linux-android /data/local/tmp/armybox"
    echo "    adb shell chmod +x /data/local/tmp/armybox"
    echo "    adb shell /data/local/tmp/armybox --list"
    echo ""
}

main "$@"
