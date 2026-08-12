//! sed - stream editor
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/sed.html
//!
//! Implements a self-contained backtracking BRE/ERE regex engine (with capture
//! groups, memoized to bound catastrophic backtracking) plus a full sed cycle:
//! addresses (line, `$`, `/re/`, ranges, `0,/re/`, `!`), address-scoped `{ }`
//! blocks, and the commands `s y d p a i c q n N g G h H x = b t : { }`.

use crate::io;
use crate::applets::get_arg;

use alloc::vec::Vec;
use alloc::boxed::Box;

// ---------------------------------------------------------------------------
// Regex engine
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Class {
    negated: bool,
    ranges: Vec<(u8, u8)>,
}

impl Class {
    fn matches(&self, b: u8, icase: bool) -> bool {
        let hit = |x: u8| self.ranges.iter().any(|&(lo, hi)| x >= lo && x <= hi);
        let mut m = hit(b);
        if icase && !m {
            m = hit(other_case(b));
        }
        if self.negated { !m } else { m }
    }
}

#[derive(Clone)]
enum Inst {
    Char(u8),
    Any,
    Class(Class),
    Save(usize),
    Jmp(usize),
    Split(usize, usize),
    Start,
    End,
    Match,
}

#[derive(Clone)]
enum Ast {
    Empty,
    Char(u8),
    Any,
    Class(Class),
    Start,
    End,
    Concat(Vec<Ast>),
    Alt(Vec<Ast>),
    Group(usize, Box<Ast>),
    Star(Box<Ast>),
    Plus(Box<Ast>),
    Quest(Box<Ast>),
    Repeat(Box<Ast>, usize, Option<usize>),
}

fn other_case(x: u8) -> u8 {
    if x.is_ascii_lowercase() {
        x.to_ascii_uppercase()
    } else if x.is_ascii_uppercase() {
        x.to_ascii_lowercase()
    } else {
        x
    }
}

