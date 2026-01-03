#!/bin/bash
# benchmark-compare.sh - Compare armybox vs BusyBox performance
#
# This script runs various benchmarks and compares armybox against
# BusyBox and system utilities.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ARMYBOX="$PROJECT_DIR/target/release/armybox"
BUSYBOX="/usr/bin/busybox"
RESULTS_DIR="$PROJECT_DIR/benchmark-results"
ITERATIONS=100

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; }

# Benchmark function - returns time in milliseconds
benchmark() {
    local cmd="$1"
    local iterations="${2:-$ITERATIONS}"
    local start end elapsed

    # Warmup
    for i in $(seq 1 3); do
        eval "$cmd" >/dev/null 2>&1 || true
    done

    start=$(date +%s%N)
    for i in $(seq 1 "$iterations"); do
        eval "$cmd" >/dev/null 2>&1 || true
    done
    end=$(date +%s%N)

    elapsed=$(( (end - start) / 1000000 ))
    echo "$elapsed"
}

# Compare and report
compare() {
    local name="$1"
    local armybox_time="$2"
    local busybox_time="$3"

    if [ "$busybox_time" -eq 0 ]; then
        busybox_time=1
    fi

    local ratio=$(echo "scale=2; $armybox_time / $busybox_time" | bc)
    local speedup=$(echo "scale=2; $busybox_time / $armybox_time" | bc 2>/dev/null || echo "0")

    if (( $(echo "$ratio < 1.0" | bc -l) )); then
        log_success "$name: armybox ${armybox_time}ms vs busybox ${busybox_time}ms (${speedup}x faster)"
    elif (( $(echo "$ratio < 1.2" | bc -l) )); then
        echo -e "${YELLOW}[WARN]${NC} $name: armybox ${armybox_time}ms vs busybox ${busybox_time}ms (similar)"
    else
        log_fail "$name: armybox ${armybox_time}ms vs busybox ${busybox_time}ms (${ratio}x slower)"
    fi

    echo "$name,$armybox_time,$busybox_time,$ratio" >> "$RESULTS_DIR/comparison.csv"
}

# Create test files
setup_test_files() {
    log_info "Creating test files..."

    # Create various sized files
    dd if=/dev/urandom of="$RESULTS_DIR/small.bin" bs=1K count=10 2>/dev/null
    dd if=/dev/urandom of="$RESULTS_DIR/medium.bin" bs=1M count=10 2>/dev/null
    dd if=/dev/urandom of="$RESULTS_DIR/large.bin" bs=1M count=100 2>/dev/null

    # Create text file with lines
    seq 1 100000 > "$RESULTS_DIR/numbers.txt"
    yes "hello world this is a test line for grep and text processing" | head -n 100000 > "$RESULTS_DIR/text.txt"

    # Create directory structure
    mkdir -p "$RESULTS_DIR/testdir"
    for i in $(seq 1 1000); do
        echo "file $i content" > "$RESULTS_DIR/testdir/file_$i.txt"
    done
}

cleanup_test_files() {
    rm -rf "$RESULTS_DIR/small.bin" "$RESULTS_DIR/medium.bin" "$RESULTS_DIR/large.bin"
    rm -rf "$RESULTS_DIR/numbers.txt" "$RESULTS_DIR/text.txt" "$RESULTS_DIR/testdir"
}

