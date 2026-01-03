#!/bin/bash
# Build release binary first
echo "Building armybox release binary..."
cargo build --release 2>&1 | tail -5

# Get binary size
SIZE=$(ls -la target/release/armybox | awk '{print $5}')
echo "Binary size: $SIZE bytes ($(echo "scale=0; $SIZE/1024" | bc) KB)"

# Create temp directory for benchmarks
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Generate test data
echo "Generating test data..."
for size in 1000 10000 100000 1000000; do
    yes "The quick brown fox jumps over the lazy dog" | head -n $size > "$TEMP_DIR/lines_$size.txt"
done

dd if=/dev/urandom of="$TEMP_DIR/random_1mb.bin" bs=1M count=1 2>/dev/null

ARMYBOX="./target/release/armybox"
BUSYBOX=$(which busybox 2>/dev/null || echo "")

# Benchmark function with comparison
benchmark() {
    local name=$1
    local cmd=$2
    local iterations=${3:-100}
    
    echo -n "  $name: "
    
    # Run armybox benchmark
    start=$(date +%s.%N)
    for i in $(seq 1 $iterations); do
        eval "$ARMYBOX $cmd" > /dev/null 2>&1
    done
    end=$(date +%s.%N)
    armybox_time=$(echo "$end - $start" | bc)
    armybox_avg=$(echo "scale=3; $armybox_time * 1000 / $iterations" | bc)
    
    echo -n "armybox=${armybox_avg}ms"
    
    # Run busybox benchmark if available
    if [ -n "$BUSYBOX" ]; then
        start=$(date +%s.%N)
        for i in $(seq 1 $iterations); do
            eval "$BUSYBOX $cmd" > /dev/null 2>&1
        done
        end=$(date +%s.%N)
        busybox_time=$(echo "$end - $start" | bc)
        busybox_avg=$(echo "scale=3; $busybox_time * 1000 / $iterations" | bc)
        
        ratio=$(echo "scale=2; $busybox_avg / $armybox_avg" | bc)
        echo -n ", busybox=${busybox_avg}ms, ratio=${ratio}x"
    fi
    
    echo ""
}

echo ""
echo "=== Dispatch Overhead ==="
benchmark "true" "true" 1000
benchmark "false" "false" 1000
benchmark "echo hello" "echo hello" 1000

echo ""
echo "=== Text Processing (100K lines) ==="
benchmark "wc" "wc $TEMP_DIR/lines_100000.txt" 50
benchmark "grep (literal)" "grep fox $TEMP_DIR/lines_100000.txt" 50
benchmark "grep -i" "grep -i FOX $TEMP_DIR/lines_100000.txt" 50
benchmark "grep -v" "grep -v fox $TEMP_DIR/lines_100000.txt" 50
benchmark "head -n 1000" "head -n 1000 $TEMP_DIR/lines_100000.txt" 100
benchmark "tail -n 1000" "tail -n 1000 $TEMP_DIR/lines_100000.txt" 100
benchmark "sort" "sort $TEMP_DIR/lines_10000.txt" 20
benchmark "uniq" "uniq $TEMP_DIR/lines_10000.txt" 50
benchmark "cut -d' ' -f1" "cut -d' ' -f1 $TEMP_DIR/lines_100000.txt" 50
benchmark "tr a-z A-Z" "tr a-z A-Z < $TEMP_DIR/lines_10000.txt" 50
benchmark "sed s/fox/cat/g" "sed s/fox/cat/g $TEMP_DIR/lines_10000.txt" 50

echo ""
echo "=== File Operations ==="
benchmark "cat (100K lines)" "cat $TEMP_DIR/lines_100000.txt" 50
benchmark "ls -la" "ls -la $TEMP_DIR" 200
benchmark "basename" "basename /path/to/some/file.txt" 500
benchmark "dirname" "dirname /path/to/some/file.txt" 500

echo ""
echo "=== Checksums (1MB) ==="
benchmark "md5sum" "md5sum $TEMP_DIR/random_1mb.bin" 20

echo ""
echo "=== Summary ==="
echo "Benchmarks complete. Binary size: $(echo "scale=0; $SIZE/1024" | bc) KB"
echo "Applet count: $($ARMYBOX --list | wc -l)"
