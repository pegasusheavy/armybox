#!/bin/bash
# POSIX Compliance Test Suite for ArmyBox
#
# Tests utilities against POSIX.1-2017 specifications
# Reference: https://pubs.opengroup.org/onlinepubs/9699919799/
#
# Usage: ./run_tests.sh [path-to-armybox-binary]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Test counters
PASS=0
FAIL=0
SKIP=0

# Get armybox binary path
AB="${1:-$(dirname "$0")/../../target/release/armybox}"

if [[ ! -x "$AB" ]]; then
    echo "Error: armybox binary not found at $AB"
    echo "Usage: $0 [path-to-armybox-binary]"
    exit 1
fi

echo "Testing armybox at: $AB"
echo "Version: $($AB --version 2>&1 | head -1 || echo 'unknown')"
echo ""

# Create temp directory
TESTDIR=$(mktemp -d)
trap "rm -rf $TESTDIR" EXIT

cd "$TESTDIR"

# Test helper functions
pass() {
    echo -e "${GREEN}PASS${NC}: $1"
    PASS=$((PASS + 1))
}

fail() {
    echo -e "${RED}FAIL${NC}: $1"
    echo "       Expected: $2"
    echo "       Got:      $3"
    FAIL=$((FAIL + 1))
}

skip() {
    echo -e "${YELLOW}SKIP${NC}: $1 - $2"
    SKIP=$((SKIP + 1))
}

test_output() {
    local name="$1"
    local expected="$2"
    local actual="$3"

    if [[ "$expected" == "$actual" ]]; then
        pass "$name"
    else
        fail "$name" "$expected" "$actual"
    fi
}

test_exit_code() {
    local name="$1"
    local expected="$2"
    local actual="$3"

    if [[ "$expected" == "$actual" ]]; then
        pass "$name (exit code)"
    else
        fail "$name (exit code)" "$expected" "$actual"
    fi
}

echo "============================================"
echo "POSIX Compliance Test Suite"
echo "============================================"
echo ""

#############################################
# echo - POSIX.1-2017
#############################################
echo "--- echo ---"

test_output "echo basic" "hello" "$($AB echo hello)"
test_output "echo multiple args" "hello world" "$($AB echo hello world)"
test_output "echo -n (no newline)" "hello" "$($AB echo -n hello; echo)"
test_output "echo empty" "" "$($AB echo)"

#############################################
# true/false - POSIX.1-2017
#############################################
echo ""
echo "--- true/false ---"

$AB true
test_exit_code "true returns 0" "0" "$?"

exit_code=0
$AB false || exit_code=$?
test_exit_code "false returns 1" "1" "$exit_code"

#############################################
# cat - POSIX.1-2017
#############################################
echo ""
echo "--- cat ---"

echo "hello" > test.txt
test_output "cat file" "hello" "$($AB cat test.txt)"
test_output "cat stdin" "hello" "$(echo hello | $AB cat)"
echo -e "line1\nline2" > multi.txt
test_output "cat multiline" $'line1\nline2' "$($AB cat multi.txt)"

#############################################
# head - POSIX.1-2017
#############################################
echo ""
echo "--- head ---"

echo -e "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11" > nums.txt
test_output "head default (10 lines)" $'1\n2\n3\n4\n5\n6\n7\n8\n9\n10' "$($AB head nums.txt)"
test_output "head -n 3" $'1\n2\n3' "$($AB head -n 3 nums.txt)"
test_output "head -n 1" "1" "$($AB head -n 1 nums.txt)"

#############################################
# tail - POSIX.1-2017
#############################################
echo ""
echo "--- tail ---"

test_output "tail default (10 lines)" $'2\n3\n4\n5\n6\n7\n8\n9\n10\n11' "$($AB tail nums.txt)"
test_output "tail -n 3" $'9\n10\n11' "$($AB tail -n 3 nums.txt)"
test_output "tail -n 1" "11" "$($AB tail -n 1 nums.txt)"

#############################################
# wc - POSIX.1-2017
#############################################
echo ""
echo "--- wc ---"

echo -e "one two three\nfour five" > words.txt
# wc output format: lines words chars filename
result=$($AB wc -l words.txt | awk '{print $1}')
test_output "wc -l" "2" "$result"

result=$($AB wc -w words.txt | awk '{print $1}')
test_output "wc -w" "5" "$result"

#############################################
# grep - POSIX.1-2017
#############################################
echo ""
echo "--- grep ---"

echo -e "hello world\nfoo bar\nhello again" > grep.txt
test_output "grep basic" $'hello world\nhello again' "$($AB grep hello grep.txt)"
test_output "grep -c (count)" "2" "$($AB grep -c hello grep.txt)"
test_output "grep -n (line numbers)" $'1:hello world\n3:hello again' "$($AB grep -n hello grep.txt)"
test_output "grep -v (invert)" "foo bar" "$($AB grep -v hello grep.txt)"
test_output "grep -i (ignore case)" $'hello world\nhello again' "$($AB grep -i HELLO grep.txt)"

