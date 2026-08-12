//! grep - search for patterns in files
//!
//! POSIX.1-2017 compliant implementation with a self-contained regex engine
//! supporting Basic Regular Expressions (BRE, default) and Extended Regular
//! Expressions (ERE, `-E`), plus a fast fixed-string path (`-F`).
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/grep.html

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::applets::get_arg;
use crate::io;

// ===========================================================================
// Resource limits (guard against pathological patterns / inputs)
// ===========================================================================

/// Maximum repetition count for an interval expression `{n,m}` (POSIX RE_DUP_MAX).
const RE_DUP_MAX: usize = 32767;

/// Backstop cap on the total number of compiled VM instructions. Interval
/// expansion multiplies instruction counts, so nested bounds are capped here
/// even when each individual bound is within RE_DUP_MAX.
const MAX_PROG_LEN: usize = 1 << 20;

/// Maximum group-nesting depth accepted by the parser.
const MAX_NEST_DEPTH: usize = 1000;

/// Maximum VM recursion depth. Bounds stack usage on very long input lines
/// (the matcher recurses roughly one frame per consumed byte).
const MAX_VM_DEPTH: usize = 100_000;

// ===========================================================================
// Options
// ===========================================================================

#[derive(Clone, Copy)]
struct Opts {
    ere: bool,
    fixed: bool,
    ignore_case: bool,
    invert: bool,
    count_only: bool,
    list_files: bool,
    line_numbers: bool,
    quiet: bool,
    suppress_errors: bool,
    whole_line: bool,
    word: bool,
}

impl Opts {
    fn new(ere: bool, fixed: bool) -> Self {
        Opts {
            ere,
            fixed,
            ignore_case: false,
            invert: false,
            count_only: false,
            list_files: false,
            line_numbers: false,
            quiet: false,
            suppress_errors: false,
            whole_line: false,
            word: false,
        }
    }
}

// ===========================================================================
// Regex AST
// ===========================================================================

#[derive(Clone)]
enum Ast {
    Empty,
    Lit(u8),
    Any,
    Class([u8; 32], bool), // membership bitmap, negate flag
    Start,
    End,
    Concat(Vec<Ast>),
    Alt(Vec<Ast>),
    Star(Box<Ast>),
    Plus(Box<Ast>),
    Quest(Box<Ast>),
    Repeat(Box<Ast>, usize, Option<usize>),
}

// ===========================================================================
// Regex parser (BRE + ERE)
// ===========================================================================

struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
    ere: bool,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn new(b: &'a [u8], ere: bool) -> Self {
        Parser {
            b,
            pos: 0,
            ere,
            depth: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.pos).copied()
    }

    /// True if the remaining input starts with the two bytes `\x`.
    fn at_escaped(&self, x: u8) -> bool {
        self.pos + 1 < self.b.len() && self.b[self.pos] == b'\\' && self.b[self.pos + 1] == x
    }

    /// Alternation separator at the current position?
    fn at_alt(&self) -> bool {
        if self.ere {
            self.peek() == Some(b'|')
        } else {
            self.at_escaped(b'|')
        }
    }

    /// Group-close token at the current position?
    fn at_group_close(&self) -> bool {
        if self.ere {
            self.peek() == Some(b')')
        } else {
            self.at_escaped(b')')
        }
    }

    fn parse(&mut self) -> Result<Ast, ()> {
        let ast = self.parse_alt()?;
        if self.pos != self.b.len() {
            // Trailing unconsumed input (e.g. an unmatched close) is an error.
            return Err(());
        }
        Ok(ast)
    }

    fn parse_alt(&mut self) -> Result<Ast, ()> {
        let mut branches = Vec::new();
        branches.push(self.parse_concat()?);
        while self.at_alt() {
            self.pos += if self.ere { 1 } else { 2 };
            branches.push(self.parse_concat()?);
        }
        if branches.len() == 1 {
            Ok(branches.pop().unwrap())
        } else {
            Ok(Ast::Alt(branches))
        }
    }

    /// End-of-branch context for a `$` anchor in BRE: end of pattern, or
    /// immediately before `\)` or `\|`.
    fn bre_dollar_is_anchor(&self) -> bool {
        let next = self.pos + 1;
        if next >= self.b.len() {
            return true;
        }
        // \) or \|
        self.b[next] == b'\\'
            && next + 1 < self.b.len()
            && (self.b[next + 1] == b')' || self.b[next + 1] == b'|')
    }

    fn parse_concat(&mut self) -> Result<Ast, ()> {
        let mut items: Vec<Ast> = Vec::new();
        let mut first = true;
        loop {
            if self.pos >= self.b.len() || self.at_alt() || self.at_group_close() {
                break;
            }
            let c = self.b[self.pos];

            // '^' anchor: always in ERE; only at branch start in BRE.
            if c == b'^' && (self.ere || first) {
                self.pos += 1;
                items.push(Ast::Start);
                first = false;
                continue;
            }
            // '$' anchor: always in ERE; only at branch end in BRE.
            if c == b'$' && (self.ere || self.bre_dollar_is_anchor()) {
                self.pos += 1;
                items.push(Ast::End);
                first = false;
                continue;
            }

            let atom = self.parse_atom(first)?;
            let atom = self.parse_repeat(atom)?;
            items.push(atom);
            first = false;
        }
        if items.is_empty() {
            Ok(Ast::Empty)
        } else if items.len() == 1 {
            Ok(items.pop().unwrap())
        } else {
            Ok(Ast::Concat(items))
        }
    }

    fn parse_atom(&mut self, at_branch_start: bool) -> Result<Ast, ()> {
        let c = self.b[self.pos];

        // Grouping.
        if self.ere && c == b'(' {
            self.pos += 1;
            self.depth += 1;
            if self.depth > MAX_NEST_DEPTH {
                return Err(());
            }
            let inner = self.parse_alt()?;
            if !self.at_group_close() {
                return Err(());
            }
            self.pos += 1;
            self.depth -= 1;
            return Ok(inner);
        }
        if !self.ere && self.at_escaped(b'(') {
            self.pos += 2;
            self.depth += 1;
            if self.depth > MAX_NEST_DEPTH {
                return Err(());
            }
            let inner = self.parse_alt()?;
            if !self.at_group_close() {
                return Err(());
            }
            self.pos += 2;
            self.depth -= 1;
            return Ok(inner);
        }

        // '.' any char.
        if c == b'.' {
            self.pos += 1;
            return Ok(Ast::Any);
        }

        // Bracket expression.
        if c == b'[' {
            return self.parse_class();
        }

        // Backslash escape -> literal next byte (or specials handled elsewhere).
        if c == b'\\' {
            if self.pos + 1 < self.b.len() {
                let lit = self.b[self.pos + 1];
                self.pos += 2;
                return Ok(Ast::Lit(lit));
            }
            // Trailing backslash: literal backslash.
            self.pos += 1;
            return Ok(Ast::Lit(b'\\'));
        }

        // A '*' with no preceding atom is a literal (leading star rule).
        // Since parse_repeat consumes trailing quantifiers, a '*' reaching
        // parse_atom is at branch start.
        let _ = at_branch_start;
        self.pos += 1;
        Ok(Ast::Lit(c))
    }

    /// Read a decimal count. Returns `None` if no digits are present. Uses
    /// checked arithmetic: any value that overflows `usize` is clamped to
    /// `usize::MAX`, which the caller rejects as above `RE_DUP_MAX`.
    fn read_num(&mut self) -> Option<usize> {
        let s = self.pos;
        let mut v: usize = 0;
        while let Some(d) = self.peek() {
            if d.is_ascii_digit() {
                v = match v
                    .checked_mul(10)
                    .and_then(|x| x.checked_add((d - b'0') as usize))
                {
                    Some(x) => x,
                    None => usize::MAX,
                };
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == s {
            None
        } else {
            Some(v)
        }
    }

    /// Attempt to parse a `{n}`, `{n,}`, `{n,m}` bound at the current position.
    ///
    /// - `Ok(None)` (position restored): the text here is not a bound.
    /// - `Ok(Some(..))`: a valid bound within limits.
    /// - `Err(())`: the text is a syntactically complete bound whose counts are
    ///   out of range (above `RE_DUP_MAX`, or `n > m`) — a compile error.
    fn try_bound(&mut self) -> Result<Option<(usize, Option<usize>)>, ()> {
        let save = self.pos;
        // Opening token.
        if self.ere {
            if self.peek() != Some(b'{') {
                return Ok(None);
            }
            self.pos += 1;
        } else {
            if !self.at_escaped(b'{') {
                return Ok(None);
            }
            self.pos += 2;
        }

        let n = match self.read_num() {
            Some(v) => v,
            None => {
                self.pos = save;
                return Ok(None);
            }
        };
        let max;
        if self.peek() == Some(b',') {
            self.pos += 1;
            // {n,} open, or {n,m}
            let closing_now = if self.ere {
                self.peek() == Some(b'}')
            } else {
                self.at_escaped(b'}')
            };
            if closing_now {
                max = None;
            } else {
                match self.read_num() {
                    Some(m) => max = Some(m),
                    None => {
                        self.pos = save;
                        return Ok(None);
                    }
                }
            }
        } else {
            max = Some(n);
        }
        // Closing token.
        if self.ere {
            if self.peek() != Some(b'}') {
                self.pos = save;
                return Ok(None);
            }
            self.pos += 1;
        } else {
            if !self.at_escaped(b'}') {
                self.pos = save;
                return Ok(None);
            }
            self.pos += 2;
        }
        // Syntactically a bound: now enforce the count limits. Beyond this
        // point the input is consumed and out-of-range values are errors, not
        // a fallback to literal text.
        if n > RE_DUP_MAX {
            return Err(());
        }
        if let Some(m) = max {
            if m > RE_DUP_MAX || n > m {
                return Err(());
            }
        }
        Ok(Some((n, max)))
    }

    fn parse_repeat(&mut self, mut atom: Ast) -> Result<Ast, ()> {
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.pos += 1;
                    atom = Ast::Star(Box::new(atom));
                }
                Some(b'+') if self.ere => {
                    self.pos += 1;
                    atom = Ast::Plus(Box::new(atom));
                }
                Some(b'?') if self.ere => {
                    self.pos += 1;
                    atom = Ast::Quest(Box::new(atom));
                }
                _ => {
                    if !self.ere && self.at_escaped(b'+') {
                        self.pos += 2;
                        atom = Ast::Plus(Box::new(atom));
                        continue;
                    }
                    if !self.ere && self.at_escaped(b'?') {
                        self.pos += 2;
                        atom = Ast::Quest(Box::new(atom));
                        continue;
                    }
                    if let Some((n, m)) = self.try_bound()? {
                        atom = Ast::Repeat(Box::new(atom), n, m);
                        continue;
                    }
                    break;
                }
            }
        }
        Ok(atom)
    }

    fn parse_class(&mut self) -> Result<Ast, ()> {
        // Consume '['.
        self.pos += 1;
        let mut negate = false;
        if self.peek() == Some(b'^') {
            negate = true;
            self.pos += 1;
        }
        let mut bm = [0u8; 32];
        let set = |bm: &mut [u8; 32], c: u8| bm[(c >> 3) as usize] |= 1u8 << (c & 7);

        let mut first = true;
        loop {
            let c = match self.peek() {
                Some(c) => c,
                None => return Err(()), // unterminated
            };
            if c == b']' && !first {
                self.pos += 1;
                break;
            }

            // [:class:] character class.
            if c == b'[' && self.pos + 1 < self.b.len() && self.b[self.pos + 1] == b':' {
                // find ":]"
                let mut k = self.pos + 2;
                let start = k;
                while k + 1 < self.b.len() && !(self.b[k] == b':' && self.b[k + 1] == b']') {
                    k += 1;
                }
                if k + 1 >= self.b.len() {
                    return Err(());
                }
                let name = &self.b[start..k];
                add_named_class(&mut bm, name)?;
                self.pos = k + 2;
                first = false;
                continue;
            }

            // Collating [.x.] / equivalence [=x=]: take the enclosed byte literally.
            if c == b'['
                && self.pos + 1 < self.b.len()
                && (self.b[self.pos + 1] == b'.' || self.b[self.pos + 1] == b'=')
            {
                let delim = self.b[self.pos + 1];
                let mut k = self.pos + 2;
                let start = k;
                while k + 1 < self.b.len() && !(self.b[k] == delim && self.b[k + 1] == b']') {
                    k += 1;
                }
                if k + 1 >= self.b.len() {
                    return Err(());
                }
                for &byte in &self.b[start..k] {
                    set(&mut bm, byte);
                }
                self.pos = k + 2;
                first = false;
                continue;
            }

            // Range a-z (only when '-' is not the last char before ']').
            if self.pos + 2 < self.b.len()
                && self.b[self.pos + 1] == b'-'
                && self.b[self.pos + 2] != b']'
            {
                let lo = c;
                let hi = self.b[self.pos + 2];
                if lo <= hi {
                    let mut x = lo;
                    loop {
                        set(&mut bm, x);
                        if x == hi {
                            break;
                        }
                        x += 1;
                    }
                } else {
                    // Invalid range: treat literally.
                    set(&mut bm, lo);
                    set(&mut bm, b'-');
                    set(&mut bm, hi);
                }
                self.pos += 3;
                first = false;
                continue;
            }

            set(&mut bm, c);
            self.pos += 1;
            first = false;
        }
        Ok(Ast::Class(bm, negate))
    }
}

