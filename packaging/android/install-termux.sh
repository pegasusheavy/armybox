#!/data/data/com.termux/files/usr/bin/bash
# install-termux.sh - Install armybox in Termux
#
# Usage: curl -fsSL <url> | bash

set -euo pipefail

VERSION="${ARMYBOX_VERSION:-latest}"
PREFIX="${PREFIX:-$HOME/.local}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        aarch64|arm64)
            echo "aarch64-linux-android"
            ;;
        armv7l|armv8l)
            echo "armv7-linux-androideabi"
            ;;
        x86_64)
            echo "x86_64-linux-android"
            ;;
        i686|i386)
            echo "i686-linux-android"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
}

# Check if running in Termux
check_termux() {
    if [[ ! -d "/data/data/com.termux" ]]; then
        log_error "This script is designed for Termux"
        log_error "Install Termux from F-Droid: https://f-droid.org/packages/com.termux/"
        exit 1
    fi
}

# Install dependencies
install_deps() {
    log_info "Installing dependencies..."
    pkg update -y
    pkg install -y curl
}

# Download and install
install_armybox() {
    local arch=$(detect_arch)
    local url="https://github.com/PegasusHeavyIndustries/armybox/releases/download/${VERSION}/armybox-${arch}"

    log_info "Downloading armybox for $arch..."

    mkdir -p "$PREFIX/bin"

    if curl -fsSL "$url" -o "$PREFIX/bin/armybox"; then
        chmod +x "$PREFIX/bin/armybox"
        log_success "Downloaded armybox"
    else
        log_info "Pre-built binary not available, building from source..."
        build_from_source
        return
    fi

    # Verify
    if "$PREFIX/bin/armybox" --version >/dev/null 2>&1; then
        log_success "armybox installed successfully!"
    else
        log_error "Installation failed"
        exit 1
    fi
}

# Build from source
build_from_source() {
    log_info "Building from source..."

    # Install Rust if needed
    if ! command -v cargo &>/dev/null; then
        log_info "Installing Rust..."
        pkg install -y rust
    fi

    # Clone and build
    local tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    cd "$tmpdir"

    if command -v git &>/dev/null; then
        git clone --depth 1 https://github.com/PegasusHeavyIndustries/armybox
    else
        pkg install -y git
        git clone --depth 1 https://github.com/PegasusHeavyIndustries/armybox
    fi

    cd armybox
    cargo build --release

    mkdir -p "$PREFIX/bin"
    cp target/release/armybox "$PREFIX/bin/"
    chmod +x "$PREFIX/bin/armybox"

    log_success "Built and installed armybox"
}

# Install symlinks
install_symlinks() {
    log_info "Installing symlinks..."
    "$PREFIX/bin/armybox" --install "$PREFIX/bin"
    log_success "Symlinks installed"
}

# Update PATH
update_path() {
    if [[ ":$PATH:" != *":$PREFIX/bin:"* ]]; then
        log_info "Adding $PREFIX/bin to PATH..."

        # Add to shell config
        local shell_rc=""
        if [[ -f "$HOME/.bashrc" ]]; then
            shell_rc="$HOME/.bashrc"
        elif [[ -f "$HOME/.zshrc" ]]; then
            shell_rc="$HOME/.zshrc"
        fi

        if [[ -n "$shell_rc" ]]; then
            echo "export PATH=\"$PREFIX/bin:\$PATH\"" >> "$shell_rc"
            log_info "Added to $shell_rc - restart shell or run: source $shell_rc"
        fi

        export PATH="$PREFIX/bin:$PATH"
    fi
}

# Main
main() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║    armybox Termux Installer            ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    check_termux
    install_deps
    install_armybox
    install_symlinks
    update_path

    echo ""
    log_success "Installation complete!"
    echo ""
    echo "  armybox is installed at: $PREFIX/bin/armybox"
    echo ""
    echo "  Try it:"
    echo "    armybox --list"
    echo "    armybox ls -la"
    echo "    armybox sh"
    echo ""
}

main "$@"
