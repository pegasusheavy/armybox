//! Variable expansion for shell
//!
//! This module handles shell variable expansion including:
//! - Simple variable expansion: `$VAR` and `${VAR}`
//! - Special variables: `$?` (last exit status) and `$$` (process ID)
//! - Command substitution: `$(cmd)`
//! - Arithmetic expansion: `$((expr))`
//! - Word splitting for expanded values

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use super::state::Shell;
#[cfg(feature = "alloc")]
use super::arithmetic::eval_arithmetic;
use crate::io;

#[cfg(feature = "alloc")]
use super::util::{format_number, format_signed};

/// Expand a single $ expression starting at the given position
///
/// Handles:
/// - `$VAR` - simple variable
/// - `${VAR}` - braced variable
/// - `$?` - last exit status
/// - `$$` - current process ID
/// - `$(cmd)` - command substitution
/// - `$((expr))` - arithmetic expansion
///
/// Returns the expanded bytes and the new position after the expansion.
#[cfg(feature = "alloc")]
pub(super) fn expand_dollar(shell: &Shell, input: &[u8], start: usize) -> (Vec<u8>, usize) {
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
            if input[pos] == b'(' {
                depth += 1;
            } else if input[pos] == b')' {
                depth -= 1;
            }
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
            if input[pos] == b'(' {
                depth += 1;
            } else if input[pos] == b')' {
                depth -= 1;
            }
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
        if pos < input.len() {
            pos += 1;
        }
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

/// Expand all $ expressions in a string
///
/// Iterates through the input and expands each `$` using `expand_dollar`.
#[cfg(feature = "alloc")]
pub(super) fn expand_string(shell: &Shell, input: &[u8]) -> Vec<u8> {
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

/// Execute a command and capture its output
///
/// Used for command substitution `$(cmd)`. Forks a subshell,
/// captures stdout via a pipe, and returns the output bytes.
#[cfg(feature = "alloc")]
pub(super) fn execute_capture(shell: &Shell, cmd: &[u8]) -> Vec<u8> {
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
        super::execute_script(&mut subshell, cmd);
        io::exit(subshell.last_status);
    }

    // Parent
    io::close(pipe_fds[1]);

    let mut output = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = io::read(pipe_fds[0], &mut buf);
        if n <= 0 {
            break;
        }
        output.extend_from_slice(&buf[..n as usize]);
    }
    io::close(pipe_fds[0]);

    let mut status = 0;
    io::waitpid(pid, &mut status, 0);

    output
}

/// Split a string into words
///
/// Splits the input on whitespace, respecting shell quoting rules.
/// Uses `skip_whitespace_and_comments` and `parse_word` from the parent module.
#[cfg(feature = "alloc")]
pub(super) fn split_words(shell: &Shell, s: &[u8]) -> Vec<Vec<u8>> {
    let mut words = Vec::new();
    let mut pos = 0;

    while pos < s.len() {
        pos = super::skip_whitespace_and_comments(s, pos);
        if pos >= s.len() {
            break;
        }

        let (word, new_pos) = super::parse_word(shell, s, pos);
        if !word.is_empty() {
            words.push(word);
        }
        pos = new_pos;
    }

    words
}