fn add_named_class(bm: &mut [u8; 32], name: &[u8]) -> Result<(), ()> {
    // Validate the class name once.
    match name {
        b"alpha" | b"digit" | b"alnum" | b"upper" | b"lower" | b"space" | b"blank" | b"punct"
        | b"cntrl" | b"graph" | b"print" | b"xdigit" => {}
        _ => return Err(()),
    }
    for v in 0u16..256 {
        let c = v as u8;
        let member = match name {
            b"alpha" => c.is_ascii_alphabetic(),
            b"digit" => c.is_ascii_digit(),
            b"alnum" => c.is_ascii_alphanumeric(),
            b"upper" => c.is_ascii_uppercase(),
            b"lower" => c.is_ascii_lowercase(),
            b"space" => matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c),
            b"blank" => c == b' ' || c == b'\t',
            b"punct" => c.is_ascii_punctuation(),
            b"cntrl" => c.is_ascii_control(),
            b"graph" => c.is_ascii_graphic(),
            b"print" => c.is_ascii_graphic() || c == b' ',
            b"xdigit" => c.is_ascii_hexdigit(),
            _ => false,
        };
        if member {
            bm[(c >> 3) as usize] |= 1u8 << (c & 7);
        }
    }
    Ok(())
}

// ===========================================================================
// Compiled program (backtracking VM instructions)
// ===========================================================================