fn eqc(a: u8, b: u8, icase: bool) -> bool {
    a == b || (icase && a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

fn is_alt_op(b: &[u8], p: usize, ere: bool) -> bool {
    p < b.len()
        && ((ere && b[p] == b'|')
            || (!ere && b[p] == b'\\' && p + 1 < b.len() && b[p + 1] == b'|'))
}

fn is_group_close(b: &[u8], p: usize, ere: bool) -> bool {
    p < b.len()
        && ((ere && b[p] == b')')
            || (!ere && b[p] == b'\\' && p + 1 < b.len() && b[p + 1] == b')'))
}

fn is_group_open(b: &[u8], p: usize, ere: bool) -> bool {
    p < b.len()
        && ((ere && b[p] == b'(')
            || (!ere && b[p] == b'\\' && p + 1 < b.len() && b[p + 1] == b'('))
}

/// POSIX RE_DUP_MAX: maximum permitted count in a `{n,m}` interval.
const RE_DUP_MAX: usize = 32767;
/// Maximum parser nesting depth for groups/alternation (parse-error backstop).
const MAX_NEST: usize = 200;
/// Maximum recursion depth for the matcher (stack-overflow backstop on long lines).
const RE_MAX_DEPTH: usize = 40_000;

struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
    ere: bool,
    gc: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn alt(&mut self) -> Result<Ast, ()> {
        self.depth += 1;
        if self.depth > MAX_NEST {
            return Err(());
        }
        let mut br: Vec<Ast> = Vec::new();
        br.push(self.concat()?);
        while is_alt_op(self.b, self.pos, self.ere) {
            self.pos += if self.ere { 1 } else { 2 };
            br.push(self.concat()?);
        }
        self.depth -= 1;
        Ok(if br.len() == 1 {
            br.pop().unwrap()
        } else {
            Ast::Alt(br)
        })
    }

    fn concat(&mut self) -> Result<Ast, ()> {
        let mut items: Vec<Ast> = Vec::new();
        while self.pos < self.b.len()
            && !is_alt_op(self.b, self.pos, self.ere)
            && !is_group_close(self.b, self.pos, self.ere)
        {
            let first = items.is_empty();
            let a = self.atom(first)?;
            let a = self.quant(a)?;
            items.push(a);
        }
        Ok(if items.is_empty() {
            Ast::Empty
        } else if items.len() == 1 {
            items.pop().unwrap()
        } else {
            Ast::Concat(items)
        })
    }

    fn atom(&mut self, first: bool) -> Result<Ast, ()> {
        let b = self.b;
        if is_group_open(b, self.pos, self.ere) {
            self.pos += if self.ere { 1 } else { 2 };
            self.gc += 1;
            let idx = self.gc;
            let inner = self.alt()?;
            if is_group_close(b, self.pos, self.ere) {
                self.pos += if self.ere { 1 } else { 2 };
            } else {
                // Unterminated group: `\(` (BRE) or `(` (ERE) with no close.
                return Err(());
            }
            return Ok(Ast::Group(idx, Box::new(inner)));
        }
        let c = b[self.pos];
        if c == b'^' && first {
            self.pos += 1;
            return Ok(Ast::Start);
        }
        if c == b'$' {
            let nx = self.pos + 1;
            let end = nx >= b.len()
                || is_alt_op(b, nx, self.ere)
                || is_group_close(b, nx, self.ere);
            if end {
                self.pos += 1;
                return Ok(Ast::End);
            }
        }
        if c == b'.' {
            self.pos += 1;
            return Ok(Ast::Any);
        }
        if c == b'[' {
            return self.class();
        }
        if c == b'\\' && self.pos + 1 < b.len() {
            let nx = b[self.pos + 1];
            self.pos += 2;
            let ch = match nx {
                b'n' => b'\n',
                b't' => b'\t',
                b'r' => b'\r',
                _ => nx,
            };
            return Ok(Ast::Char(ch));
        }
        self.pos += 1;
        Ok(Ast::Char(c))
    }

    fn quant(&mut self, mut a: Ast) -> Result<Ast, ()> {
        loop {
            if self.pos >= self.b.len() {
                break;
            }
            let c = self.b[self.pos];
            if c == b'*' {
                self.pos += 1;
                a = Ast::Star(Box::new(a));
                continue;
            }
            if self.ere {
                if c == b'+' {
                    self.pos += 1;
                    a = Ast::Plus(Box::new(a));
                    continue;
                }
                if c == b'?' {
                    self.pos += 1;
                    a = Ast::Quest(Box::new(a));
                    continue;
                }
                if c == b'{' {
                    if let Some((mn, mx)) = self.interval(self.pos + 1, false)? {
                        a = Ast::Repeat(Box::new(a), mn, mx);
                        continue;
                    }
                }
            } else if c == b'\\' && self.pos + 1 < self.b.len() {
                let nx = self.b[self.pos + 1];
                if nx == b'+' {
                    self.pos += 2;
                    a = Ast::Plus(Box::new(a));
                    continue;
                }
                if nx == b'?' {
                    self.pos += 2;
                    a = Ast::Quest(Box::new(a));
                    continue;
                }
                if nx == b'{' {
                    if let Some((mn, mx)) = self.interval(self.pos + 2, true)? {
                        a = Ast::Repeat(Box::new(a), mn, mx);
                        continue;
                    }
                }
            }
            break;
        }
        Ok(a)
    }

    /// Parse a `{n}` / `{n,}` / `{n,m}` interval starting at `start`.
    /// `Ok(None)` means "not a well-formed interval here" (treat `{` literally);
    /// `Err(())` means a syntactically-present interval with an illegal bound
    /// (overflow / count > RE_DUP_MAX / n > m).
    fn interval(&mut self, start: usize, bre: bool) -> Result<Option<(usize, Option<usize>)>, ()> {
        let b = self.b;
        let mut p = start;
        let mut mn: usize = 0;
        let mut mn_overflow = false;
        let mut got = false;
        while p < b.len() && b[p].is_ascii_digit() {
            mn = mn.saturating_mul(10).saturating_add((b[p] - b'0') as usize);
            if mn > RE_DUP_MAX {
                mn_overflow = true;
            }
            p += 1;
            got = true;
        }
        if !got {
            return Ok(None);
        }
        let mut mx_overflow = false;
        let mx;
        if p < b.len() && b[p] == b',' {
            p += 1;
            if p < b.len() && b[p].is_ascii_digit() {
                let mut m: usize = 0;
                while p < b.len() && b[p].is_ascii_digit() {
                    m = m.saturating_mul(10).saturating_add((b[p] - b'0') as usize);
                    if m > RE_DUP_MAX {
                        mx_overflow = true;
                    }
                    p += 1;
                }
                mx = Some(m);
            } else {
                mx = None;
            }
        } else {
            mx = Some(mn);
        }
        // Require the closing brace before deciding this is an interval at all.
        if bre {
            if p + 1 < b.len() && b[p] == b'\\' && b[p + 1] == b'}' {
                p += 2;
            } else {
                return Ok(None);
            }
        } else if p < b.len() && b[p] == b'}' {
            p += 1;
        } else {
            return Ok(None);
        }
        // Syntactically an interval: now validate the bounds.
        if mn_overflow || mx_overflow {
            return Err(());
        }
        if let Some(m) = mx {
            if mn > m {
                return Err(());
            }
        }
        self.pos = p;
        Ok(Some((mn, mx)))
    }

    fn class(&mut self) -> Result<Ast, ()> {
        let b = self.b;
        self.pos += 1; // consume '['
        let mut negated = false;
        if self.pos < b.len() && b[self.pos] == b'^' {
            negated = true;
            self.pos += 1;
        }
        let mut ranges: Vec<(u8, u8)> = Vec::new();
        let mut first = true;
        let mut closed = false;
        while self.pos < b.len() {
            let c = b[self.pos];
            if c == b']' && !first {
                self.pos += 1;
                closed = true;
                break;
            }
            first = false;
            if self.pos + 2 < b.len() && b[self.pos + 1] == b'-' && b[self.pos + 2] != b']' {
                ranges.push((c, b[self.pos + 2]));
                self.pos += 3;
            } else {
                ranges.push((c, c));
                self.pos += 1;
            }
        }
        if !closed {
            // Unterminated bracket expression.
            return Err(());
        }
        Ok(Ast::Class(Class { negated, ranges }))
    }
}

fn emit(ast: &Ast, prog: &mut Vec<Inst>) {
    match ast {
        Ast::Empty => {}
        Ast::Char(c) => prog.push(Inst::Char(*c)),
        Ast::Any => prog.push(Inst::Any),
        Ast::Class(cl) => prog.push(Inst::Class(cl.clone())),
        Ast::Start => prog.push(Inst::Start),
        Ast::End => prog.push(Inst::End),
        Ast::Concat(v) => {
            for a in v {
                emit(a, prog);
            }
        }
        Ast::Group(n, inner) => {
            prog.push(Inst::Save(2 * n));
            emit(inner, prog);
            prog.push(Inst::Save(2 * n + 1));
        }
        Ast::Alt(v) => {
            let mut jmps: Vec<usize> = Vec::new();
            for (i, a) in v.iter().enumerate() {
                if i + 1 < v.len() {
                    let sp = prog.len();
                    prog.push(Inst::Split(0, 0));
                    let astart = prog.len();
                    emit(a, prog);
                    let jp = prog.len();
                    prog.push(Inst::Jmp(0));
                    jmps.push(jp);
                    let next = prog.len();
                    prog[sp] = Inst::Split(astart, next);
                } else {
                    emit(a, prog);
                }
            }
            let end = prog.len();
            for jp in jmps {
                prog[jp] = Inst::Jmp(end);
            }
        }
        Ast::Star(inner) => {
            let sp = prog.len();
            prog.push(Inst::Split(0, 0));
            let body = prog.len();
            emit(inner, prog);
            prog.push(Inst::Jmp(sp));
            let out = prog.len();
            prog[sp] = Inst::Split(body, out);
        }
        Ast::Plus(inner) => {
            let body = prog.len();
            emit(inner, prog);
            let sp = prog.len();
            prog.push(Inst::Split(0, 0));
            prog[sp] = Inst::Split(body, prog.len());
        }
        Ast::Quest(inner) => {
            let sp = prog.len();
            prog.push(Inst::Split(0, 0));
            let body = prog.len();
            emit(inner, prog);
            let out = prog.len();
            prog[sp] = Inst::Split(body, out);
        }
        Ast::Repeat(inner, mn, mx) => {
            for _ in 0..*mn {
                emit(inner, prog);
            }
            match mx {
                None => emit(&Ast::Star(inner.clone()), prog),
                Some(m) => {
                    for _ in *mn..*m {
                        emit(&Ast::Quest(inner.clone()), prog);
                    }
                }
            }
        }
    }
}

struct Regex {
    prog: Vec<Inst>,
    nslots: usize,
    icase: bool,
}

/// Backtracking matcher.
///
/// `memo[pc*(len+1)+pos]` records that `(pc, pos)` has already been entered
/// during the current match attempt. Revisiting it can never succeed (any
/// successful continuation would have been found the first time), so we prune
/// it. This bounds catastrophic backtracking (ReDoS) and also serves as the
/// empty-loop progress guard: a nullable star that loops back to its own
/// `Split` at the same `pos` hits the memo and terminates instead of recursing
/// forever. `depth` is a hard stack-overflow backstop on very long lines.
///
/// The pattern language has no back-references, so captures never gate whether
/// the remainder matches; pruning by `(pc, pos)` alone is therefore sound, and
/// the greedy-first exploration still records greedy-correct capture slots.
#[allow(clippy::too_many_arguments)]
fn re_run(
    prog: &[Inst],
    icase: bool,
    pc: usize,
    text: &[u8],
    pos: usize,
    saves: &mut Vec<Option<usize>>,
    memo: &mut [bool],
    depth: usize,
) -> Option<usize> {
    if depth > RE_MAX_DEPTH {
        return None;
    }
    let len = text.len();
    let key = pc * (len + 1) + pos;
    if memo[key] {
        return None;
    }
    memo[key] = true;
    let d = depth + 1;
    match &prog[pc] {
        Inst::Match => Some(pos),
        Inst::Char(c) => {
            if pos < text.len() && eqc(text[pos], *c, icase) {
                re_run(prog, icase, pc + 1, text, pos + 1, saves, memo, d)
            } else {
                None
            }
        }
        Inst::Any => {
            if pos < text.len() {
                re_run(prog, icase, pc + 1, text, pos + 1, saves, memo, d)
            } else {
                None
            }
        }
        Inst::Class(cl) => {
            if pos < text.len() && cl.matches(text[pos], icase) {
                re_run(prog, icase, pc + 1, text, pos + 1, saves, memo, d)
            } else {
                None
            }
        }
        Inst::Start => {
            if pos == 0 {
                re_run(prog, icase, pc + 1, text, pos, saves, memo, d)
            } else {
                None
            }
        }
        Inst::End => {
            if pos == text.len() {
                re_run(prog, icase, pc + 1, text, pos, saves, memo, d)
            } else {
                None
            }
        }
        Inst::Jmp(x) => re_run(prog, icase, *x, text, pos, saves, memo, d),
        Inst::Split(a, b) => {
            // No snapshot of `saves` is needed: every `Save` frame restores its
            // own touched slot when its subtree fails, so on a `None` return all
            // capture slots are already back to their pre-call values.
            if let Some(e) = re_run(prog, icase, *a, text, pos, saves, memo, d) {
                return Some(e);
            }
            re_run(prog, icase, *b, text, pos, saves, memo, d)
        }
        Inst::Save(k) => {
            let old = saves[*k];
            saves[*k] = Some(pos);
            match re_run(prog, icase, pc + 1, text, pos, saves, memo, d) {
                Some(e) => Some(e),
                None => {
                    saves[*k] = old;
                    None
                }
            }
        }
    }
}

impl Regex {
    fn compile(src: &[u8], ere: bool, icase: bool) -> Result<Regex, ()> {
        let mut p = Parser {
            b: src,
            pos: 0,
            ere,
            gc: 0,
            depth: 0,
        };
        let ast = p.alt()?;
        // Leftover input means an unmatched group close (e.g. a stray `\)` / `)`).
        if p.pos != src.len() {
            return Err(());
        }
        let ngroups = p.gc;
        let mut prog: Vec<Inst> = Vec::new();
        prog.push(Inst::Save(0));
        emit(&ast, &mut prog);
        prog.push(Inst::Save(1));
        prog.push(Inst::Match);
        Ok(Regex {
            prog,
            nslots: 2 * (ngroups + 1),
            icase,
        })
    }

    fn search_from(&self, text: &[u8], start: usize) -> Option<Vec<Option<usize>>> {
        let len = text.len();
        let cells = self.prog.len() * (len + 1);
        let mut memo: Vec<bool> = Vec::new();
        memo.resize(cells, false);
        let mut s = start;
        while s <= len {
            for e in memo.iter_mut() {
                *e = false;
            }
            let mut saves: Vec<Option<usize>> = Vec::new();
            saves.resize(self.nslots, None);
            if re_run(&self.prog, self.icase, 0, text, s, &mut saves, &mut memo, 0).is_some() {
                return Some(saves);
            }
            s += 1;
        }
        None
    }

    fn is_match(&self, text: &[u8]) -> bool {
        self.search_from(text, 0).is_some()
    }
}

// ---------------------------------------------------------------------------
// Replacement text
// ---------------------------------------------------------------------------

enum ReplPart {
    Lit(u8),
    Amp,
    Group(usize),
}

fn parse_repl(src: &[u8]) -> Vec<ReplPart> {
    let mut out: Vec<ReplPart> = Vec::new();
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        if c == b'&' {
            out.push(ReplPart::Amp);
            i += 1;
        } else if c == b'\\' && i + 1 < src.len() {
            let nx = src[i + 1];
            match nx {
                b'0'..=b'9' => out.push(ReplPart::Group((nx - b'0') as usize)),
                b'n' => out.push(ReplPart::Lit(b'\n')),
                b't' => out.push(ReplPart::Lit(b'\t')),
                b'r' => out.push(ReplPart::Lit(b'\r')),
                _ => out.push(ReplPart::Lit(nx)),
            }
            i += 2;
        } else {
            out.push(ReplPart::Lit(c));
            i += 1;
        }
    }
    out
}

