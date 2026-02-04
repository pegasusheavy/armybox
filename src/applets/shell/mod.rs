//! POSIX Shell implementation
//!
//! A minimal but functional shell supporting:
//! - Command execution
//! - Pipes (cmd1 | cmd2)
//! - Redirections (>, >>, <, 2>)
//! - Environment variables ($VAR)
//! - Variable assignment (VAR=value)
//! - Control structures (if/then/else/fi, for/do/done, while/do/done, case/esac)
//! - Command substitution $(cmd)
//! - Arithmetic expansion $((expr))
//! - Built-in commands (cd, exit, export, etc.)
//! - Script execution

use crate::io;
use crate::sys;
use super::get_arg;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// Shell state
#[cfg(feature = "alloc")]
struct Shell {
    /// Last exit status
    last_status: i32,
    /// Interactive mode
    interactive: bool,
    /// Should exit
    should_exit: bool,
    /// Exit code to return
    exit_code: i32,
    /// Local variables (not exported)
    variables: BTreeMap<Vec<u8>, Vec<u8>>,
}

#[cfg(feature = "alloc")]
impl Shell {
    fn new(interactive: bool) -> Self {
        Shell {
            last_status: 0,
            interactive,
            should_exit: false,
            exit_code: 0,
            variables: BTreeMap::new(),
        }
    }

    fn set_var(&mut self, name: &[u8], value: &[u8]) {
        self.variables.insert(name.to_vec(), value.to_vec());
    }

    fn get_var(&self, name: &[u8]) -> Option<&[u8]> {
        self.variables.get(name).map(|v| v.as_slice())
    }
}

/// Main shell entry point
pub fn sh(argc: i32, argv: *const *const u8) -> i32 {
    #[cfg(feature = "alloc")]
    {
        let mut script_file: Option<&[u8]> = None;
        let mut command_string: Option<&[u8]> = None;
        let mut login_shell = false;

        // Parse arguments
        let mut i = 1;
        while i < argc {
            let arg = match unsafe { get_arg(argv, i) } {
                Some(a) => a,
                None => { i += 1; continue; }
            };

            if arg == b"-c" {
                // Execute command string
                if i + 1 < argc {
                    command_string = unsafe { get_arg(argv, i + 1) };
                    i += 1;
                }
            } else if arg == b"-l" || arg == b"--login" {
                login_shell = true;
            } else if arg[0] != b'-' {
                // Script file
                script_file = Some(arg);
            }
            i += 1;
        }

        // If -l, source profile
        if login_shell {
            let profile = b"/etc/profile";
            let fd = io::open(profile, libc::O_RDONLY, 0);
            if fd >= 0 {
                let content = io::read_all(fd);
                io::close(fd);
                let mut shell = Shell::new(false);
                execute_script(&mut shell, &content);
            }
        }

        if let Some(cmd) = command_string {
            // Execute -c command
            let mut shell = Shell::new(false);
            execute_script(&mut shell, cmd);
            return shell.last_status;
        }

        if let Some(file) = script_file {
            // Execute script file
            let fd = io::open(file, libc::O_RDONLY, 0);
            if fd < 0 {
                io::write_str(2, b"sh: cannot open ");
                io::write_all(2, file);
                io::write_str(2, b"\n");
                return 127;
            }
            let content = io::read_all(fd);
            io::close(fd);

            let mut shell = Shell::new(false);
            execute_script(&mut shell, &content);
            return shell.last_status;
        }

        // Interactive mode
        let interactive = io::isatty(0);
        let mut shell = Shell::new(interactive);

        if interactive {
            io::write_str(1, b"ArmyBox sh\n");
        }

        interactive_loop(&mut shell);
        return shell.exit_code;
    }

    #[cfg(not(feature = "alloc"))]
    {
        // Minimal shell without alloc
        io::write_str(1, b"ArmyBox sh (minimal)\n");
        minimal_shell();
        0
    }
}

pub fn ash(argc: i32, argv: *const *const u8) -> i32 {
    sh(argc, argv)
}

pub fn dash(argc: i32, argv: *const *const u8) -> i32 {
    sh(argc, argv)
}

/// Minimal shell for no-alloc builds
#[cfg(not(feature = "alloc"))]
fn minimal_shell() {
    let mut line_buf = [0u8; 1024];
    let mut pos = 0;

    loop {
        if io::isatty(0) {
            io::write_str(1, b"$ ");
        }

        pos = 0;
        loop {
            let mut c = [0u8; 1];
            let n = io::read(0, &mut c);
            if n <= 0 {
                if pos == 0 { return; }
                break;
            }
            if c[0] == b'\n' { break; }
            if pos < line_buf.len() - 1 {
                line_buf[pos] = c[0];
                pos += 1;
            }
        }

        if pos == 0 { continue; }

        let line = &line_buf[..pos];
        if line == b"exit" { return; }

        // Handle cd
        if line.starts_with(b"cd ") {
            let path = &line[3..];
            let mut path_buf = [0u8; 256];
            let plen = core::cmp::min(path.len(), path_buf.len() - 1);
            path_buf[..plen].copy_from_slice(&path[..plen]);
            unsafe {
                if libc::chdir(path_buf.as_ptr() as *const i8) != 0 {
                    io::write_str(2, b"cd: failed\n");
                }
            }
            continue;
        }

        // Fork and exec
        let pid = io::fork();
        if pid == 0 {
            let mut args: [*const i8; 32] = [core::ptr::null(); 32];
            let mut arg_count = 0;
            let mut start = 0;
            let mut in_word = false;

            for i in 0..=pos {
                if i == pos || line_buf[i] == b' ' || line_buf[i] == b'\t' {
                    if in_word && arg_count < 31 {
                        line_buf[i] = 0;
                        args[arg_count] = line_buf[start..].as_ptr() as *const i8;
                        arg_count += 1;
                        in_word = false;
                    }
                } else if !in_word {
                    start = i;
                    in_word = true;
                }
            }

            if arg_count > 0 {
                unsafe { libc::execvp(args[0], args.as_ptr()); }
                io::write_str(2, b"sh: command not found\n");
            }
            io::exit(127);
        }

        let mut status: i32 = 0;
        io::waitpid(pid, &mut status, 0);
    }
}

/// Interactive shell loop
#[cfg(feature = "alloc")]
fn interactive_loop(shell: &mut Shell) {
    let mut line_buf = Vec::new();

    loop {
        if shell.should_exit { return; }

        if shell.interactive {
            io::write_str(1, b"$ ");
        }

        line_buf.clear();
        loop {
            let mut c = [0u8; 1];
            let n = io::read(0, &mut c);
            if n <= 0 {
                if line_buf.is_empty() {
                    shell.should_exit = true;
                    return;
                }
                break;
            }
            if c[0] == b'\n' { break; }
            line_buf.push(c[0]);
        }

        if line_buf.is_empty() { continue; }

        execute_script(shell, &line_buf);
    }
}

/// Execute a script (multiple lines/statements)
#[cfg(feature = "alloc")]
fn execute_script(shell: &mut Shell, script: &[u8]) {
    let mut pos = 0;
    while pos < script.len() && !shell.should_exit {
        pos = execute_statement(shell, script, pos);
    }
}