run_benchmarks() {
    echo "name,armybox_ms,busybox_ms,ratio" > "$RESULTS_DIR/comparison.csv"

    echo ""
    echo "========================================"
    echo "armybox vs BusyBox Performance Comparison"
    echo "========================================"
    echo "Iterations per test: $ITERATIONS"
    echo ""

    # ========================================
    # Startup / Dispatch Overhead
    # ========================================
    echo ""
    log_info "=== Startup Overhead ==="

    ab_time=$(benchmark "$ARMYBOX true" 1000)
    bb_time=$(benchmark "$BUSYBOX true" 1000)
    compare "true (1000 iter)" "$ab_time" "$bb_time"

    ab_time=$(benchmark "$ARMYBOX echo hello" 1000)
    bb_time=$(benchmark "$BUSYBOX echo hello" 1000)
    compare "echo hello (1000 iter)" "$ab_time" "$bb_time"

    # ========================================
    # Text Processing
    # ========================================
    echo ""
    log_info "=== Text Processing ==="

    # cat
    ab_time=$(benchmark "$ARMYBOX cat $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX cat $RESULTS_DIR/text.txt")
    compare "cat 100K lines" "$ab_time" "$bb_time"

    # head
    ab_time=$(benchmark "$ARMYBOX head -n 1000 $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX head -n 1000 $RESULTS_DIR/text.txt")
    compare "head -n 1000" "$ab_time" "$bb_time"

    # tail
    ab_time=$(benchmark "$ARMYBOX tail -n 1000 $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX tail -n 1000 $RESULTS_DIR/text.txt")
    compare "tail -n 1000" "$ab_time" "$bb_time"

    # wc
    ab_time=$(benchmark "$ARMYBOX wc $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX wc $RESULTS_DIR/text.txt")
    compare "wc 100K lines" "$ab_time" "$bb_time"

    # grep literal
    ab_time=$(benchmark "$ARMYBOX grep 'hello' $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX grep 'hello' $RESULTS_DIR/text.txt")
    compare "grep literal" "$ab_time" "$bb_time"

    # grep regex
    ab_time=$(benchmark "$ARMYBOX grep -E 'hel+o' $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX grep -E 'hel+o' $RESULTS_DIR/text.txt")
    compare "grep regex" "$ab_time" "$bb_time"

    # sort
    ab_time=$(benchmark "$ARMYBOX sort $RESULTS_DIR/numbers.txt" 10)
    bb_time=$(benchmark "$BUSYBOX sort $RESULTS_DIR/numbers.txt" 10)
    compare "sort 100K numbers (10 iter)" "$ab_time" "$bb_time"

    # uniq
    ab_time=$(benchmark "$ARMYBOX uniq $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX uniq $RESULTS_DIR/text.txt")
    compare "uniq 100K lines" "$ab_time" "$bb_time"

    # ========================================
    # File Operations
    # ========================================
    echo ""
    log_info "=== File Operations ==="

    # ls
    ab_time=$(benchmark "$ARMYBOX ls $RESULTS_DIR/testdir")
    bb_time=$(benchmark "$BUSYBOX ls $RESULTS_DIR/testdir")
    compare "ls 1000 files" "$ab_time" "$bb_time"

    # ls -la
    ab_time=$(benchmark "$ARMYBOX ls -la $RESULTS_DIR/testdir")
    bb_time=$(benchmark "$BUSYBOX ls -la $RESULTS_DIR/testdir")
    compare "ls -la 1000 files" "$ab_time" "$bb_time"

    # find
    ab_time=$(benchmark "$ARMYBOX find $RESULTS_DIR/testdir -name '*.txt'" 10)
    bb_time=$(benchmark "$BUSYBOX find $RESULTS_DIR/testdir -name '*.txt'" 10)
    compare "find 1000 files (10 iter)" "$ab_time" "$bb_time"

    # stat
    ab_time=$(benchmark "$ARMYBOX stat $RESULTS_DIR/text.txt")
    bb_time=$(benchmark "$BUSYBOX stat $RESULTS_DIR/text.txt")
    compare "stat" "$ab_time" "$bb_time"

    # ========================================
    # Checksums
    # ========================================
    echo ""
    log_info "=== Checksums ==="

    ab_time=$(benchmark "$ARMYBOX md5sum $RESULTS_DIR/medium.bin" 10)
    bb_time=$(benchmark "$BUSYBOX md5sum $RESULTS_DIR/medium.bin" 10)
    compare "md5sum 10MB (10 iter)" "$ab_time" "$bb_time"

    ab_time=$(benchmark "$ARMYBOX sha256sum $RESULTS_DIR/medium.bin" 10)
    bb_time=$(benchmark "$BUSYBOX sha256sum $RESULTS_DIR/medium.bin" 10)
    compare "sha256sum 10MB (10 iter)" "$ab_time" "$bb_time"

    # ========================================
    # Compression
    # ========================================
    echo ""
    log_info "=== Compression ==="

    # gzip compression
    ab_time=$(benchmark "cat $RESULTS_DIR/text.txt | $ARMYBOX gzip -c > /dev/null" 10)
    bb_time=$(benchmark "cat $RESULTS_DIR/text.txt | $BUSYBOX gzip -c > /dev/null" 10)
    compare "gzip compress (10 iter)" "$ab_time" "$bb_time"

    # Create compressed file for decompression test
    gzip -c "$RESULTS_DIR/text.txt" > "$RESULTS_DIR/text.txt.gz"

    ab_time=$(benchmark "$ARMYBOX gunzip -c $RESULTS_DIR/text.txt.gz > /dev/null" 10)
    bb_time=$(benchmark "$BUSYBOX gunzip -c $RESULTS_DIR/text.txt.gz > /dev/null" 10)
    compare "gunzip decompress (10 iter)" "$ab_time" "$bb_time"

    rm -f "$RESULTS_DIR/text.txt.gz"

    # ========================================
    # Encoding
    # ========================================
    echo ""
    log_info "=== Encoding ==="

    ab_time=$(benchmark "$ARMYBOX base64 $RESULTS_DIR/small.bin" 100)
    bb_time=$(benchmark "$BUSYBOX base64 $RESULTS_DIR/small.bin" 100)
    compare "base64 encode 10KB" "$ab_time" "$bb_time"

    # ========================================
    # System Info
    # ========================================
    echo ""
    log_info "=== System Info ==="

    ab_time=$(benchmark "$ARMYBOX uname -a" 1000)
    bb_time=$(benchmark "$BUSYBOX uname -a" 1000)
    compare "uname -a (1000 iter)" "$ab_time" "$bb_time"

    ab_time=$(benchmark "$ARMYBOX whoami" 1000)
    bb_time=$(benchmark "$BUSYBOX whoami" 1000)
    compare "whoami (1000 iter)" "$ab_time" "$bb_time"

    ab_time=$(benchmark "$ARMYBOX date" 1000)
    bb_time=$(benchmark "$BUSYBOX date" 1000)
    compare "date (1000 iter)" "$ab_time" "$bb_time"

    # ========================================
    # Summary
    # ========================================
    echo ""
    echo "========================================"
    echo "Summary"
    echo "========================================"

    total_ab=0
    total_bb=0
    count=0
    while IFS=',' read -r name ab bb ratio; do
        if [ "$name" != "name" ]; then
            total_ab=$((total_ab + ab))
            total_bb=$((total_bb + bb))
            count=$((count + 1))
        fi
    done < "$RESULTS_DIR/comparison.csv"

    if [ $total_bb -gt 0 ]; then
        overall_ratio=$(echo "scale=2; $total_ab / $total_bb" | bc)
        echo "Total armybox time: ${total_ab}ms"
        echo "Total BusyBox time: ${total_bb}ms"
        echo "Overall ratio: ${overall_ratio}x"

        if (( $(echo "$overall_ratio < 1.0" | bc -l) )); then
            echo -e "${GREEN}armybox is faster overall!${NC}"
        else
            echo -e "${YELLOW}armybox needs optimization${NC}"
        fi
    fi

    echo ""
    echo "Detailed results saved to: $RESULTS_DIR/comparison.csv"
}

main() {
    if [ ! -f "$ARMYBOX" ]; then
        log_info "Building armybox..."
        cd "$PROJECT_DIR" && cargo build --release
    fi

    setup_test_files
    run_benchmarks
    cleanup_test_files
}

main "$@"