fn apply_repl(parts: &[ReplPart], text: &[u8], saves: &[Option<usize>], out: &mut Vec<u8>) {
    for p in parts {
        match p {
            ReplPart::Lit(b) => out.push(*b),
            ReplPart::Amp => {
                if let (Some(s), Some(e)) = (saves[0], saves[1]) {
                    out.extend_from_slice(&text[s..e]);
                }
            }
            ReplPart::Group(n) => {
                let i = 2 * n;
                if i + 1 < saves.len() {
                    if let (Some(s), Some(e)) = (saves[i], saves[i + 1]) {
                        out.extend_from_slice(&text[s..e]);
                    }
                }
            }
        }
    }
}

fn substitute(
    re: &Regex,
    repl: &[ReplPart],
    text: &[u8],
    global: bool,
    nth: usize,
) -> (Vec<u8>, bool) {
    let mut out: Vec<u8> = Vec::new();
    let len = text.len();
    let mut i = 0usize;
    let mut count = 0usize;
    let mut changed = false;
    loop {
        match re.search_from(text, i) {
            None => {
                out.extend_from_slice(&text[i..]);
                break;
            }
            Some(saves) => {
                let ms = saves[0].unwrap();
                let me = saves[1].unwrap();
                out.extend_from_slice(&text[i..ms]);
                count += 1;
                let do_it = if global { count >= nth } else { count == nth };
                if do_it {
                    apply_repl(repl, text, &saves, &mut out);
                    changed = true;
                } else {
                    out.extend_from_slice(&text[ms..me]);
                }
                if me == ms {
                    if me < len {
                        out.push(text[me]);
                        i = me + 1;
                    } else {
                        break;
                    }
                } else {
                    i = me;
                }
                if !global && count >= nth {
                    if i <= len {
                        out.extend_from_slice(&text[i..]);
                    }
                    break;
                }
                if i > len {
                    break;
                }
            }
        }
    }
    (out, changed)
}