/// Execute a single statement, return position after it
#[cfg(feature = "alloc")]
fn execute_statement(shell: &mut Shell, script: &[u8], start: usize) -> usize {
    let mut pos = skip_whitespace_and_comments(script, start);
    if pos >= script.len() { return pos; }

    // Check for control structures
    let word_end = find_word_end(script, pos);
    let first_word = &script[pos..word_end];

    if first_word == b"if" {
        return execute_if(shell, script, pos);
    }
    if first_word == b"while" {
        return execute_while(shell, script, pos);
    }
    if first_word == b"until" {
        return execute_until(shell, script, pos);
    }
    if first_word == b"for" {
        return execute_for(shell, script, pos);
    }
    if first_word == b"case" {
        return execute_case(shell, script, pos);
    }

    // Regular command or pipeline
    execute_simple_line(shell, script, pos)
}

/// Skip whitespace, semicolons, and comments, return new position
#[cfg(feature = "alloc")]
fn skip_whitespace_and_comments(script: &[u8], start: usize) -> usize {
    let mut pos = start;
    while pos < script.len() {
        let c = script[pos];
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' || c == b';' {
            pos += 1;
        } else if c == b'#' {
            // Skip to end of line
            while pos < script.len() && script[pos] != b'\n' {
                pos += 1;
            }
        } else {
            break;
        }
    }
    pos
}

/// Find end of word (alphanumeric + underscore)
#[cfg(feature = "alloc")]
fn find_word_end(script: &[u8], start: usize) -> usize {
    let mut pos = start;
    while pos < script.len() {
        let c = script[pos];
        if c.is_ascii_alphanumeric() || c == b'_' {
            pos += 1;
        } else {
            break;
        }
    }
    pos
}

/// Execute if/then/else/elif/fi
#[cfg(feature = "alloc")]
fn execute_if(shell: &mut Shell, script: &[u8], start: usize) -> usize {
    let mut pos = start + 2; // skip "if"

    // Find "then"
    let then_pos = find_keyword(script, pos, b"then");
    if then_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'then'\n");
        return script.len();
    }
    let then_pos = then_pos.unwrap();

    // Execute condition
    let condition = &script[pos..then_pos];
    execute_script(shell, condition.trim_ascii());
    let condition_result = shell.last_status == 0;

    pos = then_pos + 4; // skip "then"

    // Find matching fi, tracking elif/else
    let mut depth = 1;
    let mut else_pos: Option<usize> = None;
    let mut fi_pos: Option<usize> = None;
    let mut scan = pos;

    while scan < script.len() && depth > 0 {
        scan = skip_whitespace_and_comments(script, scan);
        if scan >= script.len() { break; }

        let word_end = find_word_end(script, scan);
        let word = &script[scan..word_end];

        if word == b"if" {
            depth += 1;
            scan = word_end;
        } else if word == b"fi" {
            depth -= 1;
            if depth == 0 {
                fi_pos = Some(scan);
            }
            scan = word_end;
        } else if word == b"else" && depth == 1 {
            else_pos = Some(scan);
            scan = word_end;
        } else if word == b"elif" && depth == 1 {
            // Treat elif as else + nested if
            else_pos = Some(scan);
            scan = word_end;
        } else {
            scan = skip_to_next_token(script, scan);
        }
    }

    if fi_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'fi'\n");
        return script.len();
    }
    let fi_pos = fi_pos.unwrap();

    if condition_result {
        // Execute then branch
        let end = else_pos.unwrap_or(fi_pos);
        let then_body = &script[pos..end];
        execute_script(shell, then_body);
    } else if let Some(else_start) = else_pos {
        // Execute else branch
        let else_body = &script[else_start + 4..fi_pos]; // skip "else"

        // Check if it's elif
        let trimmed = else_body.trim_ascii();
        if trimmed.starts_with(b"if ") || trimmed == b"if" {
            // It's elif - recursively handle
            execute_script(shell, trimmed);
        } else {
            execute_script(shell, else_body);
        }
    }

    fi_pos + 2 // skip "fi"
}

/// Execute while/do/done
#[cfg(feature = "alloc")]
fn execute_while(shell: &mut Shell, script: &[u8], start: usize) -> usize {
    let mut pos = start + 5; // skip "while"

    // Find "do"
    let do_pos = find_keyword(script, pos, b"do");
    if do_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'do'\n");
        return script.len();
    }
    let do_pos = do_pos.unwrap();
    let condition = &script[pos..do_pos];

    pos = do_pos + 2; // skip "do"

    // Find matching "done"
    let done_pos = find_matching_done(script, pos);
    if done_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'done'\n");
        return script.len();
    }
    let done_pos = done_pos.unwrap();

    let body = &script[pos..done_pos];

    // Execute while loop
    loop {
        execute_script(shell, condition.trim_ascii());
        if shell.last_status != 0 { break; }
        execute_script(shell, body);
        if shell.should_exit { break; }
    }

    done_pos + 4 // skip "done"
}

/// Execute until/do/done (opposite of while)
#[cfg(feature = "alloc")]
fn execute_until(shell: &mut Shell, script: &[u8], start: usize) -> usize {
    let mut pos = start + 5; // skip "until"

    let do_pos = find_keyword(script, pos, b"do");
    if do_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'do'\n");
        return script.len();
    }
    let do_pos = do_pos.unwrap();
    let condition = &script[pos..do_pos];

    pos = do_pos + 2;

    let done_pos = find_matching_done(script, pos);
    if done_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'done'\n");
        return script.len();
    }
    let done_pos = done_pos.unwrap();

    let body = &script[pos..done_pos];

    loop {
        execute_script(shell, condition.trim_ascii());
        if shell.last_status == 0 { break; } // opposite of while
        execute_script(shell, body);
        if shell.should_exit { break; }
    }

    done_pos + 4
}

/// Execute for/in/do/done
#[cfg(feature = "alloc")]
fn execute_for(shell: &mut Shell, script: &[u8], start: usize) -> usize {
    let mut pos = start + 3; // skip "for"
    pos = skip_whitespace_and_comments(script, pos);

    // Get variable name
    let var_end = find_word_end(script, pos);
    let var_name = script[pos..var_end].to_vec();
    pos = var_end;

    pos = skip_whitespace_and_comments(script, pos);

    // Check for "in"
    if !script[pos..].starts_with(b"in") {
        io::write_str(2, b"sh: syntax error: expected 'in'\n");
        return script.len();
    }
    pos += 2;

    // Get word list (until "do" or newline/semicolon)
    let do_pos = find_keyword(script, pos, b"do");
    if do_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'do'\n");
        return script.len();
    }
    let do_pos = do_pos.unwrap();

    let words_str = &script[pos..do_pos];
    let words = split_words(shell, words_str);

    pos = do_pos + 2;

    let done_pos = find_matching_done(script, pos);
    if done_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'done'\n");
        return script.len();
    }
    let done_pos = done_pos.unwrap();

    let body = &script[pos..done_pos];

    // Execute for each word
    for word in words {
        shell.set_var(&var_name, &word);
        execute_script(shell, body);
        if shell.should_exit { break; }
    }

    done_pos + 4
}

