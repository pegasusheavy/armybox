#!/bin/bash
# build-ios.sh - Build armybox for iOS (jailbroken devices)
#
# Prerequisites:
#   - macOS with Xcode installed
#   - Rust iOS targets added
#   - (Optional) ldid for code signing
#   - (Optional) dpkg for .deb creation

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VERSION=$(grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)
OUTPUT_DIR="$PROJECT_DIR/dist/ios"

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

# Check prerequisites
check_prereqs() {
    log_info "Checking prerequisites..."

    # Check OS
    if [[ "$(uname)" != "Darwin" ]]; then
        log_error "This script must be run on macOS"
        exit 1
    fi

    # Check Xcode
    if ! command -v xcrun &>/dev/null; then
        log_error "Xcode command line tools not found"
        echo "Install with: xcode-select --install"
        exit 1
    fi

    # Check iOS SDK
    if ! xcrun --sdk iphoneos --show-sdk-path &>/dev/null; then
        log_error "iOS SDK not found"
        echo "Install Xcode from the App Store"
        exit 1
    fi

    log_success "Prerequisites OK"
}

# Add Rust targets
add_targets() {
    log_info "Adding Rust iOS targets..."

    if ! rustup target list --installed | grep -q "aarch64-apple-ios"; then
        rustup target add aarch64-apple-ios
    fi

    log_success "Targets ready"
}

# Build for iOS
build_ios() {
    log_info "Building for iOS (ARM64)..."

    cd "$PROJECT_DIR"

    # Set SDK path
    export SDKROOT=$(xcrun --sdk iphoneos --show-sdk-path)

    # Build
    cargo build --release --target aarch64-apple-ios

    # Copy to output
    mkdir -p "$OUTPUT_DIR"
    cp "target/aarch64-apple-ios/release/armybox" "$OUTPUT_DIR/armybox-aarch64-apple-ios"

    # Sign if ldid is available
    if command -v ldid &>/dev/null; then
        log_info "Signing with ldid..."
        ldid -S"$PROJECT_DIR/packaging/ios/entitlements.plist" "$OUTPUT_DIR/armybox-aarch64-apple-ios"
    else
        log_warn "ldid not found, binary will need to be signed on device"
    fi

    log_success "Built: $OUTPUT_DIR/armybox-aarch64-apple-ios"
}

# Create .deb package
create_deb() {
    log_info "Creating .deb package..."

    if ! command -v dpkg-deb &>/dev/null; then
        log_warn "dpkg-deb not found, skipping .deb creation"
        echo "Install with: brew install dpkg"
        return
    fi

    cd "$PROJECT_DIR/packaging/ios"
    make package

    # Move to output
    mv packages/*.deb "$OUTPUT_DIR/"

    log_success "Created .deb package"
}

# Show usage
usage() {
    cat << EOF
Build armybox for iOS (jailbroken devices)

Usage: $0 [COMMAND]

Commands:
    build       Build the binary only
    package     Build and create .deb package
    help        Show this help

Requirements:
    - macOS with Xcode
    - Rust with aarch64-apple-ios target

Examples:
    $0 build        # Just build the binary
    $0 package      # Build and create .deb
EOF
}

# Main
main() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║    Armybox iOS Builder                 ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    check_prereqs
    add_targets

    mkdir -p "$OUTPUT_DIR"

    case "${1:-build}" in
        build)
            build_ios
            ;;
        package)
            build_ios
            create_deb
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
    echo "  To install on device:"
    echo "    scp $OUTPUT_DIR/armybox-aarch64-apple-ios root@<device-ip>:/usr/bin/armybox"
    echo "    ssh root@<device-ip> 'ldid -S /usr/bin/armybox && chmod +x /usr/bin/armybox'"
    echo ""
}

main "$@"