// ---------------------------------------------------------------------------
// Commands / script parsing
// ---------------------------------------------------------------------------

enum Addr {
    Line(usize),
    Last,
    Rx(Regex),
}

enum Cmd {
    Subst {
        re: Regex,
        repl: Vec<ReplPart>,
        global: bool,
        nth: usize,
        print: bool,
    },
    Delete,
    Print,
    LineNum,
    Quit,
    Next,
    NextApp,
    Append(Vec<u8>),
    Insert(Vec<u8>),
    Change(Vec<u8>),
    Trans {
        from: Vec<u8>,
        to: Vec<u8>,
    },
    Branch(Option<Vec<u8>>),
    Test(Option<Vec<u8>>),
    Label(Vec<u8>),
    // Hold-space commands.
    Get,      // g: pattern <- hold
    GetApp,   // G: pattern <- pattern \n hold
    Hold,     // h: hold <- pattern
    HoldApp,  // H: hold <- hold \n pattern
    Exchange, // x: swap pattern and hold
    // `{ ... }` address-scoped grouping. `BlockStart` carries the index of its
    // matching `BlockEnd`; the enclosing address/negate live on the Command.
    BlockStart { end: usize },
    BlockEnd,
}

struct Command {
    a1: Option<Addr>,
    a2: Option<Addr>,
    negate: bool,
    active: bool,
    cmd: Cmd,
}

/// Read a delimited field starting at `*pos`, up to the next unescaped `delim`.
/// `\<delim>` becomes a literal delim; other escapes are preserved verbatim.
/// Advances `*pos` past the closing delimiter. Returns None if unterminated.
fn read_delim(b: &[u8], pos: &mut usize, delim: u8) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    while *pos < b.len() {
        let c = b[*pos];
        if c == delim {
            *pos += 1;
            return Some(out);
        }
        if c == b'\\' && *pos + 1 < b.len() {
            let nx = b[*pos + 1];
            if nx == delim {
                out.push(delim);
                *pos += 2;
                continue;
            }
            out.push(b'\\');
            out.push(nx);
            *pos += 2;
            continue;
        }
        out.push(c);
        *pos += 1;
    }
    None
}

/// Parse an optional address. `Ok(None)` = no address here; `Err(())` = a
/// syntactically-present address that is malformed (e.g. bad regex).
fn parse_addr(b: &[u8], pos: &mut usize, ere: bool) -> Result<Option<Addr>, ()> {
    while *pos < b.len() && (b[*pos] == b' ' || b[*pos] == b'\t') {
        *pos += 1;
    }
    if *pos >= b.len() {
        return Ok(None);
    }
    let c = b[*pos];
    if c.is_ascii_digit() {
        let mut n = 0usize;
        while *pos < b.len() && b[*pos].is_ascii_digit() {
            n = n.saturating_mul(10).saturating_add((b[*pos] - b'0') as usize);
            *pos += 1;
        }
        Ok(Some(Addr::Line(n)))
    } else if c == b'$' {
        *pos += 1;
        Ok(Some(Addr::Last))
    } else if c == b'/' {
        *pos += 1;
        let raw = read_delim(b, pos, b'/').ok_or(())?;
        Ok(Some(Addr::Rx(Regex::compile(&raw, ere, false)?)))
    } else {
        Ok(None)
    }
}