/// Execute case/in/esac
#[cfg(feature = "alloc")]
fn execute_case(shell: &mut Shell, script: &[u8], start: usize) -> usize {
    let mut pos = start + 4; // skip "case"
    pos = skip_whitespace_and_comments(script, pos);

    // Get the word to match
    let (match_word, new_pos) = parse_word(shell, script, pos);
    pos = new_pos;

    pos = skip_whitespace_and_comments(script, pos);

    // Expect "in"
    if !script[pos..].starts_with(b"in") {
        io::write_str(2, b"sh: syntax error: expected 'in'\n");
        return script.len();
    }
    pos += 2;

    // Find esac
    let esac_pos = find_keyword(script, pos, b"esac");
    if esac_pos.is_none() {
        io::write_str(2, b"sh: syntax error: expected 'esac'\n");
        return script.len();
    }
    let esac_pos = esac_pos.unwrap();

    // Parse and execute case patterns
    let mut matched = false;
    while pos < esac_pos && !matched {
        pos = skip_whitespace_and_comments(script, pos);
        if pos >= esac_pos { break; }

        // Find pattern(s) ending with )
        let paren_pos = script[pos..esac_pos].iter().position(|&c| c == b')');
        if paren_pos.is_none() { break; }
        let paren_pos = pos + paren_pos.unwrap();

        let patterns = &script[pos..paren_pos];
        pos = paren_pos + 1;

        // Find ;; or esac
        let end_pos = find_case_end(script, pos, esac_pos);
        let body = &script[pos..end_pos];

        // Check if any pattern matches
        for pattern in patterns.split(|&c| c == b'|') {
            let pattern = pattern.trim_ascii();
            if pattern_matches(&match_word, pattern) {
                execute_script(shell, body);
                matched = true;
                break;
            }
        }

        pos = end_pos;
        if script[pos..].starts_with(b";;") {
            pos += 2;
        }
    }

    esac_pos + 4
}

/// Check if word matches a shell pattern (supports * and ?)
#[cfg(feature = "alloc")]
fn pattern_matches(word: &[u8], pattern: &[u8]) -> bool {
    if pattern == b"*" { return true; }

    let mut wi = 0;
    let mut pi = 0;
    let mut star_pi: Option<usize> = None;
    let mut star_wi: usize = 0;

    while wi < word.len() {
        if pi < pattern.len() && (pattern[pi] == b'?' || pattern[pi] == word[wi]) {
            wi += 1;
            pi += 1;
        } else if pi < pattern.len() && pattern[pi] == b'*' {
            star_pi = Some(pi);
            star_wi = wi;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_wi += 1;
            wi = star_wi;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }

    pi == pattern.len()
}

/// Find end of case branch (;; or esac)
#[cfg(feature = "alloc")]
fn find_case_end(script: &[u8], start: usize, esac_pos: usize) -> usize {
    let mut pos = start;
    while pos < esac_pos {
        if script[pos..].starts_with(b";;") {
            return pos;
        }
        pos += 1;
    }
    esac_pos
}

/// Find a keyword at word boundary
#[cfg(feature = "alloc")]
fn find_keyword(script: &[u8], start: usize, keyword: &[u8]) -> Option<usize> {
    let mut pos = start;
    while pos < script.len() {
        pos = skip_whitespace_and_comments(script, pos);
        if pos >= script.len() { return None; }

        let word_end = find_word_end(script, pos);
        if &script[pos..word_end] == keyword {
            return Some(pos);
        }

        pos = skip_to_next_token(script, pos);
    }
    None
}

/// Find matching done for while/for/until
#[cfg(feature = "alloc")]
fn find_matching_done(script: &[u8], start: usize) -> Option<usize> {
    let mut pos = start;
    let mut depth = 1;

    while pos < script.len() && depth > 0 {
        pos = skip_whitespace_and_comments(script, pos);
        if pos >= script.len() { return None; }

        let word_end = find_word_end(script, pos);
        let word = &script[pos..word_end];

        if word == b"while" || word == b"for" || word == b"until" {
            depth += 1;
        } else if word == b"done" {
            depth -= 1;
            if depth == 0 {
                return Some(pos);
            }
        }

        pos = skip_to_next_token(script, pos);
    }
    None
}

/// Skip to next token
#[cfg(feature = "alloc")]
fn skip_to_next_token(script: &[u8], start: usize) -> usize {
    let mut pos = start;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while pos < script.len() {
        let c = script[pos];

        if in_single_quote {
            if c == b'\'' { in_single_quote = false; }
            pos += 1;
        } else if in_double_quote {
            if c == b'"' { in_double_quote = false; }
            else if c == b'\\' && pos + 1 < script.len() { pos += 1; }
            pos += 1;
        } else if c == b'\'' {
            in_single_quote = true;
            pos += 1;
        } else if c == b'"' {
            in_double_quote = true;
            pos += 1;
        } else if c == b' ' || c == b'\t' || c == b'\n' || c == b';' {
            break;
        } else {
            pos += 1;
        }
    }
    pos
}

/// Execute a simple line (no control structures)
#[cfg(feature = "alloc")]
fn execute_simple_line(shell: &mut Shell, script: &[u8], start: usize) -> usize {
    let mut pos = start;

    // Find end of line/statement
    let mut end = pos;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while end < script.len() {
        let c = script[end];
        if in_single_quote {
            if c == b'\'' { in_single_quote = false; }
            end += 1;
        } else if in_double_quote {
            if c == b'"' { in_double_quote = false; }
            else if c == b'\\' && end + 1 < script.len() { end += 1; }
            end += 1;
        } else if c == b'\'' {
            in_single_quote = true;
            end += 1;
        } else if c == b'"' {
            in_double_quote = true;
            end += 1;
        } else if c == b'\n' || c == b';' {
            break;
        } else {
            end += 1;
        }
    }

    let line = &script[pos..end];
    if !line.trim_ascii().is_empty() {
        execute_line(shell, line);
    }

    if end < script.len() && (script[end] == b'\n' || script[end] == b';') {
        end + 1
    } else {
        end
    }
}

/// Execute a single command line (may contain pipes)
#[cfg(feature = "alloc")]
fn execute_line(shell: &mut Shell, line: &[u8]) {
    let line = line.trim_ascii();
    if line.is_empty() || line[0] == b'#' { return; }

    // Check for variable assignment: VAR=value or VAR=value cmd...
    if let Some(eq_pos) = line.iter().position(|&c| c == b'=') {
        let before_eq = &line[..eq_pos];
        // Check if it's a valid variable name (starts with letter/underscore, contains only alnum/_)
        if !before_eq.is_empty()
            && (before_eq[0].is_ascii_alphabetic() || before_eq[0] == b'_')
            && before_eq.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_')
        {
            // Find end of value (space or end of line)
            let after_eq = &line[eq_pos + 1..];
            let (value, rest) = parse_assignment_value(after_eq);
            let expanded_value = expand_string(shell, &value);

            if rest.trim_ascii().is_empty() {
                // Just assignment, no command
                shell.set_var(before_eq, &expanded_value);
                shell.last_status = 0;
                return;
            } else {
                // Assignment before command - set in environment for child
                shell.set_var(before_eq, &expanded_value);
                // Continue to execute rest as command
                execute_line(shell, rest.trim_ascii());
                return;
            }
        }
    }

    // Parse into pipeline
    let tokens = tokenize(shell, line);
    if tokens.is_empty() { return; }

    execute_pipeline(shell, &tokens);
}

/// Parse value part of assignment, return (value, rest)
#[cfg(feature = "alloc")]
fn parse_assignment_value(s: &[u8]) -> (Vec<u8>, &[u8]) {
    let mut value = Vec::new();
    let mut pos = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut paren_depth = 0;

    while pos < s.len() {
        let c = s[pos];

        if in_single_quote {
            if c == b'\'' {
                in_single_quote = false;
            } else {
                value.push(c);
            }
            pos += 1;
        } else if in_double_quote {
            if c == b'"' {
                in_double_quote = false;
            } else if c == b'\\' && pos + 1 < s.len() {
                pos += 1;
                value.push(s[pos]);
            } else {
                value.push(c);
            }
            pos += 1;
        } else if c == b'\'' {
            in_single_quote = true;
            pos += 1;
        } else if c == b'"' {
            in_double_quote = true;
            pos += 1;
        } else if c == b'(' {
            paren_depth += 1;
            value.push(c);
            pos += 1;
        } else if c == b')' {
            if paren_depth > 0 { paren_depth -= 1; }
            value.push(c);
            pos += 1;
        } else if (c == b' ' || c == b'\t') && paren_depth == 0 {
            break;
        } else {
            value.push(c);
            pos += 1;
        }
    }

    (value, &s[pos..])
}

/// Token types
#[cfg(feature = "alloc")]
#[derive(Clone, PartialEq)]
enum Token {
    Word(Vec<u8>),
    Pipe,
    AndIf,      // &&
    OrIf,       // ||
    RedirectOut,
    RedirectAppend,
    RedirectIn,
    RedirectErr,
    Background,
}

/// Command structure
#[cfg(feature = "alloc")]
struct Command {
    args: Vec<Vec<u8>>,
    stdin_file: Option<Vec<u8>>,
    stdout_file: Option<Vec<u8>>,
    stdout_append: bool,
    stderr_file: Option<Vec<u8>>,
    background: bool,
}

#[cfg(feature = "alloc")]
impl Command {
    fn new() -> Self {
        Command {
            args: Vec::new(),
            stdin_file: None,
            stdout_file: None,
            stdout_append: false,
            stderr_file: None,
            background: false,
        }
    }
}

/// Tokenize input
#[cfg(feature = "alloc")]
fn tokenize(shell: &Shell, input: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        let c = input[pos];

        if c == b' ' || c == b'\t' {
            pos += 1;
            continue;
        }

        if c == b'#' { break; }

        if c == b'|' {
            if pos + 1 < input.len() && input[pos + 1] == b'|' {
                tokens.push(Token::OrIf);
                pos += 2;
            } else {
                tokens.push(Token::Pipe);
                pos += 1;
            }
            continue;
        }

        if c == b'&' {
            if pos + 1 < input.len() && input[pos + 1] == b'&' {
                tokens.push(Token::AndIf);
                pos += 2;
            } else {
                tokens.push(Token::Background);
                pos += 1;
            }
            continue;
        }

        if c == b'>' {
            if pos + 1 < input.len() && input[pos + 1] == b'>' {
                tokens.push(Token::RedirectAppend);
                pos += 2;
            } else {
                tokens.push(Token::RedirectOut);
                pos += 1;
            }
            continue;
        }

        if c == b'<' {
            tokens.push(Token::RedirectIn);
            pos += 1;
            continue;
        }

        if c == b'2' && pos + 1 < input.len() && input[pos + 1] == b'>' {
            tokens.push(Token::RedirectErr);
            pos += 2;
            continue;
        }

        // Parse word
        let (word, new_pos) = parse_word(shell, input, pos);
        if !word.is_empty() {
            tokens.push(Token::Word(word));
        }
        pos = new_pos;
    }

    tokens
}

