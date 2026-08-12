//! expr - evaluate expressions
//!
//! Evaluate an expression given as separate argv tokens and print the result.
//! Implements a recursive-descent parser over the tokens honoring POSIX
//! precedence, plus the common `length`, `substr`, `index` and `match`
//! string functions and a minimal BRE engine for the `:` / `match` operators.

extern crate alloc;

use alloc::vec::Vec;

use crate::io;
use crate::sys;
use super::get_arg;

/// A value in `expr` is always a string; numeric operators parse it as i64.
type Value = Vec<u8>;

/// Error exit code produced when the expression cannot be evaluated.
const E_SYNTAX: i32 = 2;
const E_MATH: i32 = 2;

/// A value is "false" (null or zero) when it is the empty string or when it
/// parses as an integer equal to zero.
fn is_falsey(v: &[u8]) -> bool {
    if v.is_empty() {
        return true;
    }
    matches!(sys::parse_i64(v), Some(0))
}

/// Format a signed integer into a freshly allocated byte vector.
fn int_to_value(n: i64) -> Value {
    let mut buf = [0u8; 24];
    let neg = n < 0;
    // Use u64 to safely handle i64::MIN.
    let mut m: u64 = if neg { (n as i128).unsigned_abs() as u64 } else { n as u64 };
    let mut i = buf.len();
    if m == 0 {
        i -= 1;
        buf[i] = b'0';
    } else {
        while m > 0 {
            i -= 1;
            buf[i] = b'0' + (m % 10) as u8;
            m /= 10;
        }
    }
    if neg {
        i -= 1;
        buf[i] = b'-';
    }
    buf[i..].to_vec()
}

/// Parse a value as an integer, requiring the entire string to be a valid
/// optionally-signed decimal integer.
fn to_int(v: &[u8]) -> Option<i64> {
    sys::parse_i64(v)
}

// ------------------------------------------------------------------------
// Recursive-descent parser
// ------------------------------------------------------------------------