/// Read a/i/c text: skip a leading backslash and optional newline, then take the
/// remainder of the line, honoring backslash escapes and line continuations.
fn read_text(b: &[u8], pos: &mut usize) -> Vec<u8> {
    if *pos < b.len() && b[*pos] == b'\\' {
        *pos += 1;
    }
    if *pos < b.len() && b[*pos] == b'\n' {
        *pos += 1;
    }
    // Strip leading blanks of the first text line (GNU behavior).
    while *pos < b.len() && (b[*pos] == b' ' || b[*pos] == b'\t') {
        *pos += 1;
    }
    let mut text: Vec<u8> = Vec::new();
    while *pos < b.len() {
        let c = b[*pos];
        if c == b'\\' && *pos + 1 < b.len() {
            let nx = b[*pos + 1];
            if nx == b'\n' {
                text.push(b'\n');
                *pos += 2;
                continue;
            }
            text.push(nx);
            *pos += 2;
            continue;
        }
        if c == b'\n' {
            break;
        }
        text.push(c);
        *pos += 1;
    }
    text
}

fn read_label(b: &[u8], pos: &mut usize) -> Vec<u8> {
    while *pos < b.len() && (b[*pos] == b' ' || b[*pos] == b'\t') {
        *pos += 1;
    }
    let mut out: Vec<u8> = Vec::new();
    while *pos < b.len() && b[*pos] != b';' && b[*pos] != b'\n' {
        out.push(b[*pos]);
        *pos += 1;
    }
    while out.last() == Some(&b' ') || out.last() == Some(&b'\t') {
        out.pop();
    }
    out
}