/// Parse a word (handling quotes and expansions)
#[cfg(feature = "alloc")]
fn parse_word(shell: &Shell, input: &[u8], start: usize) -> (Vec<u8>, usize) {
    let mut word = Vec::new();
    let mut pos = start;

    while pos < input.len() {
        let c = input[pos];

        if c == b' ' || c == b'\t' || c == b'\n' || c == b';' || c == b'|' || c == b'&' || c == b'>' || c == b'<' || c == b'#' {
            break;
        }

        if c == b'\'' {
            // Single quote - literal
            pos += 1;
            while pos < input.len() && input[pos] != b'\'' {
                word.push(input[pos]);
                pos += 1;
            }
            if pos < input.len() { pos += 1; }
        } else if c == b'"' {
            // Double quote - allow expansions
            pos += 1;
            while pos < input.len() && input[pos] != b'"' {
                if input[pos] == b'\\' && pos + 1 < input.len() {
                    pos += 1;
                    word.push(input[pos]);
                    pos += 1;
                } else if input[pos] == b'$' {
                    let (expanded, new_pos) = expand_dollar(shell, input, pos);
                    word.extend_from_slice(&expanded);
                    pos = new_pos;
                } else {
                    word.push(input[pos]);
                    pos += 1;
                }
            }
            if pos < input.len() { pos += 1; }
        } else if c == b'\\' && pos + 1 < input.len() {
            pos += 1;
            word.push(input[pos]);
            pos += 1;
        } else if c == b'$' {
            let (expanded, new_pos) = expand_dollar(shell, input, pos);
            word.extend_from_slice(&expanded);
            pos = new_pos;
        } else {
            word.push(c);
            pos += 1;
        }
    }

    (word, pos)
}

/// Expand $ expressions
#[cfg(feature = "alloc")]
fn expand_dollar(shell: &Shell, input: &[u8], start: usize) -> (Vec<u8>, usize) {
    let mut pos = start + 1; // skip $

    if pos >= input.len() {
        return (b"$".to_vec(), pos);
    }

    let c = input[pos];

    // $?
    if c == b'?' {
        let mut buf = [0u8; 16];
        let s = format_number(shell.last_status as u64, &mut buf);
        return (s.to_vec(), pos + 1);
    }

    // $$
    if c == b'$' {
        let pid = unsafe { libc::getpid() };
        let mut buf = [0u8; 16];
        let s = format_number(pid as u64, &mut buf);
        return (s.to_vec(), pos + 1);
    }

    // $((arithmetic))
    if c == b'(' && pos + 1 < input.len() && input[pos + 1] == b'(' {
        pos += 2;
        let start_expr = pos;
        let mut depth = 2;
        while pos < input.len() && depth > 0 {
            if input[pos] == b'(' { depth += 1; }
            else if input[pos] == b')' { depth -= 1; }
            pos += 1;
        }
        let expr = &input[start_expr..pos - 2];
        let result = eval_arithmetic(shell, expr);
        let mut buf = [0u8; 20];
        let s = format_signed(result, &mut buf);
        return (s.to_vec(), pos);
    }

    // $(command)
    if c == b'(' {
        pos += 1;
        let start_cmd = pos;
        let mut depth = 1;
        while pos < input.len() && depth > 0 {
            if input[pos] == b'(' { depth += 1; }
            else if input[pos] == b')' { depth -= 1; }
            pos += 1;
        }
        let cmd = &input[start_cmd..pos - 1];
        let output = execute_capture(shell, cmd);
        // Trim trailing newlines
        let output = output.trim_ascii_end().to_vec();
        return (output, pos);
    }

    // ${VAR} or $VAR
    let mut var_name = Vec::new();
    if c == b'{' {
        pos += 1;
        while pos < input.len() && input[pos] != b'}' {
            var_name.push(input[pos]);
            pos += 1;
        }
        if pos < input.len() { pos += 1; }
    } else {
        while pos < input.len() && (input[pos].is_ascii_alphanumeric() || input[pos] == b'_') {
            var_name.push(input[pos]);
            pos += 1;
        }
    }

    if var_name.is_empty() {
        return (b"$".to_vec(), start + 1);
    }

    // Look up variable
    if let Some(value) = shell.get_var(&var_name) {
        return (value.to_vec(), pos);
    }
    if let Some(value) = io::getenv(&var_name) {
        return (value.to_vec(), pos);
    }

    (Vec::new(), pos)
}

