#!/bin/bash
#
# Build ArmyLinux Docker image
#
# Creates a minimal Docker image using armybox as the base.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISTRO_DIR="$(dirname "$SCRIPT_DIR")"
ARMYBOX_DIR="$(dirname "$DISTRO_DIR")"
ROOTFS_DIR="$DISTRO_DIR/rootfs"

IMAGE_NAME="${IMAGE_NAME:-armylinux}"
IMAGE_TAG="${IMAGE_TAG:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}==>${NC} $*"; }
warn() { echo -e "${YELLOW}WARNING:${NC} $*"; }
error() { echo -e "${RED}ERROR:${NC} $*" >&2; exit 1; }

# Build using the root filesystem
build_from_rootfs() {
    log "Building Docker image from root filesystem..."

    if [[ ! -d "$ROOTFS_DIR/bin" ]]; then
        error "Root filesystem not found. Run build-rootfs.sh first."
    fi

    # Create a tarball of the rootfs
    local tarball="/tmp/armylinux-rootfs.tar"
    log "Creating rootfs tarball..."
    tar -C "$ROOTFS_DIR" -cf "$tarball" .

    # Import as Docker image
    log "Importing into Docker..."
    docker import "$tarball" "$IMAGE_NAME:$IMAGE_TAG"

    # Clean up
    rm -f "$tarball"

    log "Image created: $IMAGE_NAME:$IMAGE_TAG"
}

# Build using Dockerfile
build_from_dockerfile() {
    log "Building Docker image from Dockerfile..."

    cd "$ARMYBOX_DIR"

    docker build \
        -f distro/Dockerfile \
        -t "$IMAGE_NAME:$IMAGE_TAG" \
        .

    log "Image created: $IMAGE_NAME:$IMAGE_TAG"
}

# Print image info
print_info() {
    echo ""
    log "Image details:"
    docker images "$IMAGE_NAME:$IMAGE_TAG"

    echo ""
    echo "To run:"
    echo "  docker run -it $IMAGE_NAME:$IMAGE_TAG"
    echo ""
    echo "To use as base image:"
    echo "  FROM $IMAGE_NAME:$IMAGE_TAG"
}

# Main
main() {
    local method="${1:-rootfs}"

    case "$method" in
        rootfs)
            build_from_rootfs
            ;;
        dockerfile)
            build_from_dockerfile
            ;;
        *)
            echo "Usage: $0 [rootfs|dockerfile]"
            echo ""
            echo "Methods:"
            echo "  rootfs     - Build from pre-built root filesystem (default)"
            echo "  dockerfile - Build using multi-stage Dockerfile"
            exit 1
            ;;
    esac

    print_info
}

main "$@"
