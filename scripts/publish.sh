#!/bin/bash
# publish.sh - Publish armybox to crates.io
#
# Prerequisites:
#   1. Login to crates.io: cargo login <your-api-token>
#   2. Verify package: cargo publish --dry-run
#
# Usage: ./scripts/publish.sh [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

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

DRY_RUN=""
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN="--dry-run"
    log_info "Running in dry-run mode"
fi

# Pre-flight checks
log_info "Running pre-flight checks..."

# Check Cargo.toml version
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
log_info "Version: $VERSION"

# Check if git is clean (for non-dry-run)
if [[ -z "$DRY_RUN" ]]; then
    if [[ -n "$(git status --porcelain)" ]]; then
        log_warn "Working directory is not clean. Consider committing changes first."
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
fi

# Run tests
log_info "Running tests..."
cargo test --release || {
    log_error "Tests failed"
    exit 1
}
log_success "Tests passed"

# Build release
log_info "Building release..."
cargo build --release || {
    log_error "Build failed"
    exit 1
}
log_success "Build succeeded"

# Package
log_info "Creating package..."
cargo package --allow-dirty || {
    log_error "Package creation failed"
    exit 1
}
log_success "Package created: target/package/armybox-${VERSION}.crate"

# Show package info
log_info "Package contents:"
tar -tzf "target/package/armybox-${VERSION}.crate" | head -20
echo "..."
TOTAL=$(tar -tzf "target/package/armybox-${VERSION}.crate" | wc -l)
echo "Total: $TOTAL files"

SIZE=$(ls -lh "target/package/armybox-${VERSION}.crate" | awk '{print $5}')
log_info "Package size: $SIZE"

# Publish
if [[ -n "$DRY_RUN" ]]; then
    log_info "Dry-run: would publish armybox v${VERSION}"
    cargo publish --dry-run --allow-dirty
    log_success "Dry-run complete"
else
    log_info "Publishing armybox v${VERSION} to crates.io..."
    read -p "Are you sure you want to publish? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cargo publish
        log_success "Published armybox v${VERSION} to crates.io!"

        echo ""
        echo "Next steps:"
        echo "  1. Create git tag: git tag v${VERSION}"
        echo "  2. Push tag: git push origin v${VERSION}"
        echo "  3. Create GitHub release"
    else
        log_info "Publish cancelled"
    fi
fi