/// Expand all $ in a string
#[cfg(feature = "alloc")]
fn expand_string(shell: &Shell, input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        if input[pos] == b'$' {
            let (expanded, new_pos) = expand_dollar(shell, input, pos);
            result.extend_from_slice(&expanded);
            pos = new_pos;
        } else {
            result.push(input[pos]);
            pos += 1;
        }
    }

    result
}

/// Evaluate arithmetic expression
#[cfg(feature = "alloc")]
fn eval_arithmetic(shell: &Shell, expr: &[u8]) -> i64 {
    // Expand $variables first
    let expanded = expand_string(shell, expr);
    // Then evaluate, passing shell for bare variable names
    eval_arith_expr_with_shell(shell, &expanded, 0).0
}

/// Arithmetic parser with shell variable lookup for bare names
#[cfg(feature = "alloc")]
fn eval_arith_expr_with_shell(shell: &Shell, expr: &[u8], pos: usize) -> (i64, usize) {
    let (mut left, mut pos) = eval_arith_term_with_shell(shell, expr, pos);

    loop {
        pos = skip_arith_ws(expr, pos);
        if pos >= expr.len() { break; }

        let op = expr[pos];
        if op == b'+' {
            let (right, new_pos) = eval_arith_term_with_shell(shell, expr, pos + 1);
            left += right;
            pos = new_pos;
        } else if op == b'-' {
            let (right, new_pos) = eval_arith_term_with_shell(shell, expr, pos + 1);
            left -= right;
            pos = new_pos;
        } else {
            break;
        }
    }

    (left, pos)
}

#[cfg(feature = "alloc")]
fn eval_arith_term_with_shell(shell: &Shell, expr: &[u8], pos: usize) -> (i64, usize) {
    let (mut left, mut pos) = eval_arith_factor_with_shell(shell, expr, pos);

    loop {
        pos = skip_arith_ws(expr, pos);
        if pos >= expr.len() { break; }

        let op = expr[pos];
        if op == b'*' {
            let (right, new_pos) = eval_arith_factor_with_shell(shell, expr, pos + 1);
            left *= right;
            pos = new_pos;
        } else if op == b'/' {
            let (right, new_pos) = eval_arith_factor_with_shell(shell, expr, pos + 1);
            if right != 0 { left /= right; }
            pos = new_pos;
        } else if op == b'%' {
            let (right, new_pos) = eval_arith_factor_with_shell(shell, expr, pos + 1);
            if right != 0 { left %= right; }
            pos = new_pos;
        } else {
            break;
        }
    }

    (left, pos)
}

#[cfg(feature = "alloc")]
fn eval_arith_factor_with_shell(shell: &Shell, expr: &[u8], pos: usize) -> (i64, usize) {
    let mut pos = skip_arith_ws(expr, pos);
    if pos >= expr.len() { return (0, pos); }

    // Handle parentheses
    if expr[pos] == b'(' {
        let (val, new_pos) = eval_arith_expr_with_shell(shell, expr, pos + 1);
        let mut pos = skip_arith_ws(expr, new_pos);
        if pos < expr.len() && expr[pos] == b')' { pos += 1; }
        return (val, pos);
    }

    // Handle negative
    if expr[pos] == b'-' {
        let (val, new_pos) = eval_arith_factor_with_shell(shell, expr, pos + 1);
        return (-val, new_pos);
    }

    // Parse number
    if expr[pos] >= b'0' && expr[pos] <= b'9' {
        let mut num: i64 = 0;
        while pos < expr.len() && expr[pos] >= b'0' && expr[pos] <= b'9' {
            num = num * 10 + (expr[pos] - b'0') as i64;
            pos += 1;
        }
        return (num, pos);
    }

    // Parse bare variable name
    if expr[pos].is_ascii_alphabetic() || expr[pos] == b'_' {
        let start = pos;
        while pos < expr.len() && (expr[pos].is_ascii_alphanumeric() || expr[pos] == b'_') {
            pos += 1;
        }
        let var_name = &expr[start..pos];
        // Look up variable and parse as number
        if let Some(value) = shell.get_var(var_name) {
            let num = sys::parse_i64(&value).unwrap_or(0);
            return (num, pos);
        }
        return (0, pos);
    }

    (0, pos)
}

/// Simple arithmetic parser
#[cfg(feature = "alloc")]
fn eval_arith_expr(expr: &[u8], pos: usize) -> (i64, usize) {
    let (mut left, mut pos) = eval_arith_term(expr, pos);

    loop {
        pos = skip_arith_ws(expr, pos);
        if pos >= expr.len() { break; }

        let op = expr[pos];
        if op == b'+' {
            let (right, new_pos) = eval_arith_term(expr, pos + 1);
            left += right;
            pos = new_pos;
        } else if op == b'-' {
            let (right, new_pos) = eval_arith_term(expr, pos + 1);
            left -= right;
            pos = new_pos;
        } else {
            break;
        }
    }

    (left, pos)
}

#[cfg(feature = "alloc")]
fn eval_arith_term(expr: &[u8], pos: usize) -> (i64, usize) {
    let (mut left, mut pos) = eval_arith_factor(expr, pos);

    loop {
        pos = skip_arith_ws(expr, pos);
        if pos >= expr.len() { break; }

        let op = expr[pos];
        if op == b'*' {
            let (right, new_pos) = eval_arith_factor(expr, pos + 1);
            left *= right;
            pos = new_pos;
        } else if op == b'/' {
            let (right, new_pos) = eval_arith_factor(expr, pos + 1);
            if right != 0 { left /= right; }
            pos = new_pos;
        } else if op == b'%' {
            let (right, new_pos) = eval_arith_factor(expr, pos + 1);
            if right != 0 { left %= right; }
            pos = new_pos;
        } else {
            break;
        }
    }

    (left, pos)
}

#[cfg(feature = "alloc")]
fn eval_arith_factor(expr: &[u8], pos: usize) -> (i64, usize) {
    let mut pos = skip_arith_ws(expr, pos);
    if pos >= expr.len() { return (0, pos); }

    // Handle parentheses
    if expr[pos] == b'(' {
        let (val, new_pos) = eval_arith_expr(expr, pos + 1);
        let mut pos = skip_arith_ws(expr, new_pos);
        if pos < expr.len() && expr[pos] == b')' { pos += 1; }
        return (val, pos);
    }

    // Handle negative
    if expr[pos] == b'-' {
        let (val, new_pos) = eval_arith_factor(expr, pos + 1);
        return (-val, new_pos);
    }

    // Parse number
    let mut num: i64 = 0;
    while pos < expr.len() && expr[pos] >= b'0' && expr[pos] <= b'9' {
        num = num * 10 + (expr[pos] - b'0') as i64;
        pos += 1;
    }

    (num, pos)
}

