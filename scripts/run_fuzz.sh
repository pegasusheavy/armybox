#!/bin/bash
# Run fuzzing for armybox
#
# Usage:
#   ./scripts/run_fuzz.sh fuzz_grep_pattern    # Run specific fuzz target
#   ./scripts/run_fuzz.sh --list              # List available targets
#   ./scripts/run_fuzz.sh --all               # Run all targets sequentially

set -e

cd "$(dirname "$0")/.."

# Check if cargo-fuzz is installed
if ! cargo fuzz --help &>/dev/null; then
    echo "cargo-fuzz not installed. Installing..."
    cargo install cargo-fuzz
fi

# Build release binary first (fuzz targets need it)
echo "Building release binary..."
cargo build --release

case "$1" in
    --list)
        echo "Available fuzz targets:"
        cargo fuzz list
        ;;
    --all)
        echo "Running all fuzz targets (1 minute each)..."
        for target in $(cargo fuzz list); do
            echo ""
            echo "=== Fuzzing: $target ==="
            cargo fuzz run "$target" -- -max_total_time=60 || true
        done
        ;;
    *)
        if [ -z "$1" ]; then
            echo "Usage: $0 <target> [fuzz_args...]"
            echo ""
            echo "Available targets:"
            cargo fuzz list
            echo ""
            echo "Examples:"
            echo "  $0 fuzz_grep_pattern"
            echo "  $0 fuzz_compression -- -max_total_time=300"
            echo "  $0 --all"
            exit 1
        fi

        TARGET=$1
        shift
        echo "Running fuzzer: $TARGET"
        cargo fuzz run "$TARGET" "$@"
        ;;
esac
