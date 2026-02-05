//! Shell entry points
//!
//! This module contains the main entry points for the shell applet:
//! - `sh` - The main POSIX shell entry point
//! - `ash` - Alias for sh (Almquist shell compatible)
//! - `dash` - Alias for sh (Debian Almquist shell compatible)
//!
//! It also contains the interactive loop and minimal shell implementations.

use crate::io;
use super::get_arg;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use super::state::Shell;

#[cfg(feature = "alloc")]
use super::execute::execute_script;

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

/// Ash shell entry point (alias for sh)
pub fn ash(argc: i32, argv: *const *const u8) -> i32 {
    sh(argc, argv)
}

/// Dash shell entry point (alias for sh)
pub fn dash(argc: i32, argv: *const *const u8) -> i32 {
    sh(argc, argv)
}

/// Minimal shell for no-alloc builds
///
/// This provides a very basic shell implementation that works without
/// heap allocation. It supports:
/// - Simple command execution via fork/exec
/// - The `cd` builtin
/// - The `exit` builtin
#[cfg(not(feature = "alloc"))]
pub(super) fn minimal_shell() {
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
///
/// Reads commands from stdin and executes them until EOF or the shell
/// is instructed to exit.
#[cfg(feature = "alloc")]
pub(super) fn interactive_loop(shell: &mut Shell) {
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