#[cfg(feature = "alloc")]
fn skip_arith_ws(expr: &[u8], pos: usize) -> usize {
    let mut pos = pos;
    while pos < expr.len() && (expr[pos] == b' ' || expr[pos] == b'\t') {
        pos += 1;
    }
    pos
}

/// Execute command and capture output
#[cfg(feature = "alloc")]
fn execute_capture(shell: &Shell, cmd: &[u8]) -> Vec<u8> {
    // Create pipe
    let mut pipe_fds = [-1i32; 2];
    unsafe {
        if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
            return Vec::new();
        }
    }

    let pid = io::fork();
    if pid == 0 {
        // Child
        io::close(pipe_fds[0]);
        io::dup2(pipe_fds[1], 1);
        io::close(pipe_fds[1]);

        // Execute in subshell
        let mut subshell = Shell::new(false);
        // Copy variables
        for (k, v) in &shell.variables {
            subshell.variables.insert(k.clone(), v.clone());
        }
        execute_script(&mut subshell, cmd);
        io::exit(subshell.last_status);
    }

    // Parent
    io::close(pipe_fds[1]);

    let mut output = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(pipe_fds[0], &mut buf);
        if n <= 0 { break; }
        output.extend_from_slice(&buf[..n as usize]);
    }
    io::close(pipe_fds[0]);

    let mut status = 0;
    io::waitpid(pid, &mut status, 0);

    output
}

/// Split string into words
#[cfg(feature = "alloc")]
fn split_words(shell: &Shell, s: &[u8]) -> Vec<Vec<u8>> {
    let mut words = Vec::new();
    let mut pos = 0;

    while pos < s.len() {
        pos = skip_whitespace_and_comments(s, pos);
        if pos >= s.len() { break; }

        let (word, new_pos) = parse_word(shell, s, pos);
        if !word.is_empty() {
            words.push(word);
        }
        pos = new_pos;
    }

    words
}

/// Execute a pipeline (handles &&, ||, pipes)
#[cfg(feature = "alloc")]
fn execute_pipeline(shell: &mut Shell, tokens: &[Token]) {
    // Split into segments by && and ||
    let mut segments: Vec<(Vec<Token>, Option<Token>)> = Vec::new();
    let mut current_segment: Vec<Token> = Vec::new();

    for token in tokens {
        match token {
            Token::AndIf | Token::OrIf => {
                segments.push((current_segment, Some(token.clone())));
                current_segment = Vec::new();
            }
            _ => {
                current_segment.push(token.clone());
            }
        }
    }
    if !current_segment.is_empty() {
        segments.push((current_segment, None));
    }

    // Execute segments with && / || logic
    for (segment, connector) in segments {
        // Check if we should execute based on previous status and connector
        let should_execute = match connector {
            None => true, // Last segment, always execute
            _ => true,    // First segment or continuation
        };

        if !should_execute {
            continue;
        }

        execute_simple_pipeline(shell, &segment);

        // Check if we should continue to next segment
        if let Some(Token::AndIf) = connector {
            if shell.last_status != 0 {
                break; // && failed, stop
            }
        } else if let Some(Token::OrIf) = connector {
            if shell.last_status == 0 {
                break; // || succeeded, stop
            }
        }
    }
}

/// Execute a simple pipeline (no &&/||)
#[cfg(feature = "alloc")]
fn execute_simple_pipeline(shell: &mut Shell, tokens: &[Token]) {
    let mut commands: Vec<Command> = Vec::new();
    let mut current = Command::new();

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Word(w) => {
                current.args.push(w.clone());
            }
            Token::Pipe => {
                if !current.args.is_empty() {
                    commands.push(current);
                    current = Command::new();
                }
            }
            Token::RedirectOut => {
                if i + 1 < tokens.len() {
                    if let Token::Word(f) = &tokens[i + 1] {
                        current.stdout_file = Some(f.clone());
                        current.stdout_append = false;
                        i += 1;
                    }
                }
            }
            Token::RedirectAppend => {
                if i + 1 < tokens.len() {
                    if let Token::Word(f) = &tokens[i + 1] {
                        current.stdout_file = Some(f.clone());
                        current.stdout_append = true;
                        i += 1;
                    }
                }
            }
            Token::RedirectIn => {
                if i + 1 < tokens.len() {
                    if let Token::Word(f) = &tokens[i + 1] {
                        current.stdin_file = Some(f.clone());
                        i += 1;
                    }
                }
            }
            Token::RedirectErr => {
                if i + 1 < tokens.len() {
                    if let Token::Word(f) = &tokens[i + 1] {
                        current.stderr_file = Some(f.clone());
                        i += 1;
                    }
                }
            }
            Token::Background => {
                current.background = true;
            }
            Token::AndIf | Token::OrIf => {
                // Should not happen in simple pipeline
            }
        }
        i += 1;
    }

    if !current.args.is_empty() {
        commands.push(current);
    }

    if commands.is_empty() { return; }

    // Single command - check for builtins
    if commands.len() == 1 && !commands[0].background {
        if execute_builtin(shell, &commands[0]) {
            return;
        }
    }

    // Execute pipeline with fork
    let n = commands.len();
    let mut prev_pipe_read: i32 = -1;
    let mut pids: Vec<i32> = Vec::new();

    for (i, cmd) in commands.iter().enumerate() {
        let is_last = i == n - 1;

        let mut pipe_fds = [-1i32; 2];
        if !is_last {
            unsafe {
                if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
                    io::write_str(2, b"sh: pipe failed\n");
                    return;
                }
            }
        }

        let pid = io::fork();
        if pid < 0 {
            io::write_str(2, b"sh: fork failed\n");
            return;
        }

        if pid == 0 {
            // Child
            if prev_pipe_read >= 0 {
                io::dup2(prev_pipe_read, 0);
                io::close(prev_pipe_read);
            }

            if !is_last {
                io::close(pipe_fds[0]);
                io::dup2(pipe_fds[1], 1);
                io::close(pipe_fds[1]);
            }

            // Handle redirections
            if let Some(ref f) = cmd.stdin_file {
                let fd = io::open(f, libc::O_RDONLY, 0);
                if fd < 0 {
                    io::write_str(2, f);
                    io::write_str(2, b": No such file or directory\n");
                    io::exit(1);
                }
                io::dup2(fd, 0);
                io::close(fd);
            }

            if let Some(ref f) = cmd.stdout_file {
                let flags = if cmd.stdout_append {
                    libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND
                } else {
                    libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC
                };
                let fd = io::open(f, flags, 0o644);
                if fd < 0 {
                    io::write_str(2, b"sh: cannot create ");
                    io::write_all(2, f);
                    io::write_str(2, b"\n");
                    io::exit(1);
                }
                io::dup2(fd, 1);
                io::close(fd);
            }

            if let Some(ref f) = cmd.stderr_file {
                let fd = io::open(f, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644);
                if fd >= 0 {
                    io::dup2(fd, 2);
                    io::close(fd);
                }
            }

            // Set local variables in environment
            for (k, v) in &shell.variables {
                let mut name_buf = [0u8; 256];
                let mut value_buf = [0u8; 4096];
                let nlen = core::cmp::min(k.len(), name_buf.len() - 1);
                let vlen = core::cmp::min(v.len(), value_buf.len() - 1);
                name_buf[..nlen].copy_from_slice(&k[..nlen]);
                value_buf[..vlen].copy_from_slice(&v[..vlen]);
                unsafe {
                    libc::setenv(
                        name_buf.as_ptr() as *const i8,
                        value_buf.as_ptr() as *const i8,
                        1,
                    );
                }
            }

            execute_command(cmd);
            io::exit(127);
        }

        pids.push(pid);

        if prev_pipe_read >= 0 {
            io::close(prev_pipe_read);
        }

        if !is_last {
            io::close(pipe_fds[1]);
            prev_pipe_read = pipe_fds[0];
        }
    }

    // Wait
    let background = commands.last().map(|c| c.background).unwrap_or(false);
    if !background {
        for pid in pids {
            let mut status: i32 = 0;
            io::waitpid(pid, &mut status, 0);
            shell.last_status = (status >> 8) & 0xff;
        }
    }
}