fn parse_script(src: &[u8], ere: bool) -> Result<Vec<Command>, ()> {
    let mut cmds: Vec<Command> = Vec::new();
    // Stack of open `{` command indices, so `}` can back-patch the jump target.
    let mut block_stack: Vec<usize> = Vec::new();
    let mut pos = 0usize;
    let b = src;
    loop {
        // Skip separators / whitespace.
        while pos < b.len()
            && (b[pos] == b';' || b[pos] == b'\n' || b[pos] == b' ' || b[pos] == b'\t')
        {
            pos += 1;
        }
        if pos >= b.len() {
            break;
        }
        // Comment.
        if b[pos] == b'#' {
            while pos < b.len() && b[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }

        // Addresses.
        let a1 = parse_addr(b, &mut pos, ere)?;
        let mut a2 = None;
        if a1.is_some() {
            while pos < b.len() && (b[pos] == b' ' || b[pos] == b'\t') {
                pos += 1;
            }
            if pos < b.len() && b[pos] == b',' {
                pos += 1;
                a2 = parse_addr(b, &mut pos, ere)?;
            }
        }

        // Negation.
        let mut negate = false;
        loop {
            while pos < b.len() && (b[pos] == b' ' || b[pos] == b'\t') {
                pos += 1;
            }
            if pos < b.len() && b[pos] == b'!' {
                negate = !negate;
                pos += 1;
            } else {
                break;
            }
        }

        if pos >= b.len() {
            // A trailing address with no command is a syntax error.
            if a1.is_some() {
                return Err(());
            }
            break;
        }
        let ch = b[pos];
        pos += 1;

        // `{` opens an address-scoped block; `}` closes the most recent one.
        if ch == b'{' {
            block_stack.push(cmds.len());
            cmds.push(Command {
                a1,
                a2,
                negate,
                active: false,
                cmd: Cmd::BlockStart { end: 0 },
            });
            continue;
        }
        if ch == b'}' {
            // A close brace takes no address of its own.
            if a1.is_some() || negate {
                return Err(());
            }
            let start = block_stack.pop().ok_or(())?;
            let end_idx = cmds.len();
            if let Cmd::BlockStart { end } = &mut cmds[start].cmd {
                *end = end_idx;
            }
            cmds.push(Command {
                a1: None,
                a2: None,
                negate: false,
                active: false,
                cmd: Cmd::BlockEnd,
            });
            continue;
        }

        let cmd = match ch {
            b's' => {
                if pos >= b.len() {
                    return Err(());
                }
                let delim = b[pos];
                pos += 1;
                let pat = read_delim(b, &mut pos, delim).ok_or(())?;
                let rep = read_delim(b, &mut pos, delim).ok_or(())?;
                let mut global = false;
                let mut print = false;
                let mut icase = false;
                let mut nth = 0usize;
                while pos < b.len() {
                    let f = b[pos];
                    match f {
                        b'g' => {
                            global = true;
                            pos += 1;
                        }
                        b'p' => {
                            print = true;
                            pos += 1;
                        }
                        b'i' | b'I' => {
                            icase = true;
                            pos += 1;
                        }
                        b'0'..=b'9' => {
                            let mut n = 0usize;
                            while pos < b.len() && b[pos].is_ascii_digit() {
                                n = n.saturating_mul(10).saturating_add((b[pos] - b'0') as usize);
                                pos += 1;
                            }
                            nth = n;
                        }
                        _ => break,
                    }
                }
                if nth == 0 {
                    nth = 1;
                }
                Cmd::Subst {
                    re: Regex::compile(&pat, ere, icase)?,
                    repl: parse_repl(&rep),
                    global,
                    nth,
                    print,
                }
            }
            b'y' => {
                if pos >= b.len() {
                    return Err(());
                }
                let delim = b[pos];
                pos += 1;
                let from = read_delim(b, &mut pos, delim).ok_or(())?;
                let to = read_delim(b, &mut pos, delim).ok_or(())?;
                let from = unescape_y(&from);
                let to = unescape_y(&to);
                // POSIX: the two strings must have equal length.
                if from.len() != to.len() {
                    return Err(());
                }
                Cmd::Trans { from, to }
            }
            b'd' => Cmd::Delete,
            b'p' => Cmd::Print,
            b'=' => Cmd::LineNum,
            b'q' => Cmd::Quit,
            b'n' => Cmd::Next,
            b'N' => Cmd::NextApp,
            b'g' => Cmd::Get,
            b'G' => Cmd::GetApp,
            b'h' => Cmd::Hold,
            b'H' => Cmd::HoldApp,
            b'x' => Cmd::Exchange,
            b'a' => Cmd::Append(read_text(b, &mut pos)),
            b'i' => Cmd::Insert(read_text(b, &mut pos)),
            b'c' => Cmd::Change(read_text(b, &mut pos)),
            b'b' => {
                let l = read_label(b, &mut pos);
                Cmd::Branch(if l.is_empty() { None } else { Some(l) })
            }
            b't' => {
                let l = read_label(b, &mut pos);
                Cmd::Test(if l.is_empty() { None } else { Some(l) })
            }
            b':' => Cmd::Label(read_label(b, &mut pos)),
            _ => {
                // Genuinely unsupported command: hard error.
                return Err(());
            }
        };

        cmds.push(Command {
            a1,
            a2,
            negate,
            active: false,
            cmd,
        });
    }

    // Every `{` must have a matching `}`.
    if !block_stack.is_empty() {
        return Err(());
    }

    // Validate that every b/t target label is defined.
    for c in &cmds {
        let name = match &c.cmd {
            Cmd::Branch(n) => n,
            Cmd::Test(n) => n,
            _ => continue,
        };
        if let Some(target) = name {
            let mut found = false;
            for other in &cmds {
                if let Cmd::Label(l) = &other.cmd {
                    if l == target {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                return Err(());
            }
        }
    }

    Ok(cmds)
}

fn unescape_y(src: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < src.len() {
        if src[i] == b'\\' && i + 1 < src.len() {
            let nx = src[i + 1];
            out.push(match nx {
                b'n' => b'\n',
                b't' => b'\t',
                b'r' => b'\r',
                _ => nx,
            });
            i += 2;
        } else {
            out.push(src[i]);
            i += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

fn push_num(out: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        out.push(b'0');
        return;
    }
    let mut tmp = [0u8; 20];
    let mut i = 20;
    while n > 0 {
        i -= 1;
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    out.extend_from_slice(&tmp[i..]);
}

fn split_lines(data: &[u8]) -> Vec<Vec<u8>> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    for &c in data {
        if c == b'\n' {
            lines.push(core::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn addr_matches(a: &Addr, line_no: usize, is_last: bool, ps: &[u8]) -> bool {
    match a {
        Addr::Line(n) => *n == line_no,
        Addr::Last => is_last,
        Addr::Rx(r) => r.is_match(ps),
    }
}

/// Returns whether the command's address range currently selects this line,
/// updating range activation state.
fn compute_selection(cmd: &mut Command, line_no: usize, is_last: bool, ps: &[u8]) -> bool {
    if cmd.a1.is_none() {
        return true;
    }
    if cmd.a2.is_none() {
        return addr_matches(cmd.a1.as_ref().unwrap(), line_no, is_last, ps);
    }
    if cmd.active {
        let end = {
            let a2 = cmd.a2.as_ref().unwrap();
            match a2 {
                Addr::Line(n) => line_no >= *n,
                Addr::Last => is_last,
                Addr::Rx(r) => r.is_match(ps),
            }
        };
        if end {
            cmd.active = false;
        }
        true
    } else {
        let started = {
            let a1 = cmd.a1.as_ref().unwrap();
            addr_matches(a1, line_no, is_last, ps)
        };
        if started {
            let single = {
                let a2 = cmd.a2.as_ref().unwrap();
                matches!(a2, Addr::Line(n) if *n <= line_no)
            };
            if !single {
                cmd.active = true;
            }
            true
        } else {
            false
        }
    }
}

fn resolve_label(cmds: &[Command], name: &Option<Vec<u8>>) -> usize {
    match name {
        None => cmds.len(),
        Some(n) => {
            for (idx, c) in cmds.iter().enumerate() {
                if let Cmd::Label(l) = &c.cmd {
                    if l == n {
                        return idx;
                    }
                }
            }
            cmds.len()
        }
    }
}

fn run_program(lines: &[Vec<u8>], cmds: &mut [Command], autoprint: bool, out: &mut Vec<u8>) {
    for c in cmds.iter_mut() {
        // GNU `0,/re/` ranges are active from line 1, so their end address can
        // fire on the very first line. Mark them active up front.
        c.active = matches!(c.a1, Some(Addr::Line(0))) && c.a2.is_some();
    }
    let mut hold: Vec<u8> = Vec::new();
    let total = lines.len();
    let mut next_idx = 0usize;
    'outer: loop {
        if next_idx >= total {
            break;
        }
        let mut ps: Vec<u8> = lines[next_idx].clone();
        let mut line_no = next_idx + 1;
        let mut is_last = next_idx + 1 >= total;
        next_idx += 1;

        let mut append_q: Vec<Vec<u8>> = Vec::new();
        let mut deleted = false;
        let mut quit = false;
        let mut tflag = false;
        let mut pc = 0usize;

        'prog: loop {
            if pc >= cmds.len() {
                break;
            }
            let sel = compute_selection(&mut cmds[pc], line_no, is_last, &ps);

            // Address-scoped block: if the block's address does not select this
            // line, skip the entire block by jumping to its matching `}`.
            let block_end = if let Cmd::BlockStart { end } = &cmds[pc].cmd {
                Some(*end)
            } else {
                None
            };
            if let Some(end) = block_end {
                if sel == cmds[pc].negate {
                    pc = end; // BlockEnd is a no-op; the next iteration advances past it.
                } else {
                    pc += 1; // enter the block
                }
                continue 'prog;
            }

            if sel == cmds[pc].negate {
                pc += 1;
                continue;
            }

            let mut jump: Option<usize> = None;
            match &cmds[pc].cmd {
                Cmd::Print => {
                    out.extend_from_slice(&ps);
                    out.push(b'\n');
                }
                Cmd::LineNum => {
                    push_num(out, line_no as u64);
                    out.push(b'\n');
                }
                Cmd::Delete => {
                    deleted = true;
                    break 'prog;
                }
                Cmd::Quit => {
                    quit = true;
                    break 'prog;
                }
                Cmd::Subst {
                    re,
                    repl,
                    global,
                    nth,
                    print,
                } => {
                    let (newps, changed) = substitute(re, repl, &ps, *global, *nth);
                    if changed {
                        ps = newps;
                        tflag = true;
                        if *print {
                            out.extend_from_slice(&ps);
                            out.push(b'\n');
                        }
                    }
                }
                Cmd::Trans { from, to } => {
                    for byte in ps.iter_mut() {
                        if let Some(k) = from.iter().position(|&x| x == *byte) {
                            if k < to.len() {
                                *byte = to[k];
                            }
                        }
                    }
                }
                Cmd::Append(t) => {
                    let mut v = t.clone();
                    v.push(b'\n');
                    append_q.push(v);
                }
                Cmd::Insert(t) => {
                    out.extend_from_slice(t);
                    out.push(b'\n');
                }
                Cmd::Change(t) => {
                    deleted = true;
                    let in_middle = cmds[pc].a2.is_some() && cmds[pc].active;
                    if !in_middle {
                        out.extend_from_slice(t);
                        out.push(b'\n');
                    }
                    break 'prog;
                }
                Cmd::Next => {
                    if autoprint {
                        out.extend_from_slice(&ps);
                        out.push(b'\n');
                    }
                    if next_idx >= total {
                        deleted = true;
                        quit = true;
                        break 'prog;
                    }
                    ps = lines[next_idx].clone();
                    line_no = next_idx + 1;
                    next_idx += 1;
                    is_last = next_idx >= total;
                }
                Cmd::NextApp => {
                    if next_idx >= total {
                        quit = true;
                        break 'prog;
                    }
                    ps.push(b'\n');
                    ps.extend_from_slice(&lines[next_idx]);
                    line_no = next_idx + 1;
                    next_idx += 1;
                    is_last = next_idx >= total;
                }
                Cmd::Branch(name) => {
                    jump = Some(resolve_label(cmds, name));
                }
                Cmd::Test(name) => {
                    if tflag {
                        tflag = false;
                        jump = Some(resolve_label(cmds, name));
                    }
                }
                Cmd::Label(_) => {}
                Cmd::Get => {
                    ps = hold.clone();
                }
                Cmd::GetApp => {
                    ps.push(b'\n');
                    ps.extend_from_slice(&hold);
                }
                Cmd::Hold => {
                    hold = ps.clone();
                }
                Cmd::HoldApp => {
                    hold.push(b'\n');
                    hold.extend_from_slice(&ps);
                }
                Cmd::Exchange => {
                    core::mem::swap(&mut ps, &mut hold);
                }
                Cmd::BlockStart { .. } | Cmd::BlockEnd => {}
            }

            match jump {
                Some(t) => pc = t,
                None => pc += 1,
            }
        }

        if autoprint && !deleted {
            out.extend_from_slice(&ps);
            out.push(b'\n');
        }
        for a in &append_q {
            out.extend_from_slice(a);
        }
        if quit {
            break 'outer;
        }
    }
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

fn read_file(path: &[u8]) -> Option<Vec<u8>> {
    let fd = io::open(path, libc::O_RDONLY, 0);
    if fd < 0 {
        return None;
    }
    let v = io::read_all(fd);
    io::close(fd);
    Some(v)
}

/// `base` bytes followed by `extra` bytes (used to derive backup/temp paths).
fn join_bytes(base: &[u8], extra: &[u8]) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    v.extend_from_slice(base);
    v.extend_from_slice(extra);
    v
}

/// Create/truncate `path` and write `data` in full, then close.
/// Returns true only on a fully-successful write AND close.
fn write_file_all(path: &[u8], data: &[u8]) -> bool {
    let fd = io::open(path, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644);
    if fd < 0 {
        return false;
    }
    let n = io::write_all(fd, data);
    let c = io::close(fd);
    n == data.len() as isize && c == 0
}

/// sed - stream editor
///
/// # Synopsis
/// ```text
/// sed [-nrE] [-e script]... [-f file]... [-i] [script] [file...]
/// ```
pub fn sed(argc: i32, argv: *const *const u8) -> i32 {
    let mut scripts: Vec<u8> = Vec::new();
    let mut have_script_from_opt = false;
    let mut script_taken = false;
    let mut files: Vec<&[u8]> = Vec::new();

    let mut suppress = false;
    let mut ere = false;
    let mut inplace = false;
    let mut inplace_suffix: Option<Vec<u8>> = None;
    let mut exit_code = 0i32;
    let mut end_opts = false;

    let mut i = 1i32;
    while i < argc {
        let arg = match unsafe { get_arg(argv, i) } {
            Some(a) => a,
            None => {
                i += 1;
                continue;
            }
        };

        if !end_opts && arg.len() >= 2 && arg[0] == b'-' && arg != b"--" {
            let bytes = &arg[1..];
            let mut j = 0;
            while j < bytes.len() {
                match bytes[j] {
                    b'n' => suppress = true,
                    b'r' | b'E' => ere = true,
                    b'i' => {
                        inplace = true;
                        // A suffix may be attached directly to -i (e.g. `-i.bak`).
                        let rest = &bytes[j + 1..];
                        if !rest.is_empty() {
                            let mut s: Vec<u8> = Vec::new();
                            s.extend_from_slice(rest);
                            inplace_suffix = Some(s);
                        }
                        j = bytes.len();
                        break;
                    }
                    b'e' => {
                        let rest = &bytes[j + 1..];
                        if !rest.is_empty() {
                            scripts.extend_from_slice(rest);
                        } else {
                            i += 1;
                            if let Some(s) = unsafe { get_arg(argv, i) } {
                                scripts.extend_from_slice(s);
                            }
                        }
                        scripts.push(b'\n');
                        have_script_from_opt = true;
                        break;
                    }
                    b'f' => {
                        let rest = &bytes[j + 1..];
                        let path: Option<&[u8]> = if !rest.is_empty() {
                            Some(rest)
                        } else {
                            i += 1;
                            unsafe { get_arg(argv, i) }
                        };
                        if let Some(p) = path {
                            match read_file(p) {
                                Some(v) => {
                                    scripts.extend_from_slice(&v);
                                    scripts.push(b'\n');
                                }
                                None => {
                                    io::write_str(2, b"sed: can't read ");
                                    io::write_all(2, p);
                                    io::write_str(2, b": No such file or directory\n");
                                    return 1;
                                }
                            }
                        }
                        have_script_from_opt = true;
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }
            i += 1;
            continue;
        }

        if !end_opts && arg == b"--" {
            end_opts = true;
            i += 1;
            continue;
        }

        // Operand.
        if !have_script_from_opt && !script_taken {
            scripts.extend_from_slice(arg);
            script_taken = true;
        } else {
            files.push(arg);
        }
        i += 1;
    }

    if !have_script_from_opt && !script_taken {
        io::write_str(2, b"sed: no script specified\n");
        return 1;
    }

    let mut cmds = match parse_script(&scripts, ere) {
        Ok(c) => c,
        Err(()) => {
            io::write_str(2, b"sed: error in script\n");
            return 1;
        }
    };

    let autoprint = !suppress;

    if inplace {
        // In-place editing requires at least one named file operand.
        if files.is_empty() {
            io::write_str(2, b"sed: no input files for -i\n");
            return 1;
        }
        for f in &files {
            let data = match read_file(f) {
                Some(d) => d,
                None => {
                    io::write_str(2, b"sed: can't read ");
                    io::write_all(2, f);
                    io::write_str(2, b": No such file or directory\n");
                    exit_code = 2;
                    continue;
                }
            };
            let lines = split_lines(&data);
            let mut out: Vec<u8> = Vec::new();
            run_program(&lines, &mut cmds, autoprint, &mut out);

            // Backup (`-i.bak`): copy the original before touching it.
            if let Some(suf) = &inplace_suffix {
                let bpath = join_bytes(f, suf);
                if !write_file_all(&bpath, &data) {
                    io::write_str(2, b"sed: couldn't create backup ");
                    io::write_all(2, &bpath);
                    io::write_str(2, b"\n");
                    exit_code = 2;
                    continue; // leave the original untouched
                }
            }

            // Write to a temp file in the same directory, then rename over the
            // original only after a fully-successful write+close. On any error
            // the original file is left intact.
            let mut tmp = join_bytes(f, b".sed_tmp");
            push_num(&mut tmp, io::getpid() as u64);
            if !write_file_all(&tmp, &out) {
                io::write_str(2, b"sed: couldn't write temp file for ");
                io::write_all(2, f);
                io::write_str(2, b"\n");
                io::unlink(&tmp);
                exit_code = 2;
                continue;
            }
            if io::rename(&tmp, f) != 0 {
                io::write_str(2, b"sed: couldn't replace ");
                io::write_all(2, f);
                io::write_str(2, b"\n");
                io::unlink(&tmp);
                exit_code = 2;
                continue;
            }
        }
        return exit_code;
    }

    // Regular mode: concatenate all inputs into a single stream.
    let mut data: Vec<u8> = Vec::new();
    if files.is_empty() {
        data = io::read_all(0);
    } else {
        for f in &files {
            if *f == b"-" {
                let v = io::read_all(0);
                data.extend_from_slice(&v);
            } else {
                match read_file(f) {
                    Some(v) => data.extend_from_slice(&v),
                    None => {
                        io::write_str(2, b"sed: can't read ");
                        io::write_all(2, f);
                        io::write_str(2, b": No such file or directory\n");
                        exit_code = 2;
                    }
                }
            }
        }
    }

    let lines = split_lines(&data);
    let mut out: Vec<u8> = Vec::new();
    run_program(&lines, &mut cmds, autoprint, &mut out);
    let n = io::write_all(1, &out);
    if n != out.len() as isize {
        io::write_str(2, b"sed: write error\n");
        return 2;
    }

    exit_code
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::process::{Command, Stdio};
    use std::io::Write;
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
    fn test_sed_substitute() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/foo/bar/"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"foo baz foo\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "bar baz foo\n"); // Only first occurrence
    }

    #[test]
    fn test_sed_substitute_global() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/foo/bar/g"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"foo baz foo\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "bar baz bar\n"); // All occurrences
    }

    #[test]
    fn test_sed_multiple_lines() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/old/new/"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"old text\nmore old\nno match\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines, vec!["new text", "more new", "no match"]);
    }

    #[test]
    fn test_sed_no_match() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "s/notfound/replaced/"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"hello world\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "hello world\n"); // Unchanged
    }

    #[test]
    fn test_sed_with_e_flag() {
        let armybox = get_armybox_path();
        if !armybox.exists() { return; }

        let mut child = Command::new(&armybox)
            .args(["sed", "-e", "s/a/b/"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(b"aaa\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "baa\n");
    }
}
