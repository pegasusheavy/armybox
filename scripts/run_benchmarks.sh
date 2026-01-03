#!/bin/bash
# Run all benchmarks for armybox
#
# Usage:
#   ./scripts/run_benchmarks.sh           # Run all benchmarks
#   ./scripts/run_benchmarks.sh applet    # Run specific benchmark group
#   ./scripts/run_benchmarks.sh --quick   # Quick mode (fewer samples)

set -e

cd "$(dirname "$0")/.."

# Build release first
echo "Building release binary..."
cargo build --release

export ARMYBOX_PATH="./target/release/armybox"

BENCH_ARGS=""
FILTER=""

for arg in "$@"; do
    case $arg in
        --quick)
            BENCH_ARGS="$BENCH_ARGS --sample-size 10"
            ;;
        --save)
            BENCH_ARGS="$BENCH_ARGS --save-baseline current"
            ;;
        --compare)
            BENCH_ARGS="$BENCH_ARGS --baseline current"
            ;;
        *)
            FILTER="$arg"
            ;;
    esac
done

echo "Running benchmarks..."
echo "============================================"

if [ -z "$FILTER" ]; then
    # Run all benchmarks
    echo "Running applet benchmarks..."
    cargo bench --bench applet_benchmarks $BENCH_ARGS

    echo ""
    echo "Running text processing benchmarks..."
    cargo bench --bench text_benchmarks $BENCH_ARGS

    echo ""
    echo "Running compression benchmarks..."
    cargo bench --bench compression_benchmarks $BENCH_ARGS
else
    # Run filtered benchmarks
    echo "Running benchmarks matching: $FILTER"
    cargo bench $BENCH_ARGS -- "$FILTER"
fi

echo ""
echo "============================================"
echo "Benchmark results saved to target/criterion/"
echo "View HTML report at: target/criterion/report/index.html"