#[derive(Clone)]
enum Inst {
    Char(u8),
    Any,
    Class([u8; 32], bool),
    AnchorStart,
    AnchorEnd,
    Split(usize, usize),
    Jmp(usize),
    Match,
}

fn compile(pat: &[u8], ere: bool) -> Result<Vec<Inst>, ()> {
    let ast = Parser::new(pat, ere).parse()?;
    let mut prog = Vec::new();
    emit(&mut prog, &ast);
    // Backstop: interval expansion (including nested bounds) can multiply the
    // instruction count. If we blew past the cap, reject the pattern.
    if prog.len() > MAX_PROG_LEN {
        return Err(());
    }
    prog.push(Inst::Match);
    Ok(prog)
}

fn emit(prog: &mut Vec<Inst>, ast: &Ast) {
    // Stop growing the program once the cap is exceeded; `compile` turns the
    // oversize program into a compile error.
    if prog.len() > MAX_PROG_LEN {
        return;
    }
    match ast {
        Ast::Empty => {}
        Ast::Lit(c) => prog.push(Inst::Char(*c)),
        Ast::Any => prog.push(Inst::Any),
        Ast::Class(bm, neg) => prog.push(Inst::Class(*bm, *neg)),
        Ast::Start => prog.push(Inst::AnchorStart),
        Ast::End => prog.push(Inst::AnchorEnd),
        Ast::Concat(v) => {
            for a in v {
                emit(prog, a);
            }
        }
        Ast::Alt(v) => emit_alt(prog, v),
        Ast::Star(b) => {
            let l1 = prog.len();
            prog.push(Inst::Split(0, 0));
            emit(prog, b);
            prog.push(Inst::Jmp(l1));
            let l3 = prog.len();
            prog[l1] = Inst::Split(l1 + 1, l3);
        }
        Ast::Plus(b) => {
            let l1 = prog.len();
            emit(prog, b);
            let sp = prog.len();
            prog.push(Inst::Split(l1, sp + 1));
        }
        Ast::Quest(b) => {
            let l1 = prog.len();
            prog.push(Inst::Split(0, 0));
            emit(prog, b);
            let l3 = prog.len();
            prog[l1] = Inst::Split(l1 + 1, l3);
        }
        Ast::Repeat(b, n, max) => {
            for _ in 0..*n {
                emit(prog, b);
            }
            match max {
                None => emit(prog, &Ast::Star(b.clone())),
                Some(m) => {
                    let extra = m.saturating_sub(*n);
                    for _ in 0..extra {
                        emit(prog, &Ast::Quest(b.clone()));
                    }
                }
            }
        }
    }
}

