#!/bin/bash
# build-static.sh - Build a fully static armybox binary using musl
#
# This script builds armybox as a completely static binary that can run
# on any Linux system without any shared library dependencies.
#
# Prerequisites:
#   - Rust toolchain with rustup
#   - musl-tools (apt install musl-tools) OR
#   - Docker with cross (cargo install cross)
#
# Usage:
#   ./scripts/build-static.sh [--docker]
#
# The resulting binary will be in:
#   ./target/x86_64-unknown-linux-musl/release/armybox

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TARGET="x86_64-unknown-linux-musl"
USE_DOCKER=false

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --docker)
                USE_DOCKER=true
                shift
                ;;
            --target)
                TARGET="$2"
                shift 2
                ;;
            --help|-h)
                echo "Usage: $0 [--docker] [--target TARGET]"
                echo ""
                echo "Options:"
                echo "  --docker    Use Docker/cross for building"
                echo "  --target    Specify target (default: x86_64-unknown-linux-musl)"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
}

check_musl_native() {
    # Check if we can build natively with musl
    if command -v musl-gcc &> /dev/null; then
        return 0
    fi

    # Check for musl-tools package
    if [[ -f /usr/bin/musl-gcc ]]; then
        return 0
    fi

    return 1
}

build_native() {
    log_info "Building static binary natively with musl..."

    # Install target
    rustup target add "$TARGET" 2>/dev/null || true

    # Set environment for musl
    export CC_x86_64_unknown_linux_musl=musl-gcc
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc

    # Build with static linking
    RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes" \
        cargo build --release --target "$TARGET"
}

build_docker() {
    log_info "Building static binary using Docker/cross..."

    # Check for cross
    if ! command -v cross &> /dev/null; then
        log_info "Installing cross..."
        cargo install cross
    fi

    # Build using cross
    cross build --release --target "$TARGET"
}

verify_binary() {
    local binary="$PROJECT_DIR/target/$TARGET/release/armybox"

    if [[ ! -f "$binary" ]]; then
        log_error "Binary not found: $binary"
        exit 1
    fi

    log_info "Verifying binary..."

    # Check file type
    local file_info=$(file "$binary")
    echo "  File type: $file_info"

    # Check for static linking
    if echo "$file_info" | grep -q "statically linked"; then
        log_success "Binary is statically linked!"
    else
        log_warning "Binary may have dynamic dependencies"

        # Show dependencies if ldd is available
        if command -v ldd &> /dev/null; then
            echo "  Dependencies:"
            ldd "$binary" 2>/dev/null || echo "    (no dynamic dependencies)"
        fi
    fi

    # Show size
    local size=$(du -h "$binary" | cut -f1)
    echo "  Size: $size"

    # Show applet count
    local applets=$("$binary" --list 2>/dev/null | wc -l)
    echo "  Applets: $applets"

    # Copy to dist
    mkdir -p "$PROJECT_DIR/dist"
    cp "$binary" "$PROJECT_DIR/dist/armybox-static"
    log_success "Copied to dist/armybox-static"
}

main() {
    cd "$PROJECT_DIR"
    parse_args "$@"

    log_info "Building armybox for target: $TARGET"

    if $USE_DOCKER; then
        build_docker
    elif check_musl_native; then
        build_native
    else
        log_warning "musl-gcc not found, falling back to Docker/cross"
        build_docker
    fi

    verify_binary

    echo ""
    log_success "Static build complete!"
    echo ""
    echo "Binary location: $PROJECT_DIR/target/$TARGET/release/armybox"
    echo "Dist location:   $PROJECT_DIR/dist/armybox-static"
}

main "$@"
