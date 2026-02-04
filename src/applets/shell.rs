//! POSIX Shell implementation
//!
//! A minimal but functional shell supporting:
//! - Command execution
//! - Pipes (cmd1 | cmd2)
//! - Redirections (>, >>, <, 2>)
//! - Environment variables ($VAR)
//! - Built-in commands (cd, exit, export, etc.)
//! - Script execution

use crate::io;
use super::get_arg;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
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
}

#[cfg(feature = "alloc")]
impl Shell {
    fn new(interactive: bool) -> Self {
        Shell {
            last_status: 0,
            interactive,
            should_exit: false,
            exit_code: 0,
        }
    }
}

/// Token types
#[cfg(feature = "alloc")]
#[derive(Clone, PartialEq)]
enum Token {
    Word(Vec<u8>),
    Pipe,           // |
    RedirectOut,    // >
    RedirectAppend, // >>
    RedirectIn,     // <
    RedirectErr,    // 2>
    Background,     // &
    Semicolon,      // ;
    Newline,
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
        // Minimal shell without alloc - just read and exec single commands
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
        // Print prompt
        if io::isatty(0) {
            io::write_str(1, b"$ ");
        }

        // Read a line
        pos = 0;
        loop {
            let mut c = [0u8; 1];
            let n = io::read(0, &mut c);
            if n <= 0 {
                if pos == 0 {
                    return; // EOF
                }
                break;
            }

            if c[0] == b'\n' {
                break;
            }

            if pos < line_buf.len() - 1 {
                line_buf[pos] = c[0];
                pos += 1;
            }
        }

        if pos == 0 {
            continue;
        }

        let line = &line_buf[..pos];

        // Handle built-in exit
        if line == b"exit" {
            return;
        }

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

        // Execute command
        let pid = io::fork();
        if pid == 0 {
            // Child - exec the command via /bin/sh -c
            // But we ARE /bin/sh, so just exec directly
            let mut cmd_buf = [0u8; 1024];
            cmd_buf[..pos].copy_from_slice(line);
            cmd_buf[pos] = 0;

            // Simple: split by spaces and exec
            let mut args: [*const i8; 32] = [core::ptr::null(); 32];
            let mut arg_count = 0;
            let mut start = 0;
            let mut in_word = false;

            for i in 0..=pos {
                if i == pos || line_buf[i] == b' ' || line_buf[i] == b'\t' {
                    if in_word && arg_count < 31 {
                        line_buf[i] = 0; // null terminate
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
                unsafe {
                    libc::execvp(args[0], args.as_ptr());
                }
                io::write_str(2, b"sh: command not found\n");
            }
            io::exit(127);
        }

        // Parent - wait
        let mut status: i32 = 0;
        io::waitpid(pid, &mut status, 0);
    }
}

/// Interactive shell loop
#[cfg(feature = "alloc")]
fn interactive_loop(shell: &mut Shell) {
    let mut line_buf = Vec::new();

    loop {
        if shell.should_exit {
            return;
        }

        // Print prompt
        if shell.interactive {
            io::write_str(1, b"$ ");
        }

        // Read a line
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

            if c[0] == b'\n' {
                break;
            }

            line_buf.push(c[0]);
        }

        if line_buf.is_empty() {
            continue;
        }

        execute_line(shell, &line_buf);
    }
}

/// Execute a script (multiple lines)
#[cfg(feature = "alloc")]
fn execute_script(shell: &mut Shell, script: &[u8]) {
    for line in script.split(|&c| c == b'\n') {
        if shell.should_exit {
            return;
        }

        let line = trim(line);
        if line.is_empty() || line[0] == b'#' {
            continue;
        }

        execute_line(shell, line);
    }
}

/// Execute a single line (may contain multiple commands separated by ; or |)
#[cfg(feature = "alloc")]
fn execute_line(shell: &mut Shell, line: &[u8]) {
    let tokens = tokenize(line);
    if tokens.is_empty() {
        return;
    }

    // Split into pipelines by semicolon
    let mut current_pipeline: Vec<Token> = Vec::new();

    for token in tokens {
        if matches!(token, Token::Semicolon | Token::Newline) {
            if !current_pipeline.is_empty() {
                execute_pipeline(shell, &current_pipeline);
                current_pipeline.clear();
            }
        } else {
            current_pipeline.push(token);
        }
    }

    if !current_pipeline.is_empty() {
        execute_pipeline(shell, &current_pipeline);
    }
}

