#!/bin/bash
# Run Rust-based POSIX compliance tests
#
# This script builds the armybox binary first, then runs the integration tests.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building armybox release binary..."
RUSTFLAGS="-C linker=gcc -C link-arg=-lc" cargo build --release --manifest-path="$PROJECT_DIR/Cargo.toml"

echo ""
echo "Running POSIX compliance tests..."
ARMYBOX_PATH="$PROJECT_DIR/target/release/armybox" cargo test --manifest-path="$PROJECT_DIR/Cargo.toml" --test posix_tests --no-fail-fast -- "$@"