/// Execute a command (in child process)
#[cfg(feature = "alloc")]
fn execute_command(cmd: &Command) {
    if cmd.args.is_empty() { return; }

    let mut argv_ptrs: Vec<*const i8> = Vec::new();
    let mut argv_storage: Vec<Vec<u8>> = Vec::new();

    for arg in &cmd.args {
        let mut s = arg.clone();
        s.push(0);
        argv_storage.push(s);
    }

    for s in &argv_storage {
        argv_ptrs.push(s.as_ptr() as *const i8);
    }
    argv_ptrs.push(core::ptr::null());

    unsafe {
        libc::execvp(argv_ptrs[0], argv_ptrs.as_ptr());
    }

    io::write_str(2, b"sh: ");
    io::write_all(2, &cmd.args[0]);
    io::write_str(2, b": command not found\n");
}

/// Execute built-in command
#[cfg(feature = "alloc")]
fn execute_builtin(shell: &mut Shell, cmd: &Command) -> bool {
    if cmd.args.is_empty() { return true; }

    let name = &cmd.args[0];

    if name == b"exit" {
        shell.should_exit = true;
        shell.exit_code = if cmd.args.len() > 1 {
            sys::parse_i64(&cmd.args[1]).unwrap_or(0) as i32
        } else {
            shell.last_status
        };
        return true;
    }

    if name == b"cd" {
        let path = if cmd.args.len() > 1 {
            &cmd.args[1]
        } else if let Some(home) = io::getenv(b"HOME") {
            home
        } else {
            b"/root"
        };

        if io::chdir(path) < 0 {
            io::write_str(2, b"cd: ");
            io::write_all(2, path);
            io::write_str(2, b": No such directory\n");
            shell.last_status = 1;
        } else {
            shell.last_status = 0;
        }
        return true;
    }

    if name == b"pwd" {
        let mut buf = [0u8; 4096];
        unsafe {
            if !libc::getcwd(buf.as_mut_ptr() as *mut i8, buf.len()).is_null() {
                let len = io::strlen(buf.as_ptr());
                io::write_all(1, &buf[..len]);
                io::write_str(1, b"\n");
                shell.last_status = 0;
            } else {
                shell.last_status = 1;
            }
        }
        return true;
    }

    if name == b"export" {
        if cmd.args.len() > 1 {
            let arg = &cmd.args[1];
            if let Some(eq_pos) = arg.iter().position(|&c| c == b'=') {
                let name_part = &arg[..eq_pos];
                let value_part = &arg[eq_pos + 1..];

                let mut name_buf = [0u8; 256];
                let mut value_buf = [0u8; 4096];
                let nlen = core::cmp::min(name_part.len(), name_buf.len() - 1);
                let vlen = core::cmp::min(value_part.len(), value_buf.len() - 1);
                name_buf[..nlen].copy_from_slice(&name_part[..nlen]);
                value_buf[..vlen].copy_from_slice(&value_part[..vlen]);

                unsafe {
                    libc::setenv(
                        name_buf.as_ptr() as *const i8,
                        value_buf.as_ptr() as *const i8,
                        1,
                    );
                }
            } else {
                // Export existing variable
                if let Some(value) = shell.get_var(arg) {
                    let mut name_buf = [0u8; 256];
                    let mut value_buf = [0u8; 4096];
                    let nlen = core::cmp::min(arg.len(), name_buf.len() - 1);
                    let vlen = core::cmp::min(value.len(), value_buf.len() - 1);
                    name_buf[..nlen].copy_from_slice(&arg[..nlen]);
                    value_buf[..vlen].copy_from_slice(&value[..vlen]);
                    unsafe {
                        libc::setenv(
                            name_buf.as_ptr() as *const i8,
                            value_buf.as_ptr() as *const i8,
                            1,
                        );
                    }
                }
            }
        }
        shell.last_status = 0;
        return true;
    }

    if name == b"unset" {
        if cmd.args.len() > 1 {
            let var = &cmd.args[1];
            shell.variables.remove(var);
            let mut name_buf = [0u8; 256];
            let nlen = core::cmp::min(var.len(), name_buf.len() - 1);
            name_buf[..nlen].copy_from_slice(&var[..nlen]);
            unsafe {
                libc::unsetenv(name_buf.as_ptr() as *const i8);
            }
        }
        shell.last_status = 0;
        return true;
    }

    if name == b"echo" {
        let mut newline = true;
        let mut start = 1;

        if cmd.args.len() > 1 && cmd.args[1] == b"-n" {
            newline = false;
            start = 2;
        }

        let mut first = true;
        for arg in &cmd.args[start..] {
            if !first { io::write_str(1, b" "); }
            io::write_all(1, arg);
            first = false;
        }
        if newline { io::write_str(1, b"\n"); }
        shell.last_status = 0;
        return true;
    }

    if name == b"test" || name == b"[" {
        shell.last_status = execute_test(&cmd.args[1..]);
        return true;
    }

    if name == b"true" {
        shell.last_status = 0;
        return true;
    }

    if name == b"false" {
        shell.last_status = 1;
        return true;
    }

    if name == b":" {
        shell.last_status = 0;
        return true;
    }

    if name == b"source" || name == b"." {
        if cmd.args.len() > 1 {
            let fd = io::open(&cmd.args[1], libc::O_RDONLY, 0);
            if fd >= 0 {
                let content = io::read_all(fd);
                io::close(fd);
                execute_script(shell, &content);
            } else {
                io::write_str(2, b"sh: cannot open ");
                io::write_all(2, &cmd.args[1]);
                io::write_str(2, b"\n");
                shell.last_status = 1;
            }
        }
        return true;
    }

    if name == b"read" {
        if cmd.args.len() > 1 {
            let var_name = &cmd.args[1];
            let mut line = Vec::new();
            let mut buf = [0u8; 1];
            loop {
                let n = io::read(0, &mut buf);
                if n <= 0 || buf[0] == b'\n' { break; }
                line.push(buf[0]);
            }
            shell.set_var(var_name, &line);
        }
        shell.last_status = 0;
        return true;
    }

    if name == b"exec" {
        if cmd.args.len() > 1 {
            let mut cmd_exec = Command::new();
            for arg in &cmd.args[1..] {
                cmd_exec.args.push(arg.clone());
            }
            execute_command(&cmd_exec);
            shell.last_status = 127;
        }
        return true;
    }

    if name == b"set" {
        if cmd.args.len() > 1 && cmd.args[1] == b"-e" {
            // set -e: exit on error (just acknowledge, don't implement fully)
        }
        shell.last_status = 0;
        return true;
    }

    if name == b"shift" {
        // shift - in scripts this shifts positional parameters
        shell.last_status = 0;
        return true;
    }

    if name == b"return" {
        shell.last_status = if cmd.args.len() > 1 {
            sys::parse_i64(&cmd.args[1]).unwrap_or(0) as i32
        } else {
            0
        };
        return true;
    }

    false
}