/// Execute a pipeline (commands separated by |)
#[cfg(feature = "alloc")]
fn execute_pipeline(shell: &mut Shell, tokens: &[Token]) {
    // Parse into commands
    let mut commands: Vec<Command> = Vec::new();
    let mut current = Command::new();

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Word(w) => {
                // Expand variables
                let expanded = expand_variables(w);
                current.args.push(expanded);
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
            _ => {}
        }
        i += 1;
    }

    if !current.args.is_empty() {
        commands.push(current);
    }

    if commands.is_empty() {
        return;
    }

    // Single command - check for built-ins
    if commands.len() == 1 && !commands[0].background {
        if execute_builtin(shell, &commands[0]) {
            return;
        }
    }

    // Execute pipeline
    let n = commands.len();
    let mut prev_pipe_read: i32 = -1;
    let mut pids: Vec<i32> = Vec::new();

    for (i, cmd) in commands.iter().enumerate() {
        let is_last = i == n - 1;

        // Create pipe if not last command
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
            // Child process

            // Set up stdin from previous pipe
            if prev_pipe_read >= 0 {
                io::dup2(prev_pipe_read, 0);
                io::close(prev_pipe_read);
            }

            // Set up stdout to next pipe
            if !is_last {
                io::close(pipe_fds[0]);
                io::dup2(pipe_fds[1], 1);
                io::close(pipe_fds[1]);
            }

            // Handle redirections
            if let Some(ref f) = cmd.stdin_file {
                let fd = io::open(f, libc::O_RDONLY, 0);
                if fd < 0 {
                    io::write_str(2, b"sh: cannot open ");
                    io::write_all(2, f);
                    io::write_str(2, b"\n");
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
                    io::write_str(2, b"sh: cannot open ");
                    io::write_all(2, f);
                    io::write_str(2, b"\n");
                    io::exit(1);
                }
                io::dup2(fd, 1);
                io::close(fd);
            }

            if let Some(ref f) = cmd.stderr_file {
                let fd = io::open(f, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644);
                if fd < 0 {
                    io::exit(1);
                }
                io::dup2(fd, 2);
                io::close(fd);
            }

            // Execute
            execute_command(cmd);
            io::exit(127);
        }

        // Parent
        pids.push(pid);

        // Close previous pipe read end
        if prev_pipe_read >= 0 {
            io::close(prev_pipe_read);
        }

        // Save this pipe's read end for next command
        if !is_last {
            io::close(pipe_fds[1]);
            prev_pipe_read = pipe_fds[0];
        }
    }

    // Wait for all children (unless background)
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
    if cmd.args.is_empty() {
        return;
    }

    // Build null-terminated argv
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

    // If exec failed
    io::write_str(2, b"sh: ");
    io::write_all(2, &cmd.args[0]);
    io::write_str(2, b": command not found\n");
}

/// Execute built-in command, returns true if it was a built-in
#[cfg(feature = "alloc")]
fn execute_builtin(shell: &mut Shell, cmd: &Command) -> bool {
    if cmd.args.is_empty() {
        return true;
    }

    let name = &cmd.args[0];

    if name == b"exit" {
        shell.should_exit = true;
        shell.exit_code = if cmd.args.len() > 1 {
            parse_number(&cmd.args[1]).unwrap_or(0)
        } else {
            shell.last_status
        };
        return true;
    }

    if name == b"cd" {
        let path = if cmd.args.len() > 1 {
            &cmd.args[1]
        } else {
            // Default to HOME
            if let Some(home) = io::getenv(b"HOME") {
                // Can't easily use this without allocation for path
                b"/root" as &[u8]
            } else {
                b"/root"
            }
        };

        let mut path_buf = [0u8; 4096];
        let plen = core::cmp::min(path.len(), path_buf.len() - 1);
        path_buf[..plen].copy_from_slice(&path[..plen]);

        unsafe {
            if libc::chdir(path_buf.as_ptr() as *const i8) != 0 {
                io::write_str(2, b"cd: ");
                io::write_all(2, path);
                io::write_str(2, b": No such directory\n");
                shell.last_status = 1;
            } else {
                shell.last_status = 0;
            }
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
                io::write_str(2, b"pwd: error\n");
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
                name_buf[..nlen].copy_from_slice(&name_part[..nlen]);

                let vlen = core::cmp::min(value_part.len(), value_buf.len() - 1);
                value_buf[..vlen].copy_from_slice(&value_part[..vlen]);

                unsafe {
                    libc::setenv(
                        name_buf.as_ptr() as *const i8,
                        value_buf.as_ptr() as *const i8,
                        1,
                    );
                }
            }
        }
        shell.last_status = 0;
        return true;
    }

    if name == b"unset" {
        if cmd.args.len() > 1 {
            let mut name_buf = [0u8; 256];
            let nlen = core::cmp::min(cmd.args[1].len(), name_buf.len() - 1);
            name_buf[..nlen].copy_from_slice(&cmd.args[1][..nlen]);

            unsafe {
                libc::unsetenv(name_buf.as_ptr() as *const i8);
            }
        }
        shell.last_status = 0;
        return true;
    }

    if name == b"echo" {
        let mut first = true;
        let mut newline = true;
        let mut start = 1;

        // Check for -n flag
        if cmd.args.len() > 1 && cmd.args[1] == b"-n" {
            newline = false;
            start = 2;
        }

        for arg in &cmd.args[start..] {
            if !first {
                io::write_str(1, b" ");
            }
            io::write_all(1, arg);
            first = false;
        }

        if newline {
            io::write_str(1, b"\n");
        }
        shell.last_status = 0;
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
        // No-op
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

    if name == b"exec" {
        if cmd.args.len() > 1 {
            let rest = &cmd.args[1..];
            let mut cmd_exec = Command::new();
            for arg in rest {
                cmd_exec.args.push(arg.clone());
            }
            execute_command(&cmd_exec);
            // If we get here, exec failed
            shell.last_status = 127;
        }
        return true;
    }

    // Not a built-in
    false
}

