#!/bin/bash
# install.sh - Universal installer for armybox
#
# This script detects your distribution and installs armybox
# using the appropriate method.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/PegasusHeavyIndustries/armybox/main/scripts/install.sh | bash
#
#   Or with options:
#   curl -fsSL ... | bash -s -- --prefix=/usr/local --symlinks

set -euo pipefail

VERSION="${ARMYBOX_VERSION:-latest}"
PREFIX="${PREFIX:-/usr/local}"
INSTALL_SYMLINKS="${INSTALL_SYMLINKS:-0}"
BINARY_URL="https://github.com/PegasusHeavyIndustries/armybox/releases/download"

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

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        armv7l|armv7)
            ARCH="armv7"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    case "$OS" in
        linux)
            # Detect Linux distribution
            if [ -f /etc/os-release ]; then
                . /etc/os-release
                DISTRO="$ID"
            elif [ -f /etc/alpine-release ]; then
                DISTRO="alpine"
            else
                DISTRO="unknown"
            fi
            ;;
        darwin)
            DISTRO="macos"
            ;;
        *)
            log_error "Unsupported OS: $OS"
            exit 1
            ;;
    esac

    log_info "Detected: $DISTRO ($OS/$ARCH)"
}

# Get latest version from GitHub
get_latest_version() {
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/PegasusHeavyIndustries/armybox/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        if [ -z "$VERSION" ]; then
            VERSION="v0.1.0"
        fi
    fi
    log_info "Version: $VERSION"
}

# Install from binary release
install_binary() {
    local target="${ARCH}-unknown-linux-musl"
    local url="${BINARY_URL}/${VERSION}/armybox-${target}"

    log_info "Downloading armybox from $url..."

    local tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    if command -v curl &> /dev/null; then
        curl -fsSL "$url" -o "$tmpdir/armybox"
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "$tmpdir/armybox"
    else
        log_error "Neither curl nor wget found"
        exit 1
    fi

    chmod +x "$tmpdir/armybox"

    log_info "Installing to $PREFIX/bin/armybox..."

    if [ -w "$PREFIX/bin" ]; then
        install -m 755 "$tmpdir/armybox" "$PREFIX/bin/armybox"
    else
        sudo install -m 755 "$tmpdir/armybox" "$PREFIX/bin/armybox"
    fi

    log_success "armybox installed successfully!"
}

# Install from source using cargo
install_from_source() {
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Install Rust from https://rustup.rs"
        exit 1
    fi

    log_info "Installing from source using cargo..."
    cargo install armybox --locked
    log_success "armybox installed via cargo!"
}

# Install using package manager
install_package() {
    case "$DISTRO" in
        ubuntu|debian|pop|linuxmint)
            log_info "Installing for Debian/Ubuntu..."
            install_binary
            ;;
        fedora|rhel|centos|rocky|alma)
            log_info "Installing for RHEL/Fedora..."
            install_binary
            ;;
        arch|manjaro|endeavouros)
            log_info "Installing for Arch Linux..."
            if command -v yay &> /dev/null; then
                yay -S armybox
            elif command -v paru &> /dev/null; then
                paru -S armybox
            else
                install_binary
            fi
            ;;
        alpine)
            log_info "Installing for Alpine Linux..."
            install_binary
            ;;
        *)
            log_warn "Unknown distribution, installing from binary..."
            install_binary
            ;;
    esac
}

# Install symlinks
install_symlinks() {
    if [ "$INSTALL_SYMLINKS" = "1" ]; then
        log_info "Installing symlinks..."
        if [ -w "$PREFIX/bin" ]; then
            "$PREFIX/bin/armybox" --install "$PREFIX/bin"
        else
            sudo "$PREFIX/bin/armybox" --install "$PREFIX/bin"
        fi
        log_success "Symlinks installed!"
    fi
}

# Show usage
usage() {
    cat << EOF
armybox installer

Usage: $0 [OPTIONS]

Options:
    --prefix=PATH       Installation prefix (default: /usr/local)
    --symlinks          Install symlinks for all applets
    --source            Install from source using cargo
    --version=VERSION   Install specific version (default: latest)
    --help              Show this help message

Environment variables:
    ARMYBOX_VERSION     Version to install
    PREFIX              Installation prefix
    INSTALL_SYMLINKS    Set to 1 to install symlinks

Examples:
    # Install latest binary
    $0

    # Install with symlinks
    $0 --symlinks

    # Install to custom location
    $0 --prefix=/opt/armybox --symlinks

    # Install from source
    $0 --source
EOF
}

# Parse arguments
parse_args() {
    local from_source=0

    while [ $# -gt 0 ]; do
        case "$1" in
            --prefix=*)
                PREFIX="${1#*=}"
                ;;
            --symlinks)
                INSTALL_SYMLINKS=1
                ;;
            --source)
                from_source=1
                ;;
            --version=*)
                VERSION="${1#*=}"
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
        shift
    done

    if [ "$from_source" = "1" ]; then
        install_from_source
        exit 0
    fi
}

# Main
main() {
    parse_args "$@"

    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║       armybox installer                ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    detect_platform
    get_latest_version
    install_package
    install_symlinks

    echo ""
    log_success "Installation complete!"
    echo ""
    echo "  armybox is installed at: $PREFIX/bin/armybox"
    echo ""
    echo "  Try it:"
    echo "    $PREFIX/bin/armybox --list"
    echo "    $PREFIX/bin/armybox ls -la"
    echo ""

    if [ "$INSTALL_SYMLINKS" != "1" ]; then
        echo "  To install symlinks for all applets:"
        echo "    armybox --install $PREFIX/bin"
        echo ""
    fi
}

main "$@"
