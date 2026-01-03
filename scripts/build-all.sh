#!/bin/bash
# build-all.sh - Build armybox for all supported targets
#
# Prerequisites:
#   - Rust toolchain with rustup
#   - cross (cargo install cross)
#   - Docker (for cross-compilation)
#
# Usage:
#   ./scripts/build-all.sh [target...]
#
# Examples:
#   ./scripts/build-all.sh                    # Build all targets
#   ./scripts/build-all.sh x86_64 arm64       # Build specific targets
#   ./scripts/build-all.sh --native           # Build only native target

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_DIR/dist"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# All supported targets
declare -A TARGETS=(
    ["x86_64"]="x86_64-unknown-linux-musl"
    ["x86"]="i686-unknown-linux-musl"
    ["arm64"]="aarch64-unknown-linux-musl"
    ["arm32"]="armv7-unknown-linux-musleabihf"
    ["arm32-soft"]="arm-unknown-linux-musleabi"
    ["mips"]="mips-unknown-linux-musl"
    ["mipsel"]="mipsel-unknown-linux-musl"
    ["mips64"]="mips64-unknown-linux-muslabi64"
    ["riscv64"]="riscv64gc-unknown-linux-gnu"
    ["ppc"]="powerpc-unknown-linux-gnu"
    ["ppc64"]="powerpc64-unknown-linux-gnu"
    ["ppc64le"]="powerpc64le-unknown-linux-gnu"
)

# Musl targets (statically linked)
MUSL_TARGETS=(
    "x86_64-unknown-linux-musl"
    "i686-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "armv7-unknown-linux-musleabihf"
    "arm-unknown-linux-musleabi"
    "mips-unknown-linux-musl"
    "mipsel-unknown-linux-musl"
    "mips64-unknown-linux-muslabi64"
)

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust."
        exit 1
    fi

    if ! command -v cross &> /dev/null; then
        log_warning "cross not found. Installing..."
        cargo install cross
    fi

    if ! command -v docker &> /dev/null; then
        log_warning "Docker not found. Cross-compilation may fail."
    fi
}

install_target() {
    local target=$1
    log_info "Installing target: $target"
    rustup target add "$target" 2>/dev/null || true
}

build_target() {
    local target=$1
    local output_name="armybox-${target}"

    log_info "Building for $target..."

    # Use cross for non-native targets
    if [[ "$target" == "$(rustc -vV | grep host | cut -d' ' -f2)" ]]; then
        cargo build --release --target "$target"
    else
        cross build --release --target "$target"
    fi

    if [[ $? -eq 0 ]]; then
        # Copy binary to dist directory
        mkdir -p "$OUTPUT_DIR"
        cp "$PROJECT_DIR/target/$target/release/armybox" "$OUTPUT_DIR/$output_name"

        # Get binary size
        local size=$(du -h "$OUTPUT_DIR/$output_name" | cut -f1)

        # Check if statically linked
        local link_type="dynamic"
        if file "$OUTPUT_DIR/$output_name" | grep -q "statically linked"; then
            link_type="static"
        fi

        log_success "Built $output_name ($size, $link_type)"
    else
        log_error "Failed to build for $target"
        return 1
    fi
}

build_native() {
    log_info "Building native release..."
    cargo build --release

    mkdir -p "$OUTPUT_DIR"
    cp "$PROJECT_DIR/target/release/armybox" "$OUTPUT_DIR/armybox-native"

    local size=$(du -h "$OUTPUT_DIR/armybox-native" | cut -f1)
    log_success "Built armybox-native ($size)"
}

build_static_native() {
    log_info "Building static native release with musl..."

    local native_target="x86_64-unknown-linux-musl"
    install_target "$native_target"

    RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "$native_target"

    mkdir -p "$OUTPUT_DIR"
    cp "$PROJECT_DIR/target/$native_target/release/armybox" "$OUTPUT_DIR/armybox-static"

    local size=$(du -h "$OUTPUT_DIR/armybox-static" | cut -f1)
    log_success "Built armybox-static ($size)"
}

print_summary() {
    echo ""
    echo "=============================================="
    echo "Build Summary"
    echo "=============================================="

    if [[ -d "$OUTPUT_DIR" ]]; then
        echo ""
        echo "Output directory: $OUTPUT_DIR"
        echo ""
        echo "Built binaries:"
        ls -lh "$OUTPUT_DIR"/armybox-* 2>/dev/null | while read line; do
            echo "  $line"
        done
    fi

    echo ""
}

show_help() {
    echo "Usage: $0 [OPTIONS] [TARGETS...]"
    echo ""
    echo "Build armybox for multiple architectures"
    echo ""
    echo "Options:"
    echo "  --help, -h      Show this help message"
    echo "  --native        Build only native target"
    echo "  --static        Build static native binary (musl)"
    echo "  --all           Build all supported targets"
    echo "  --list          List all supported targets"
    echo "  --clean         Clean build artifacts"
    echo ""
    echo "Targets:"
    for key in "${!TARGETS[@]}"; do
        echo "  $key -> ${TARGETS[$key]}"
    done
    echo ""
    echo "Examples:"
    echo "  $0 --native           # Build native only"
    echo "  $0 --static           # Build static musl binary"
    echo "  $0 x86_64 arm64       # Build for specific targets"
    echo "  $0 --all              # Build all targets"
}

main() {
    cd "$PROJECT_DIR"

    if [[ $# -eq 0 ]]; then
        show_help
        exit 0
    fi

    case "$1" in
        --help|-h)
            show_help
            exit 0
            ;;
        --list)
            echo "Supported targets:"
            for key in "${!TARGETS[@]}"; do
                echo "  $key -> ${TARGETS[$key]}"
            done
            exit 0
            ;;
        --clean)
            log_info "Cleaning build artifacts..."
            cargo clean
            rm -rf "$OUTPUT_DIR"
            log_success "Cleaned"
            exit 0
            ;;
        --native)
            check_prerequisites
            build_native
            print_summary
            exit 0
            ;;
        --static)
            check_prerequisites
            build_static_native
            print_summary
            exit 0
            ;;
        --all)
            check_prerequisites
            build_native
            for key in "${!TARGETS[@]}"; do
                target="${TARGETS[$key]}"
                install_target "$target"
                build_target "$target" || true
            done
            print_summary
            exit 0
            ;;
        *)
            check_prerequisites
            for arg in "$@"; do
                if [[ -n "${TARGETS[$arg]}" ]]; then
                    target="${TARGETS[$arg]}"
                    install_target "$target"
                    build_target "$target"
                else
                    log_error "Unknown target: $arg"
                    log_info "Use --list to see available targets"
                fi
            done
            print_summary
            ;;
    esac
}

main "$@"