/// Tokenize input line
#[cfg(feature = "alloc")]
fn tokenize(input: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < input.len() {
        let c = input[i];

        // Skip whitespace
        if c == b' ' || c == b'\t' {
            i += 1;
            continue;
        }

        // Comments
        if c == b'#' {
            break;
        }

        // Operators
        if c == b'|' {
            tokens.push(Token::Pipe);
            i += 1;
            continue;
        }

        if c == b';' {
            tokens.push(Token::Semicolon);
            i += 1;
            continue;
        }

        if c == b'&' {
            tokens.push(Token::Background);
            i += 1;
            continue;
        }

        if c == b'>' {
            if i + 1 < input.len() && input[i + 1] == b'>' {
                tokens.push(Token::RedirectAppend);
                i += 2;
            } else {
                tokens.push(Token::RedirectOut);
                i += 1;
            }
            continue;
        }

        if c == b'<' {
            tokens.push(Token::RedirectIn);
            i += 1;
            continue;
        }

        if c == b'2' && i + 1 < input.len() && input[i + 1] == b'>' {
            tokens.push(Token::RedirectErr);
            i += 2;
            continue;
        }

        // Word (possibly quoted)
        let mut word = Vec::new();

        while i < input.len() {
            let c = input[i];

            if c == b' ' || c == b'\t' || c == b'|' || c == b';' || c == b'&'
               || c == b'>' || c == b'<' || c == b'#' {
                break;
            }

            if c == b'\'' {
                // Single quote - take everything literally until closing quote
                i += 1;
                while i < input.len() && input[i] != b'\'' {
                    word.push(input[i]);
                    i += 1;
                }
                if i < input.len() {
                    i += 1; // Skip closing quote
                }
            } else if c == b'"' {
                // Double quote - allow variable expansion
                i += 1;
                while i < input.len() && input[i] != b'"' {
                    if input[i] == b'\\' && i + 1 < input.len() {
                        i += 1;
                        word.push(input[i]);
                    } else {
                        word.push(input[i]);
                    }
                    i += 1;
                }
                if i < input.len() {
                    i += 1; // Skip closing quote
                }
            } else if c == b'\\' && i + 1 < input.len() {
                // Escape
                i += 1;
                word.push(input[i]);
                i += 1;
            } else {
                word.push(c);
                i += 1;
            }
        }

        if !word.is_empty() {
            tokens.push(Token::Word(word));
        }
    }

    tokens
}

/// Expand environment variables in a word
#[cfg(feature = "alloc")]
fn expand_variables(word: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < word.len() {
        if word[i] == b'$' && i + 1 < word.len() {
            i += 1;

            // Special variables
            if word[i] == b'?' {
                // TODO: expand $? to last exit status
                result.push(b'0');
                i += 1;
                continue;
            }

            if word[i] == b'$' {
                // $$ - PID
                let pid = unsafe { libc::getpid() };
                let mut buf = [0u8; 16];
                let s = format_number(pid as u64, &mut buf);
                result.extend_from_slice(s);
                i += 1;
                continue;
            }

            // Variable name
            let mut name = Vec::new();
            if word[i] == b'{' {
                // ${VAR}
                i += 1;
                while i < word.len() && word[i] != b'}' {
                    name.push(word[i]);
                    i += 1;
                }
                if i < word.len() {
                    i += 1; // Skip }
                }
            } else {
                // $VAR
                while i < word.len() && (word[i].is_ascii_alphanumeric() || word[i] == b'_') {
                    name.push(word[i]);
                    i += 1;
                }
            }

            if !name.is_empty() {
                if let Some(value) = io::getenv(&name) {
                    result.extend_from_slice(value);
                }
            }
        } else {
            result.push(word[i]);
            i += 1;
        }
    }

    result
}

/// Format a number into a buffer, return slice
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

/// Parse a number from bytes
#[cfg(feature = "alloc")]
fn parse_number(s: &[u8]) -> Option<i32> {
    let mut result: i32 = 0;
    let mut negative = false;
    let mut i = 0;

    if !s.is_empty() && s[0] == b'-' {
        negative = true;
        i = 1;
    }

    while i < s.len() {
        if s[i] >= b'0' && s[i] <= b'9' {
            result = result.wrapping_mul(10).wrapping_add((s[i] - b'0') as i32);
        } else {
            return None;
        }
        i += 1;
    }

    Some(if negative { -result } else { result })
}

/// Trim whitespace
fn trim(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|&c| c != b' ' && c != b'\t' && c != b'\r').unwrap_or(s.len());
    let end = s.iter().rposition(|&c| c != b' ' && c != b'\t' && c != b'\r').map(|i| i + 1).unwrap_or(start);
    &s[start..end]
}
