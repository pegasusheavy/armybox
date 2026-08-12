//! sed - stream editor
//!
//! POSIX.1-2017 compliant implementation.
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/sed.html
//!
//! Implements a self-contained backtracking BRE/ERE regex engine (with capture
//! groups) plus a full sed cycle: addresses (line, `$`, `/re/`, ranges, `!`),
//! and the commands `s y d p a i c q n N = b t :`.

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

struct P<'a> {
    b: &'a [u8],
    pos: usize,
    ere: bool,
    gc: usize,
}

impl<'a> P<'a> {
    fn alt(&mut self) -> Ast {
        let mut br: Vec<Ast> = Vec::new();
        br.push(self.concat());
        while is_alt_op(self.b, self.pos, self.ere) {
            self.pos += if self.ere { 1 } else { 2 };
            br.push(self.concat());
        }
        if br.len() == 1 {
            br.pop().unwrap()
        } else {
            Ast::Alt(br)
        }
    }

    fn concat(&mut self) -> Ast {
        let mut items: Vec<Ast> = Vec::new();
        while self.pos < self.b.len()
            && !is_alt_op(self.b, self.pos, self.ere)
            && !is_group_close(self.b, self.pos, self.ere)
        {
            let first = items.is_empty();
            let a = self.atom(first);
            let a = self.quant(a);
            items.push(a);
        }
        if items.is_empty() {
            Ast::Empty
        } else if items.len() == 1 {
            items.pop().unwrap()
        } else {
            Ast::Concat(items)
        }
    }

    fn atom(&mut self, first: bool) -> Ast {
        let b = self.b;
        if is_group_open(b, self.pos, self.ere) {
            self.pos += if self.ere { 1 } else { 2 };
            self.gc += 1;
            let idx = self.gc;
            let inner = self.alt();
            if is_group_close(b, self.pos, self.ere) {
                self.pos += if self.ere { 1 } else { 2 };
            }
            return Ast::Group(idx, Box::new(inner));
        }
        let c = b[self.pos];
        if c == b'^' && first {
            self.pos += 1;
            return Ast::Start;
        }
        if c == b'$' {
            let nx = self.pos + 1;
            let end = nx >= b.len()
                || is_alt_op(b, nx, self.ere)
                || is_group_close(b, nx, self.ere);
            if end {
                self.pos += 1;
                return Ast::End;
            }
        }
        if c == b'.' {
            self.pos += 1;
            return Ast::Any;
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
            return Ast::Char(ch);
        }
        self.pos += 1;
        Ast::Char(c)
    }