fn emit_alt(prog: &mut Vec<Inst>, branches: &[Ast]) {
    if branches.is_empty() {
        return;
    }
    let mut jmp_patches: Vec<usize> = Vec::new();
    let last = branches.len() - 1;
    for (idx, branch) in branches.iter().enumerate() {
        if idx == last {
            emit(prog, branch);
        } else {
            let sp = prog.len();
            prog.push(Inst::Split(0, 0));
            emit(prog, branch);
            let jmp = prog.len();
            prog.push(Inst::Jmp(0));
            jmp_patches.push(jmp);
            let next = prog.len();
            prog[sp] = Inst::Split(sp + 1, next);
        }
    }
    let end = prog.len();
    for j in jmp_patches {
        prog[j] = Inst::Jmp(end);
    }
}

// ===========================================================================
// Matching helpers
// ===========================================================================

#[inline]
fn to_lower(c: u8) -> u8 {
    if c.is_ascii_uppercase() {
        c + 32
    } else {
        c
    }
}

#[inline]
fn swap_case(c: u8) -> u8 {
    if c.is_ascii_uppercase() {
        c + 32
    } else if c.is_ascii_lowercase() {
        c - 32
    } else {
        c
    }
}

#[inline]
fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

#[inline]
fn class_contains(bm: &[u8; 32], c: u8) -> bool {
    (bm[(c >> 3) as usize] >> (c & 7)) & 1 == 1
}

#[inline]
fn class_match(bm: &[u8; 32], neg: bool, c: u8, ic: bool) -> bool {
    let mut m = class_contains(bm, c);
    if ic {
        m |= class_contains(bm, swap_case(c));
    }
    m ^ neg
}

struct Vm<'a> {
    prog: &'a [Inst],
    text: &'a [u8],
    ic: bool,
    whole_line: bool,
    word: bool,
    start: usize,
}

impl<'a> Vm<'a> {
    fn accept(&self, end: usize) -> bool {
        let len = self.text.len();
        if self.whole_line && !(self.start == 0 && end == len) {
            return false;
        }
        if self.word {
            let before = self.start == 0 || !is_word(self.text[self.start - 1]);
            let after = end == len || !is_word(self.text[end]);
            if !(before && after) {
                return false;
            }
        }
        true
    }

    fn run(&self, memo: &mut [bool], pc: usize, sp: usize, depth: usize) -> bool {
        // Bound stack growth: the matcher recurses ~1 frame per consumed byte,
        // so a multi-megabyte line could otherwise overflow the stack. Aborting
        // this path cleanly (treat as no-match for this start position) is safe:
        // the memo already bounds total work, and outer starts are still tried.
        if depth >= MAX_VM_DEPTH {
            return false;
        }
        let len = self.text.len();
        let key = pc * (len + 1) + sp;
        if memo[key] {
            return false;
        }
        memo[key] = true;

        let d = depth + 1;
        match &self.prog[pc] {
            Inst::Char(c) => {
                if sp < len {
                    let a = self.text[sp];
                    let hit = if self.ic {
                        to_lower(a) == to_lower(*c)
                    } else {
                        a == *c
                    };
                    hit && self.run(memo, pc + 1, sp + 1, d)
                } else {
                    false
                }
            }
            Inst::Any => sp < len && self.run(memo, pc + 1, sp + 1, d),
            Inst::Class(bm, neg) => {
                sp < len
                    && class_match(bm, *neg, self.text[sp], self.ic)
                    && self.run(memo, pc + 1, sp + 1, d)
            }
            Inst::AnchorStart => sp == 0 && self.run(memo, pc + 1, sp, d),
            Inst::AnchorEnd => sp == len && self.run(memo, pc + 1, sp, d),
            Inst::Split(a, b) => self.run(memo, *a, sp, d) || self.run(memo, *b, sp, d),
            Inst::Jmp(a) => self.run(memo, *a, sp, d),
            Inst::Match => self.accept(sp),
        }
    }
}