struct Parser<'a> {
    toks: &'a [&'a [u8]],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a [u8]> {
        self.toks.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<&'a [u8]> {
        let t = self.toks.get(self.pos).copied();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn remaining(&self) -> usize {
        self.toks.len().saturating_sub(self.pos)
    }

    /// Lowest precedence: `|`.
    fn parse_or(&mut self) -> Result<Value, i32> {
        let mut left = self.parse_and()?;
        while let Some(t) = self.peek() {
            if t == b"|" {
                self.advance();
                let right = self.parse_and()?;
                // `|`: left if it is neither null nor zero, otherwise right.
                if !is_falsey(&left) {
                    // keep left
                } else {
                    left = right;
                }
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// `&`.
    fn parse_and(&mut self) -> Result<Value, i32> {
        let mut left = self.parse_rel()?;
        while let Some(t) = self.peek() {
            if t == b"&" {
                self.advance();
                let right = self.parse_rel()?;
                // `&`: left if both operands are neither null nor zero, else 0.
                if !is_falsey(&left) && !is_falsey(&right) {
                    // keep left
                } else {
                    left = b"0".to_vec();
                }
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Relational operators: `= > >= < <= !=`.
    fn parse_rel(&mut self) -> Result<Value, i32> {
        let mut left = self.parse_add()?;
        while let Some(t) = self.peek() {
            let op: &[u8] = match t {
                b"=" | b">" | b">=" | b"<" | b"<=" | b"!=" => t,
                _ => break,
            };
            self.advance();
            let right = self.parse_add()?;
            let res = compare(&left, &right, op);
            left = int_to_value(if res { 1 } else { 0 });
        }
        Ok(left)
    }

    /// Additive operators: `+ -`.
    fn parse_add(&mut self) -> Result<Value, i32> {
        let mut left = self.parse_mul()?;
        while let Some(t) = self.peek() {
            let sub = match t {
                b"+" => false,
                b"-" => true,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul()?;
            let a = to_int(&left).ok_or(E_MATH)?;
            let b = to_int(&right).ok_or(E_MATH)?;
            let r = if sub { a.wrapping_sub(b) } else { a.wrapping_add(b) };
            left = int_to_value(r);
        }
        Ok(left)
    }

    /// Multiplicative operators: `* / %`.
    fn parse_mul(&mut self) -> Result<Value, i32> {
        let mut left = self.parse_match()?;
        while let Some(t) = self.peek() {
            let kind = match t {
                b"*" => 0,
                b"/" => 1,
                b"%" => 2,
                _ => break,
            };
            self.advance();
            let right = self.parse_match()?;
            let a = to_int(&left).ok_or(E_MATH)?;
            let b = to_int(&right).ok_or(E_MATH)?;
            let r = match kind {
                0 => a.wrapping_mul(b),
                _ => {
                    if b == 0 {
                        io::write_str(2, b"expr: division by zero\n");
                        return Err(E_MATH);
                    }
                    if kind == 1 { a.wrapping_div(b) } else { a.wrapping_rem(b) }
                }
            };
            left = int_to_value(r);
        }
        Ok(left)
    }

    /// Anchored regex match: `:`.
    fn parse_match(&mut self) -> Result<Value, i32> {
        let mut left = self.parse_primary()?;
        while let Some(t) = self.peek() {
            if t == b":" {
                self.advance();
                let right = self.parse_primary()?;
                left = regex_match(&left, &right);
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parentheses, string functions, and bare operands.
    fn parse_primary(&mut self) -> Result<Value, i32> {
        let t = self.peek().ok_or(E_SYNTAX)?;

        if t == b"(" {
            self.advance();
            let v = self.parse_or()?;
            match self.advance() {
                Some(b")") => return Ok(v),
                _ => return Err(E_SYNTAX),
            }
        }

        // String functions (GNU extensions). Only treated as functions when
        // enough operands follow; otherwise the keyword is a literal string.
        match t {
            b"length" if self.remaining() >= 2 => {
                self.advance();
                let s = self.parse_primary()?;
                return Ok(int_to_value(s.len() as i64));
            }
            b"substr" if self.remaining() >= 4 => {
                self.advance();
                let s = self.parse_primary()?;
                let pos = self.parse_primary()?;
                let len = self.parse_primary()?;
                return Ok(substr(&s, &pos, &len));
            }
            b"index" if self.remaining() >= 3 => {
                self.advance();
                let s = self.parse_primary()?;
                let chars = self.parse_primary()?;
                return Ok(int_to_value(index_of(&s, &chars)));
            }
            b"match" if self.remaining() >= 3 => {
                self.advance();
                let s = self.parse_primary()?;
                let re = self.parse_primary()?;
                return Ok(regex_match(&s, &re));
            }
            _ => {}
        }

        // Bare operand.
        let tok = self.advance().ok_or(E_SYNTAX)?;
        Ok(tok.to_vec())
    }
}

/// Compare two values with the given relational operator. Comparison is numeric
/// when both operands are integers, otherwise lexicographic on bytes.
fn compare(a: &[u8], b: &[u8], op: &[u8]) -> bool {
    let ord = match (to_int(a), to_int(b)) {
        (Some(x), Some(y)) => x.cmp(&y),
        _ => a.cmp(b),
    };
    use core::cmp::Ordering::*;
    match op {
        b"=" => ord == Equal,
        b"!=" => ord != Equal,
        b">" => ord == Greater,
        b">=" => ord != Less,
        b"<" => ord == Less,
        b"<=" => ord != Greater,
        _ => false,
    }
}

/// `substr STR POS LEN`: 1-based substring. Out-of-range yields empty string.
fn substr(s: &[u8], pos: &[u8], len: &[u8]) -> Value {
    let p = to_int(pos).unwrap_or(0);
    let l = to_int(len).unwrap_or(0);
    if p < 1 || l < 1 {
        return Vec::new();
    }
    let start = (p - 1) as usize;
    if start >= s.len() {
        return Vec::new();
    }
    let end = start.saturating_add(l as usize).min(s.len());
    s[start..end].to_vec()
}

/// `index STR CHARS`: 1-based position of the first char of STR present in
/// CHARS, or 0 if none.
fn index_of(s: &[u8], chars: &[u8]) -> i64 {
    for (i, c) in s.iter().enumerate() {
        if chars.contains(c) {
            return (i + 1) as i64;
        }
    }
    0
}

// ------------------------------------------------------------------------
// Minimal BRE engine for `:` and `match`
// ------------------------------------------------------------------------

/// A single matchable atom.
enum AtomKind {
    Char(u8),
    Any,
    Class { negate: bool, ranges: Vec<(u8, u8)> },
    GroupStart,
    GroupEnd,
    End, // `$`
}

struct Piece {
    kind: AtomKind,
    star: bool,
}

/// Parse a BRE pattern into pieces. A leading `^` is treated as an anchor
/// (dropped) because `expr` always anchors at the start of the string.
fn compile(pattern: &[u8]) -> (Vec<Piece>, bool) {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut has_group = false;
    let mut i = 0;
    if pattern.first() == Some(&b'^') {
        i = 1;
    }
    while i < pattern.len() {
        let c = pattern[i];
        let kind: AtomKind = match c {
            b'\\' => {
                i += 1;
                if i >= pattern.len() {
                    AtomKind::Char(b'\\')
                } else {
                    match pattern[i] {
                        b'(' => {
                            has_group = true;
                            AtomKind::GroupStart
                        }
                        b')' => AtomKind::GroupEnd,
                        other => AtomKind::Char(other),
                    }
                }
            }
            b'.' => AtomKind::Any,
            b'$' if i == pattern.len() - 1 => AtomKind::End,
            b'[' => {
                let (cls, next) = parse_class(pattern, i);
                i = next;
                // `i` now points at the closing `]`; fall through to *-check.
                pieces.push(Piece { kind: cls, star: false });
                // Handle a possible trailing `*`.
                if i + 1 < pattern.len() && pattern[i + 1] == b'*' {
                    if let Some(last) = pieces.last_mut() {
                        last.star = true;
                    }
                    i += 1;
                }
                i += 1;
                continue;
            }
            other => AtomKind::Char(other),
        };

        // GroupStart/GroupEnd/End never take a `*`.
        let can_star = !matches!(kind, AtomKind::GroupStart | AtomKind::GroupEnd | AtomKind::End);
        let star = can_star && i + 1 < pattern.len() && pattern[i + 1] == b'*';
        pieces.push(Piece { kind, star });
        if star {
            i += 1;
        }
        i += 1;
    }
    (pieces, has_group)
}

/// Parse a `[...]` character class starting at index `start` (the `[`).
/// Returns the atom and the index of the closing `]`.
fn parse_class(pattern: &[u8], start: usize) -> (AtomKind, usize) {
    let mut i = start + 1;
    let mut negate = false;
    if i < pattern.len() && pattern[i] == b'^' {
        negate = true;
        i += 1;
    }
    let mut ranges: Vec<(u8, u8)> = Vec::new();
    // A `]` immediately after `[` (or `[^`) is a literal.
    if i < pattern.len() && pattern[i] == b']' {
        ranges.push((b']', b']'));
        i += 1;
    }
    while i < pattern.len() && pattern[i] != b']' {
        let lo = pattern[i];
        if i + 2 < pattern.len() && pattern[i + 1] == b'-' && pattern[i + 2] != b']' {
            let hi = pattern[i + 2];
            ranges.push((lo, hi));
            i += 3;
        } else {
            ranges.push((lo, lo));
            i += 1;
        }
    }
    // `i` is the index of `]` (or end of pattern if unterminated).
    (AtomKind::Class { negate, ranges }, i)
}

fn class_matches(negate: bool, ranges: &[(u8, u8)], c: u8) -> bool {
    let mut hit = false;
    for &(lo, hi) in ranges {
        if c >= lo && c <= hi {
            hit = true;
            break;
        }
    }
    hit != negate
}

fn atom_matches(kind: &AtomKind, c: u8) -> bool {
    match kind {
        AtomKind::Char(x) => *x == c,
        AtomKind::Any => true,
        AtomKind::Class { negate, ranges } => class_matches(*negate, ranges, c),
        _ => false,
    }
}

struct Capture {
    start: Option<usize>,
    end: Option<usize>,
}

/// Match pieces[pi..] against s[si..], recording capture boundaries as markers
/// are crossed on the successful path. Returns the end offset on full match.
fn match_here(pieces: &[Piece], pi: usize, s: &[u8], si: usize, cap: &mut Capture) -> Option<usize> {
    if pi >= pieces.len() {
        return Some(si);
    }
    let piece = &pieces[pi];
    match &piece.kind {
        AtomKind::GroupStart => {
            cap.start = Some(si);
            return match_here(pieces, pi + 1, s, si, cap);
        }
        AtomKind::GroupEnd => {
            cap.end = Some(si);
            return match_here(pieces, pi + 1, s, si, cap);
        }
        AtomKind::End => {
            if si == s.len() {
                return match_here(pieces, pi + 1, s, si, cap);
            }
            return None;
        }
        _ => {}
    }

    if piece.star {
        // Greedy: consume as many as possible, then backtrack.
        let mut count = 0usize;
        while si + count < s.len() && atom_matches(&piece.kind, s[si + count]) {
            count += 1;
        }
        loop {
            if let Some(r) = match_here(pieces, pi + 1, s, si + count, cap) {
                return Some(r);
            }
            if count == 0 {
                return None;
            }
            count -= 1;
        }
    } else {
        if si < s.len() && atom_matches(&piece.kind, s[si]) {
            return match_here(pieces, pi + 1, s, si + 1, cap);
        }
        None
    }
}

/// Anchored regex match used by both `:` and `match`.
///
/// Without a `\(...\)` group, returns the byte-length of the matched prefix as
/// a decimal string ("0" if no match). With a group, returns the captured
/// substring ("" if no match).
fn regex_match(s: &[u8], pattern: &[u8]) -> Value {
    let (pieces, has_group) = compile(pattern);
    let mut cap = Capture { start: None, end: None };
    let matched_end = match_here(&pieces, 0, s, 0, &mut cap);

    if has_group {
        match (matched_end, cap.start, cap.end) {
            (Some(_), Some(a), Some(b)) if a <= b && b <= s.len() => s[a..b].to_vec(),
            _ => Vec::new(),
        }
    } else {
        match matched_end {
            Some(end) => int_to_value(end as i64),
            None => b"0".to_vec(),
        }
    }
}

// ------------------------------------------------------------------------
// Entry point
// ------------------------------------------------------------------------

/// expr - evaluate expressions
///
/// # Synopsis
/// ```text
/// expr EXPRESSION
/// ```
///
/// # Exit Status
/// - 0: result is neither null nor zero
/// - 1: result is null or zero
/// - 2: invalid expression or evaluation error
pub fn expr(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"expr: missing operand\n");
        return E_SYNTAX;
    }

    // Collect tokens (argv[1..argc]).
    let mut toks: Vec<&[u8]> = Vec::new();
    for i in 1..argc {
        match unsafe { get_arg(argv, i) } {
            Some(t) => toks.push(t),
            None => break,
        }
    }

    let mut parser = Parser { toks: &toks, pos: 0 };
    match parser.parse_or() {
        Ok(v) => {
            if parser.pos != parser.toks.len() {
                io::write_str(2, b"expr: syntax error\n");
                return E_SYNTAX;
            }
            io::write_all(1, &v);
            io::write_str(1, b"\n");
            if is_falsey(&v) { 1 } else { 0 }
        }
        Err(code) => code,
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::Command;
    use std::path::PathBuf;

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() { return release; }
        manifest_dir.join("target/debug/armybox")
    }

    #[test]
    fn test_expr_addition() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "5", "+", "3"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "8");
    }

    #[test]
    fn test_expr_subtraction() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "10", "-", "3"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "7");
    }

    #[test]
    fn test_expr_multiplication() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "6", "*", "7"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "42");
    }

    #[test]
    fn test_expr_division() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "20", "/", "4"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "5");
    }

    #[test]
    fn test_expr_single_value() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let output = Command::new(&armybox)
            .args(["expr", "42"])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "42");
    }
}