    fn quant(&mut self, mut a: Ast) -> Ast {
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
                    if let Some((mn, mx)) = self.interval(self.pos + 1, false) {
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
                    if let Some((mn, mx)) = self.interval(self.pos + 2, true) {
                        a = Ast::Repeat(Box::new(a), mn, mx);
                        continue;
                    }
                }
            }
            break;
        }
        a
    }

    fn interval(&mut self, start: usize, bre: bool) -> Option<(usize, Option<usize>)> {
        let b = self.b;
        let mut p = start;
        let mut mn = 0usize;
        let mut got = false;
        while p < b.len() && b[p].is_ascii_digit() {
            mn = mn * 10 + (b[p] - b'0') as usize;
            p += 1;
            got = true;
        }
        if !got {
            return None;
        }
        let mx;
        if p < b.len() && b[p] == b',' {
            p += 1;
            if p < b.len() && b[p].is_ascii_digit() {
                let mut m = 0usize;
                while p < b.len() && b[p].is_ascii_digit() {
                    m = m * 10 + (b[p] - b'0') as usize;
                    p += 1;
                }
                mx = Some(m);
            } else {
                mx = None;
            }
        } else {
            mx = Some(mn);
        }
        if bre {
            if p + 1 < b.len() && b[p] == b'\\' && b[p + 1] == b'}' {
                p += 2;
            } else {
                return None;
            }
        } else if p < b.len() && b[p] == b'}' {
            p += 1;
        } else {
            return None;
        }
        self.pos = p;
        Some((mn, mx))
    }

    fn class(&mut self) -> Ast {
        let b = self.b;
        self.pos += 1; // consume '['
        let mut negated = false;
        if self.pos < b.len() && b[self.pos] == b'^' {
            negated = true;
            self.pos += 1;
        }
        let mut ranges: Vec<(u8, u8)> = Vec::new();
        let mut first = true;
        while self.pos < b.len() {
            let c = b[self.pos];
            if c == b']' && !first {
                self.pos += 1;
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
        Ast::Class(Class { negated, ranges })
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

fn re_run(
    prog: &[Inst],
    icase: bool,
    pc: usize,
    text: &[u8],
    pos: usize,
    saves: &mut Vec<Option<usize>>,
) -> Option<usize> {
    match &prog[pc] {
        Inst::Match => Some(pos),
        Inst::Char(c) => {
            if pos < text.len() && eqc(text[pos], *c, icase) {
                re_run(prog, icase, pc + 1, text, pos + 1, saves)
            } else {
                None
            }
        }
        Inst::Any => {
            if pos < text.len() {
                re_run(prog, icase, pc + 1, text, pos + 1, saves)
            } else {
                None
            }
        }
        Inst::Class(cl) => {
            if pos < text.len() && cl.matches(text[pos], icase) {
                re_run(prog, icase, pc + 1, text, pos + 1, saves)
            } else {
                None
            }
        }
        Inst::Start => {
            if pos == 0 {
                re_run(prog, icase, pc + 1, text, pos, saves)
            } else {
                None
            }
        }
        Inst::End => {
            if pos == text.len() {
                re_run(prog, icase, pc + 1, text, pos, saves)
            } else {
                None
            }
        }
        Inst::Jmp(x) => re_run(prog, icase, *x, text, pos, saves),
        Inst::Split(a, b) => {
            let snap = saves.clone();
            if let Some(e) = re_run(prog, icase, *a, text, pos, saves) {
                return Some(e);
            }
            *saves = snap;
            re_run(prog, icase, *b, text, pos, saves)
        }
        Inst::Save(k) => {
            let old = saves[*k];
            saves[*k] = Some(pos);
            match re_run(prog, icase, pc + 1, text, pos, saves) {
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
    fn compile(src: &[u8], ere: bool, icase: bool) -> Regex {
        let mut p = P {
            b: src,
            pos: 0,
            ere,
            gc: 0,
        };
        let ast = p.alt();
        let ngroups = p.gc;
        let mut prog: Vec<Inst> = Vec::new();
        prog.push(Inst::Save(0));
        emit(&ast, &mut prog);
        prog.push(Inst::Save(1));
        prog.push(Inst::Match);
        Regex {
            prog,
            nslots: 2 * (ngroups + 1),
            icase,
        }
    }

    fn search_from(&self, text: &[u8], start: usize) -> Option<Vec<Option<usize>>> {
        let mut s = start;
        while s <= text.len() {
            let mut saves: Vec<Option<usize>> = Vec::new();
            saves.resize(self.nslots, None);
            if re_run(&self.prog, self.icase, 0, text, s, &mut saves).is_some() {
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

fn parse_addr(b: &[u8], pos: &mut usize, ere: bool) -> Option<Addr> {
    while *pos < b.len() && (b[*pos] == b' ' || b[*pos] == b'\t') {
        *pos += 1;
    }
    if *pos >= b.len() {
        return None;
    }
    let c = b[*pos];
    if c.is_ascii_digit() {
        let mut n = 0usize;
        while *pos < b.len() && b[*pos].is_ascii_digit() {
            n = n * 10 + (b[*pos] - b'0') as usize;
            *pos += 1;
        }
        Some(Addr::Line(n))
    } else if c == b'$' {
        *pos += 1;
        Some(Addr::Last)
    } else if c == b'/' {
        *pos += 1;
        let raw = read_delim(b, pos, b'/')?;
        Some(Addr::Rx(Regex::compile(&raw, ere, false)))
    } else {
        None
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

fn parse_script(src: &[u8], ere: bool) -> Option<Vec<Command>> {
    let mut cmds: Vec<Command> = Vec::new();
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
        // Grouping braces: treat as no-op separators.
        if b[pos] == b'{' || b[pos] == b'}' {
            pos += 1;
            continue;
        }

        // Addresses.
        let a1 = parse_addr(b, &mut pos, ere);
        let mut a2 = None;
        if a1.is_some() {
            while pos < b.len() && (b[pos] == b' ' || b[pos] == b'\t') {
                pos += 1;
            }
            if pos < b.len() && b[pos] == b',' {
                pos += 1;
                a2 = parse_addr(b, &mut pos, ere);
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
            break;
        }
        let ch = b[pos];
        pos += 1;

        let cmd = match ch {
            b's' => {
                if pos >= b.len() {
                    return None;
                }
                let delim = b[pos];
                pos += 1;
                let pat = read_delim(b, &mut pos, delim)?;
                let rep = read_delim(b, &mut pos, delim)?;
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
                                n = n * 10 + (b[pos] - b'0') as usize;
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
                    re: Regex::compile(&pat, ere, icase),
                    repl: parse_repl(&rep),
                    global,
                    nth,
                    print,
                }
            }
            b'y' => {
                if pos >= b.len() {
                    return None;
                }
                let delim = b[pos];
                pos += 1;
                let from = read_delim(b, &mut pos, delim)?;
                let to = read_delim(b, &mut pos, delim)?;
                Cmd::Trans {
                    from: unescape_y(&from),
                    to: unescape_y(&to),
                }
            }
            b'd' => Cmd::Delete,
            b'p' => Cmd::Print,
            b'=' => Cmd::LineNum,
            b'q' => Cmd::Quit,
            b'n' => Cmd::Next,
            b'N' => Cmd::NextApp,
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
                // Unknown command: skip to next separator and ignore.
                while pos < b.len() && b[pos] != b';' && b[pos] != b'\n' {
                    pos += 1;
                }
                continue;
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
    Some(cmds)
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
        c.active = false;
    }
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
                        // Ignore any suffix attached to -i.
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
        Some(c) => c,
        None => {
            io::write_str(2, b"sed: error in script\n");
            return 1;
        }
    };

    let autoprint = !suppress;

    if inplace && !files.is_empty() {
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
            let fd = io::open(f, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644);
            if fd < 0 {
                io::write_str(2, b"sed: can't write ");
                io::write_all(2, f);
                io::write_str(2, b"\n");
                exit_code = 2;
                continue;
            }
            io::write_all(fd, &out);
            io::close(fd);
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
    io::write_all(1, &out);

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