fn regex_search(prog: &[Inst], line: &[u8], opts: &Opts) -> bool {
    let len = line.len();
    let ninst = prog.len();
    let mut memo = vec![false; ninst * (len + 1)];

    // For -x the match must begin at column 0, so only try start 0.
    let last_start = if opts.whole_line { 0 } else { len };
    for start in 0..=last_start {
        for e in memo.iter_mut() {
            *e = false;
        }
        let vm = Vm {
            prog,
            text: line,
            ic: opts.ignore_case,
            whole_line: opts.whole_line,
            word: opts.word,
            start,
        };
        if vm.run(&mut memo, 0, start, 0) {
            return true;
        }
    }
    false
}

fn slice_eq_ci(a: &[u8], b: &[u8], ic: bool) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if ic {
            if to_lower(a[i]) != to_lower(b[i]) {
                return false;
            }
        } else if a[i] != b[i] {
            return false;
        }
    }
    true
}

fn fixed_match(line: &[u8], pat: &[u8], opts: &Opts) -> bool {
    if opts.whole_line {
        return slice_eq_ci(line, pat, opts.ignore_case);
    }
    if pat.is_empty() {
        return true;
    }
    if line.len() < pat.len() {
        return false;
    }
    for i in 0..=(line.len() - pat.len()) {
        if slice_eq_ci(&line[i..i + pat.len()], pat, opts.ignore_case) {
            if opts.word {
                let before = i == 0 || !is_word(line[i - 1]);
                let after = i + pat.len() == line.len() || !is_word(line[i + pat.len()]);
                if before && after {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    false
}

enum Compiled {
    Regex(Vec<Vec<Inst>>),
    Fixed(Vec<Vec<u8>>),
}

impl Compiled {
    fn matches(&self, line: &[u8], opts: &Opts) -> bool {
        match self {
            Compiled::Regex(progs) => progs.iter().any(|p| regex_search(p, line, opts)),
            Compiled::Fixed(pats) => pats.iter().any(|p| fixed_match(line, p, opts)),
        }
    }
}

// ===========================================================================
// Pattern collection
// ===========================================================================

/// Split a raw pattern blob on newlines into individual patterns.
/// A single trailing newline does not create an empty trailing pattern.
fn split_patterns(blob: &[u8], out: &mut Vec<Vec<u8>>) {
    let mut start = 0;
    let len = blob.len();
    let mut i = 0;
    while i < len {
        if blob[i] == b'\n' {
            out.push(blob[start..i].to_vec());
            start = i + 1;
        }
        i += 1;
    }
    if start < len {
        out.push(blob[start..len].to_vec());
    }
    // If blob is empty, treat as a single empty pattern (matches everything).
    if len == 0 {
        out.push(Vec::new());
    }
}

// ===========================================================================
// Line iteration over a byte buffer (no fixed line buffer)
// ===========================================================================

/// Call `f(line_bytes)` for each line in `data`. A trailing newline does not
/// yield an empty final line.
fn for_each_line<F: FnMut(&[u8])>(data: &[u8], mut f: F) {
    let mut start = 0;
    let len = data.len();
    let mut i = 0;
    while i < len {
        if data[i] == b'\n' {
            f(&data[start..i]);
            start = i + 1;
        }
        i += 1;
    }
    if start < len {
        f(&data[start..len]);
    }
}

// ===========================================================================
// Entry points
// ===========================================================================

/// grep - search a file for a pattern (BRE by default).
pub fn grep(argc: i32, argv: *const *const u8) -> i32 {
    grep_main(argc, argv, false, false)
}

/// egrep - grep with Extended Regular Expressions (`-E`).
pub fn egrep(argc: i32, argv: *const *const u8) -> i32 {
    grep_main(argc, argv, true, false)
}

/// fgrep - grep with fixed strings (`-F`).
pub fn fgrep(argc: i32, argv: *const *const u8) -> i32 {
    grep_main(argc, argv, false, true)
}

fn grep_main(argc: i32, argv: *const *const u8, default_ere: bool, default_fixed: bool) -> i32 {
    let mut opts = Opts::new(default_ere, default_fixed);
    let mut patterns: Vec<Vec<u8>> = Vec::new();
    let mut have_pattern_source = false;
    let mut operands: Vec<&[u8]> = Vec::new();
    // Filename display override: Some(true) for -H, Some(false) for -h.
    let mut with_filename: Option<bool> = None;
    let mut no_more_opts = false;
    let mut had_error = false;

    let mut i = 1;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => {
                i += 1;
                continue;
            }
        };

        if !no_more_opts && arg == b"--" {
            no_more_opts = true;
            i += 1;
            continue;
        }

        if !no_more_opts && arg.len() > 1 && arg[0] == b'-' {
            let mut j = 1;
            while j < arg.len() {
                let c = arg[j];
                match c {
                    b'E' => {
                        opts.ere = true;
                        opts.fixed = false;
                    }
                    b'F' => {
                        opts.fixed = true;
                        opts.ere = false;
                    }
                    b'i' => opts.ignore_case = true,
                    b'v' => opts.invert = true,
                    b'c' => opts.count_only = true,
                    b'l' => opts.list_files = true,
                    b'n' => opts.line_numbers = true,
                    b'q' => opts.quiet = true,
                    b's' => opts.suppress_errors = true,
                    b'x' => opts.whole_line = true,
                    b'w' => opts.word = true,
                    b'H' => with_filename = Some(true),
                    b'h' => with_filename = Some(false),
                    b'e' | b'f' => {
                        // Option value: rest of this cluster, else next argument.
                        let val: Option<&[u8]> = if j + 1 < arg.len() {
                            let v = &arg[j + 1..];
                            j = arg.len(); // consume remainder
                            Some(v)
                        } else {
                            i += 1;
                            unsafe { get_arg(argv, i) }
                        };
                        let Some(v) = val else {
                            io::write_str(2, b"grep: option requires an argument\n");
                            return 2;
                        };
                        if c == b'e' {
                            split_patterns(v, &mut patterns);
                        } else {
                            let fd = io::open(v, libc::O_RDONLY, 0);
                            if fd < 0 {
                                if !opts.suppress_errors {
                                    io::write_str(2, b"grep: ");
                                    io::write_all(2, v);
                                    io::write_str(2, b": No such file or directory\n");
                                }
                                return 2;
                            }
                            let data = io::read_all(fd);
                            if fd != 0 {
                                io::close(fd);
                            }
                            split_patterns(&data, &mut patterns);
                        }
                        have_pattern_source = true;
                        break; // done with this cluster
                    }
                    _ => {
                        io::write_str(2, b"grep: invalid option -- '");
                        io::write_all(2, &[c]);
                        io::write_str(2, b"'\n");
                        return 2;
                    }
                }
                j += 1;
            }
        } else {
            operands.push(arg);
        }
        i += 1;
    }

    // Determine the pattern source if no -e/-f were given.
    if !have_pattern_source {
        if operands.is_empty() {
            io::write_str(2, b"grep: missing pattern\n");
            return 2;
        }
        let pat = operands.remove(0);
        split_patterns(pat, &mut patterns);
    }

    // Compile patterns.
    let compiled = if opts.fixed {
        Compiled::Fixed(patterns)
    } else {
        let mut progs = Vec::with_capacity(patterns.len());
        for p in &patterns {
            match compile(p, opts.ere) {
                Ok(prog) => progs.push(prog),
                Err(_) => {
                    io::write_str(2, b"grep: invalid regular expression\n");
                    return 2;
                }
            }
        }
        Compiled::Regex(progs)
    };

    let files = operands;
    let multiple_files = files.len() > 1;
    let show_name = with_filename.unwrap_or(multiple_files);

    let mut found_any = false;

    if files.is_empty() {
        // stdin
        let data = io::read_all(0);
        let r = process_source(
            &data,
            b"(standard input)",
            show_name,
            &compiled,
            &opts,
            &mut found_any,
        );
        if r {
            found_any = true;
        }
    } else {
        for &f in &files {
            let is_stdin = f == b"-";
            let fd = if is_stdin {
                0
            } else {
                io::open(f, libc::O_RDONLY, 0)
            };
            if fd < 0 {
                if !opts.suppress_errors {
                    io::write_str(2, b"grep: ");
                    io::write_all(2, f);
                    io::write_str(2, b": No such file or directory\n");
                }
                had_error = true;
                continue;
            }
            let data = io::read_all(fd);
            if fd != 0 {
                io::close(fd);
            }
            let name: &[u8] = if is_stdin { b"(standard input)" } else { f };
            let r = process_source(&data, name, show_name, &compiled, &opts, &mut found_any);
            if r {
                found_any = true;
            }
        }
    }

    if had_error {
        2
    } else if found_any {
        0
    } else {
        1
    }
}

/// Process one input source. Returns true if any line was selected.
///
/// Handles -q (immediate exit), -l, -c and normal line output.
fn process_source(
    data: &[u8],
    name: &[u8],
    show_name: bool,
    compiled: &Compiled,
    opts: &Opts,
    found_any: &mut bool,
) -> bool {
    let mut count: u64 = 0;
    let mut line_no: u64 = 0;
    let mut any = false;

    for_each_line(data, |line| {
        line_no += 1;
        let selected = compiled.matches(line, opts) != opts.invert;
        if !selected {
            return;
        }
        any = true;
        *found_any = true;
        count += 1;

        if opts.quiet {
            // Match found: exit immediately with success.
            io::exit(0);
        }
        if opts.list_files || opts.count_only {
            // Output deferred until after the loop.
            return;
        }
        // Normal line output.
        if show_name {
            io::write_all(1, name);
            io::write_str(1, b":");
        }
        if opts.line_numbers {
            io::write_num(1, line_no);
            io::write_str(1, b":");
        }
        io::write_all(1, line);
        io::write_str(1, b"\n");
    });

    if !opts.quiet {
        if opts.list_files {
            if any {
                io::write_all(1, name);
                io::write_str(1, b"\n");
            }
        } else if opts.count_only {
            if show_name {
                io::write_all(1, name);
                io::write_str(1, b":");
            }
            io::write_num(1, count);
            io::write_str(1, b"\n");
        }
    }

    any
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    fn get_armybox_path() -> PathBuf {
        if let Ok(path) = std::env::var("ARMYBOX_PATH") {
            return PathBuf::from(path);
        }
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let release = manifest_dir.join("target/release/armybox");
        if release.exists() {
            return release;
        }
        manifest_dir.join("target/debug/armybox")
    }

    fn setup() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "armybox_grep_test_{}_{}",
            std::process::id(),
            counter
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_grep_basic() {
        let armybox = get_armybox_path();
        if !armybox.exists() {
            return;
        }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "hello world\nfoo bar\nhello again\n").unwrap();

        let output = Command::new(&armybox)
            .args(["grep", "hello", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("hello"));
        assert!(lines[1].contains("hello"));
        cleanup(&dir);
    }

    #[test]
    fn test_grep_line_numbers() {
        let armybox = get_armybox_path();
        if !armybox.exists() {
            return;
        }

        let dir = setup();
        let file = dir.join("test.txt");
        fs::write(&file, "one\ntwo\nthree\ntwo again\n").unwrap();

        let output = Command::new(&armybox)
            .args(["grep", "-n", "two", file.to_str().unwrap()])
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2:two"));
        assert!(stdout.contains("4:two again"));
        cleanup(&dir);
    }

    #[test]
    fn test_grep_count() {
        let armybox = get_armybox_path();
        if !armybox.exists() {
            return;
        }

        let mut child = Command::new(&armybox)
            .args(["grep", "-c", "test"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin
                .write_all(b"test one\nno match\ntest two\ntest three\n")
                .unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "3");
    }

    #[test]
    fn test_grep_invert() {
        let armybox = get_armybox_path();
        if !armybox.exists() {
            return;
        }

        let mut child = Command::new(&armybox)
            .args(["grep", "-v", "skip"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin
                .write_all(b"keep this\nskip this\nkeep that\n")
                .unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["keep this", "keep that"]);
    }

    #[test]
    fn test_grep_ignore_case() {
        let armybox = get_armybox_path();
        if !armybox.exists() {
            return;
        }

        let mut child = Command::new(&armybox)
            .args(["grep", "-i", "hello"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"Hello World\nHELLO\nhello\nhi\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_grep_no_match() {
        let armybox = get_armybox_path();
        if !armybox.exists() {
            return;
        }

        let mut child = Command::new(&armybox)
            .args(["grep", "notfound"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"some text\nother text\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(1)); // No match returns 1
    }
}
