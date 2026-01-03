#!/bin/bash
# Build the smallest possible armybox binary
# Usage: ./scripts/build-small.sh [target]
#
# Options:
#   target: glibc (default), musl, musl-upx (smallest)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

TARGET="${1:-glibc}"

echo "=== Building Armybox (smallest binary) ==="
echo "Target: $TARGET"
echo ""

case "$TARGET" in
    glibc)
        echo "Building for glibc (dynamically linked)..."
        RUSTFLAGS="-C link-arg=-Wl,--gc-sections -C link-arg=-Wl,--as-needed" \
            cargo build --release

        OUTPUT="target/release/armybox"
        ;;

    musl)
        echo "Building for musl (statically linked)..."
        rustup target add x86_64-unknown-linux-musl 2>/dev/null || true

        RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -C link-arg=-Wl,--gc-sections -C link-arg=-Wl,--as-needed -C link-arg=-Wl,-s" \
            cargo build --release --target x86_64-unknown-linux-musl

        OUTPUT="target/x86_64-unknown-linux-musl/release/armybox"
        ;;

    musl-upx)
        echo "Building for musl with UPX compression (smallest)..."
        rustup target add x86_64-unknown-linux-musl 2>/dev/null || true

        # Check for UPX
        if ! command -v upx &> /dev/null; then
            echo "Error: UPX not found. Install with: apt install upx-ucl"
            exit 1
        fi

        RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes -C link-arg=-Wl,--gc-sections -C link-arg=-Wl,--as-needed -C link-arg=-Wl,-s" \
            cargo build --release --target x86_64-unknown-linux-musl

        echo ""
        echo "Compressing with UPX..."
        upx --best --lzma target/x86_64-unknown-linux-musl/release/armybox \
            -o target/x86_64-unknown-linux-musl/release/armybox-upx

        OUTPUT="target/x86_64-unknown-linux-musl/release/armybox-upx"
        ;;

    *)
        echo "Unknown target: $TARGET"
        echo "Usage: $0 [glibc|musl|musl-upx]"
        exit 1
        ;;
esac

echo ""
echo "=== Build Complete ==="
echo "Binary: $OUTPUT"
ls -lh "$OUTPUT"
echo ""
echo "Applets: $("$OUTPUT" --list | wc -l)"
echo ""

if [ "$TARGET" == "musl" ] || [ "$TARGET" == "musl-upx" ]; then
    echo "Binary type:"
    file "$OUTPUT"
fi