/// Execute test command
#[cfg(feature = "alloc")]
fn execute_test(args: &[Vec<u8>]) -> i32 {
    if args.is_empty() { return 1; }

    // Remove trailing ] if present
    let args: Vec<&[u8]> = args.iter()
        .map(|a| a.as_slice())
        .filter(|a| *a != b"]")
        .collect();

    if args.is_empty() { return 1; }

    // Single arg: true if non-empty
    if args.len() == 1 {
        return if args[0].is_empty() { 1 } else { 0 };
    }

    // Two args: unary operators
    if args.len() == 2 {
        let op = args[0];
        let arg = args[1];

        if op == b"-n" { return if arg.is_empty() { 1 } else { 0 }; }
        if op == b"-z" { return if arg.is_empty() { 0 } else { 1 }; }
        if op == b"-e" || op == b"-a" { return if file_exists(arg) { 0 } else { 1 }; }
        if op == b"-f" { return if is_regular_file(arg) { 0 } else { 1 }; }
        if op == b"-d" { return if is_directory(arg) { 0 } else { 1 }; }
        if op == b"-r" { return if is_readable(arg) { 0 } else { 1 }; }
        if op == b"-w" { return if is_writable(arg) { 0 } else { 1 }; }
        if op == b"-x" { return if is_executable(arg) { 0 } else { 1 }; }
        if op == b"-s" { return if file_size(arg) > 0 { 0 } else { 1 }; }
        if op == b"-L" || op == b"-h" { return if is_symlink(arg) { 0 } else { 1 }; }
        if op == b"!" { return execute_test(&[arg.to_vec()]) ^ 1; }
    }

    // Three args: binary operators
    if args.len() == 3 {
        let left = args[0];
        let op = args[1];
        let right = args[2];

        if op == b"=" || op == b"==" { return if left == right { 0 } else { 1 }; }
        if op == b"!=" { return if left != right { 0 } else { 1 }; }

        // Numeric comparisons
        let ln = sys::parse_i64(left).unwrap_or(0);
        let rn = sys::parse_i64(right).unwrap_or(0);

        if op == b"-eq" { return if ln == rn { 0 } else { 1 }; }
        if op == b"-ne" { return if ln != rn { 0 } else { 1 }; }
        if op == b"-lt" { return if ln < rn { 0 } else { 1 }; }
        if op == b"-le" { return if ln <= rn { 0 } else { 1 }; }
        if op == b"-gt" { return if ln > rn { 0 } else { 1 }; }
        if op == b"-ge" { return if ln >= rn { 0 } else { 1 }; }
    }

    1
}

#[cfg(feature = "alloc")]
fn file_exists(path: &[u8]) -> bool {
    let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe { libc::stat(path_buf.as_ptr() as *const i8, &mut stat_buf) == 0 }
}

#[cfg(feature = "alloc")]
fn is_regular_file(path: &[u8]) -> bool {
    let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe {
        if libc::stat(path_buf.as_ptr() as *const i8, &mut stat_buf) != 0 { return false; }
        (stat_buf.st_mode & libc::S_IFMT) == libc::S_IFREG
    }
}

#[cfg(feature = "alloc")]
fn is_directory(path: &[u8]) -> bool {
    let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe {
        if libc::stat(path_buf.as_ptr() as *const i8, &mut stat_buf) != 0 { return false; }
        (stat_buf.st_mode & libc::S_IFMT) == libc::S_IFDIR
    }
}

#[cfg(feature = "alloc")]
fn is_readable(path: &[u8]) -> bool {
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe { libc::access(path_buf.as_ptr() as *const i8, libc::R_OK) == 0 }
}

#[cfg(feature = "alloc")]
fn is_writable(path: &[u8]) -> bool {
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe { libc::access(path_buf.as_ptr() as *const i8, libc::W_OK) == 0 }
}

#[cfg(feature = "alloc")]
fn is_executable(path: &[u8]) -> bool {
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe { libc::access(path_buf.as_ptr() as *const i8, libc::X_OK) == 0 }
}

#[cfg(feature = "alloc")]
fn is_symlink(path: &[u8]) -> bool {
    let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe {
        if libc::lstat(path_buf.as_ptr() as *const i8, &mut stat_buf) != 0 { return false; }
        (stat_buf.st_mode & libc::S_IFMT) == libc::S_IFLNK
    }
}

#[cfg(feature = "alloc")]
fn file_size(path: &[u8]) -> i64 {
    let mut stat_buf: libc::stat = unsafe { core::mem::zeroed() };
    let mut path_buf = [0u8; 4096];
    let plen = core::cmp::min(path.len(), path_buf.len() - 1);
    path_buf[..plen].copy_from_slice(&path[..plen]);
    unsafe {
        if libc::stat(path_buf.as_ptr() as *const i8, &mut stat_buf) != 0 { return 0; }
        stat_buf.st_size
    }
}

fn format_number(mut n: u64, buf: &mut [u8]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }
    let mut i = buf.len();
    while n > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    &buf[i..]
}

fn format_signed(n: i64, buf: &mut [u8]) -> &[u8] {
    if n < 0 {
        let buf_len = buf.len();
        let s_len = format_number((-n) as u64, &mut buf[1..]).len();
        let start = buf_len - s_len - 1;
        buf[start] = b'-';
        &buf[start..]
    } else {
        format_number(n as u64, buf)
    }
}

/// Trait extension for trimming
trait TrimAscii {
    fn trim_ascii(&self) -> &[u8];
    fn trim_ascii_end(&self) -> &[u8];
}

impl TrimAscii for [u8] {
    fn trim_ascii(&self) -> &[u8] {
        let start = self.iter().position(|&c| c != b' ' && c != b'\t' && c != b'\r' && c != b'\n').unwrap_or(self.len());
        let end = self.iter().rposition(|&c| c != b' ' && c != b'\t' && c != b'\r' && c != b'\n').map(|i| i + 1).unwrap_or(start);
        &self[start..end]
    }

    fn trim_ascii_end(&self) -> &[u8] {
        let end = self.iter().rposition(|&c| c != b' ' && c != b'\t' && c != b'\r' && c != b'\n').map(|i| i + 1).unwrap_or(0);
        &self[..end]
    }
}