exit_code=0
$AB grep nomatch grep.txt || exit_code=$?
test_exit_code "grep no match returns 1" "1" "$exit_code"

#############################################
# sort - POSIX.1-2017
#############################################
echo ""
echo "--- sort ---"

echo -e "banana\napple\ncherry" > fruits.txt
test_output "sort basic" $'apple\nbanana\ncherry' "$($AB sort fruits.txt)"
test_output "sort -r (reverse)" $'cherry\nbanana\napple' "$($AB sort -r fruits.txt)"

echo -e "10\n2\n1\n20" > numbers.txt
test_output "sort -n (numeric)" $'1\n2\n10\n20' "$($AB sort -n numbers.txt)"

#############################################
# uniq - POSIX.1-2017
#############################################
echo ""
echo "--- uniq ---"

echo -e "a\na\nb\nb\nb\nc" > dups.txt
test_output "uniq basic" $'a\nb\nc' "$($AB uniq dups.txt)"
test_output "uniq -c (count)" $'2 a\n3 b\n1 c' "$($AB uniq -c dups.txt)"
test_output "uniq -d (duplicates only)" $'a\nb' "$($AB uniq -d dups.txt)"
test_output "uniq -u (unique only)" "c" "$($AB uniq -u dups.txt)"

#############################################
# cut - POSIX.1-2017
#############################################
echo ""
echo "--- cut ---"

echo -e "a:b:c\n1:2:3" > cut.txt
test_output "cut -d: -f1" $'a\n1' "$(echo -e 'a:b:c\n1:2:3' | $AB cut -d : -f 1)"
test_output "cut -d: -f2" $'b\n2' "$(echo -e 'a:b:c\n1:2:3' | $AB cut -d : -f 2)"

#############################################
# tr - POSIX.1-2017
#############################################
echo ""
echo "--- tr ---"

test_output "tr lowercase to uppercase" "HELLO" "$(echo hello | $AB tr a-z A-Z)"
test_output "tr delete chars" "hllo" "$(echo hello | $AB tr -d e)"

#############################################
# File utilities
#############################################
echo ""
echo "--- File utilities ---"

# ls
$AB ls . > /dev/null
test_exit_code "ls exits 0" "0" "$?"

# mkdir/rmdir
$AB mkdir testdir
test_exit_code "mkdir" "0" "$?"
[[ -d testdir ]] && pass "mkdir creates directory" || fail "mkdir creates directory" "directory exists" "directory missing"

$AB rmdir testdir
test_exit_code "rmdir" "0" "$?"
[[ ! -d testdir ]] && pass "rmdir removes directory" || fail "rmdir removes directory" "directory removed" "directory exists"

# touch
$AB touch newfile
[[ -f newfile ]] && pass "touch creates file" || fail "touch creates file" "file exists" "file missing"

# cp
echo "content" > source.txt
$AB cp source.txt dest.txt
[[ -f dest.txt ]] && pass "cp creates destination" || fail "cp creates destination" "file exists" "file missing"
test_output "cp preserves content" "content" "$($AB cat dest.txt)"

# mv
$AB mv dest.txt moved.txt
[[ -f moved.txt && ! -f dest.txt ]] && pass "mv moves file" || fail "mv moves file" "file moved" "file not moved"

# rm
$AB rm moved.txt
[[ ! -f moved.txt ]] && pass "rm removes file" || fail "rm removes file" "file removed" "file exists"

# ln -s
echo "link content" > linktest.txt
$AB ln -s linktest.txt symlink
[[ -L symlink ]] && pass "ln -s creates symlink" || fail "ln -s creates symlink" "symlink exists" "no symlink"
test_output "symlink reads correctly" "link content" "$($AB cat symlink)"

# chmod
$AB chmod 755 source.txt
test_exit_code "chmod" "0" "$?"

#############################################
# pwd
#############################################
echo ""
echo "--- pwd ---"

result=$($AB pwd)
[[ "$result" == "$TESTDIR" ]] && pass "pwd returns current directory" || fail "pwd returns current directory" "$TESTDIR" "$result"

#############################################
# basename/dirname
#############################################
echo ""
echo "--- basename/dirname ---"

test_output "basename" "file.txt" "$($AB basename /path/to/file.txt)"
test_output "basename with suffix" "file" "$($AB basename /path/to/file.txt .txt)"
test_output "dirname" "/path/to" "$($AB dirname /path/to/file.txt)"

#############################################
# Summary
#############################################
echo ""
echo "============================================"
echo "Test Summary"
echo "============================================"
echo -e "${GREEN}PASSED${NC}: $PASS"
echo -e "${RED}FAILED${NC}: $FAIL"
echo -e "${YELLOW}SKIPPED${NC}: $SKIP"
echo ""

if [[ $FAIL -gt 0 ]]; then
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi
