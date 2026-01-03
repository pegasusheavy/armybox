//! GNU Screen-like terminal multiplexer
//!
//! A simplified screen implementation supporting:
//! - Session creation and management
//! - Detach/reattach functionality
//! - Basic key bindings (Ctrl+A prefix)

use crate::io;
use crate::sys;
use super::get_arg;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Session state stored in /tmp/armybox-screen/
const SESSION_DIR: &[u8] = b"/tmp/armybox-screen\0";

/// Screen entry point
pub fn screen(argc: i32, argv: *const *const u8) -> i32 {
    let mut list_sessions = false;
    let mut reattach = false;
    let mut detach_others = false;
    let mut session_name: Option<&[u8]> = None;
    let mut cmd_start = argc;

    // Parse arguments
    let mut i = 1;
    while i < argc {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            match arg {
                b"-ls" | b"-list" => list_sessions = true,
                b"-r" | b"-R" => {
                    reattach = true;
                    if i + 1 < argc {
                        if let Some(name) = unsafe { get_arg(argv, i + 1) } {
                            if !name.starts_with(b"-") {
                                session_name = Some(name);
                                i += 1;
                            }
                        }
                    }
                }
                b"-d" => detach_others = true,
                b"-dr" | b"-rd" | b"-D" | b"-RD" | b"-DR" => {
                    detach_others = true;
                    reattach = true;
                    if i + 1 < argc {
                        if let Some(name) = unsafe { get_arg(argv, i + 1) } {
                            if !name.starts_with(b"-") {
                                session_name = Some(name);
                                i += 1;
                            }
                        }
                    }
                }
                b"-S" => {
                    if i + 1 < argc {
                        session_name = unsafe { get_arg(argv, i + 1) };
                        i += 1;
                    }
                }
                b"-h" | b"--help" => {
                    print_help();
                    return 0;
                }
                b"-v" | b"--version" => {
                    io::write_str(1, b"armybox screen 0.1\n");
                    return 0;
                }
                _ => {
                    if !arg.starts_with(b"-") {
                        cmd_start = i;
                        break;
                    }
                }
            }
        }
        i += 1;
    }

    // Ensure session directory exists
    ensure_session_dir();

    if list_sessions {
        return list_all_sessions();
    }

    if reattach {
        return attach_session(session_name, detach_others);
    }

    // Create new session
    create_session(session_name, argc, argv, cmd_start)
}

fn print_help() {
    io::write_str(1, b"Usage: screen [options] [cmd [args]]\n");
    io::write_str(1, b"\nOptions:\n");
    io::write_str(1, b"  -S name    Create session with name\n");
    io::write_str(1, b"  -ls        List sessions\n");
    io::write_str(1, b"  -r [name]  Reattach to session\n");
    io::write_str(1, b"  -d         Detach a running session\n");
    io::write_str(1, b"  -dr        Detach and reattach\n");
    io::write_str(1, b"\nKey bindings (inside screen):\n");
    io::write_str(1, b"  Ctrl+A d   Detach from session\n");
    io::write_str(1, b"  Ctrl+A k   Kill current window\n");
    io::write_str(1, b"  Ctrl+A ?   Show help\n");
}

fn ensure_session_dir() {
    unsafe {
        libc::mkdir(SESSION_DIR.as_ptr() as *const i8, 0o700);
    }
}

fn list_all_sessions() -> i32 {
    let fd = io::open(SESSION_DIR, libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 {
        io::write_str(1, b"No sessions.\n");
        return 0;
    }

    io::write_str(1, b"There are screens on:\n");

    let mut buf = [0u8; 4096];
    let mut found = false;

    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }

        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if name != b"." && name != b".." {
                // Check if it's a valid session (socket file)
                let mut path = [0u8; 256];
                let mut len = 0;
                for &c in SESSION_DIR.iter().take(SESSION_DIR.len() - 1) {
                    path[len] = c;
                    len += 1;
                }
                path[len] = b'/';
                len += 1;
                for &c in name {
                    path[len] = c;
                    len += 1;
                }
                path[len] = 0;

                let mut st: libc::stat = unsafe { core::mem::zeroed() };
                if io::stat(&path[..len], &mut st) == 0 {
                    io::write_str(1, b"\t");
                    io::write_all(1, name);

                    // Check if attached
                    if is_session_attached(&path[..len]) {
                        io::write_str(1, b"\t(Attached)\n");
                    } else {
                        io::write_str(1, b"\t(Detached)\n");
                    }
                    found = true;
                }
            }

            offset += dirent.d_reclen as usize;
        }
    }

    io::close(fd);

    if !found {
        io::write_str(1, b"No Sockets found.\n");
    }

    0
}

fn is_session_attached(path: &[u8]) -> bool {
    // Check for .attached file
    let mut attached_path = [0u8; 280];
    let mut len = 0;
    for &c in path.iter().take_while(|&&c| c != 0) {
        attached_path[len] = c;
        len += 1;
    }
    for &c in b".attached" {
        attached_path[len] = c;
        len += 1;
    }
    attached_path[len] = 0;

    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    io::stat(&attached_path[..len], &mut st) == 0
}

fn attach_session(name: Option<&[u8]>, _detach_others: bool) -> i32 {
    let fd = io::open(SESSION_DIR, libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 {
        io::write_str(2, b"No sessions to attach to.\n");
        return 1;
    }

    let mut session_path = [0u8; 256];
    let mut found = false;

    let mut buf = [0u8; 4096];

    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }

        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let session_name_found = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if session_name_found != b"." && session_name_found != b".." {
                let matches = match name {
                    Some(n) => session_name_found.starts_with(n) || session_name_found.ends_with(n),
                    None => true, // First session
                };

                if matches {
                    let mut len = 0;
                    for &c in SESSION_DIR.iter().take(SESSION_DIR.len() - 1) {
                        session_path[len] = c;
                        len += 1;
                    }
                    session_path[len] = b'/';
                    len += 1;
                    for &c in session_name_found {
                        session_path[len] = c;
                        len += 1;
                    }
                    session_path[len] = 0;
                    found = true;
                    break;
                }
            }

            offset += dirent.d_reclen as usize;
        }
        if found { break; }
    }

    io::close(fd);

    if !found {
        io::write_str(2, b"No matching sessions.\n");
        return 1;
    }

    // Read session info (PID, PTY path)
    let info_fd = io::open(&session_path, libc::O_RDONLY, 0);
    if info_fd < 0 {
        io::write_str(2, b"Cannot open session info.\n");
        return 1;
    }

    let mut info_buf = [0u8; 256];
    let n = io::read(info_fd, &mut info_buf);
    io::close(info_fd);

    if n <= 0 {
        io::write_str(2, b"Cannot read session info.\n");
        return 1;
    }

    // Parse PID and PTY path from info file
    let info = &info_buf[..n as usize];
    let mut lines = info.split(|&c| c == b'\n');

    let _pid_line = lines.next();
    let pty_line = lines.next();

    let pty_path = match pty_line {
        Some(p) if !p.is_empty() => p,
        _ => {
            io::write_str(2, b"Invalid session info.\n");
            return 1;
        }
    };

    // Mark as attached
    let mut attached_path = [0u8; 280];
    let mut len = 0;
    for &c in session_path.iter().take_while(|&&c| c != 0) {
        attached_path[len] = c;
        len += 1;
    }
    for &c in b".attached" {
        attached_path[len] = c;
        len += 1;
    }
    attached_path[len] = 0;

    let attach_fd = io::open(&attached_path[..len], libc::O_WRONLY | libc::O_CREAT, 0o600);
    if attach_fd >= 0 {
        io::close(attach_fd);
    }

    // Open the PTY master
    let master_fd = io::open(pty_path, libc::O_RDWR, 0);
    if master_fd < 0 {
        io::write_str(2, b"Cannot connect to session.\n");
        return 1;
    }

    io::write_str(1, b"[attached]\n");

    // Run the session relay loop
    let result = run_attached_session(master_fd);

    io::close(master_fd);

    // Remove attached marker
    io::unlink(&attached_path[..len]);

    result
}

fn create_session(name: Option<&[u8]>, argc: i32, argv: *const *const u8, cmd_start: i32) -> i32 {
    // Generate session name if not provided
    let pid = unsafe { libc::getpid() };
    let mut name_buf = [0u8; 64];
    let session_name = match name {
        Some(n) => n,
        None => {
            // Generate name: pid.pts-N.hostname
            let mut len = 0;
            let mut num_buf = [0u8; 16];
            let pid_str = sys::format_u64(pid as u64, &mut num_buf);
            for &c in pid_str {
                name_buf[len] = c;
                len += 1;
            }
            for &c in b".armybox" {
                name_buf[len] = c;
                len += 1;
            }
            &name_buf[..len]
        }
    };

    // Create PTY
    let mut master_fd: i32 = -1;
    let mut slave_fd: i32 = -1;

    if unsafe { libc::openpty(&mut master_fd, &mut slave_fd, core::ptr::null_mut(),
                               core::ptr::null(), core::ptr::null()) } < 0 {
        io::write_str(2, b"screen: cannot create pty\n");
        return 1;
    }

    // Get slave PTY name
    let mut slave_name = [0u8; 64];
    if unsafe { libc::ttyname_r(slave_fd, slave_name.as_mut_ptr() as *mut i8, slave_name.len()) } != 0 {
        io::write_str(2, b"screen: cannot get pty name\n");
        io::close(master_fd);
        io::close(slave_fd);
        return 1;
    }

    // Fork
    let child_pid = unsafe { libc::fork() };

    if child_pid < 0 {
        io::write_str(2, b"screen: fork failed\n");
        io::close(master_fd);
        io::close(slave_fd);
        return 1;
    }

    if child_pid == 0 {
        // Child process - run the shell/command
        io::close(master_fd);

        // Create new session
        unsafe { libc::setsid() };

        // Set controlling terminal
        unsafe { libc::ioctl(slave_fd, libc::TIOCSCTTY as u64, 0) };

        // Redirect stdio
        unsafe {
            libc::dup2(slave_fd, 0);
            libc::dup2(slave_fd, 1);
            libc::dup2(slave_fd, 2);
        }

        if slave_fd > 2 {
            io::close(slave_fd);
        }

        // Execute command or shell
        #[cfg(feature = "alloc")]
        {
            use alloc::ffi::CString;

            if cmd_start < argc {
                // Execute specified command
                let mut args: Vec<CString> = Vec::new();
                for i in cmd_start..argc {
                    if let Some(arg) = unsafe { get_arg(argv, i) } {
                        let mut v = Vec::with_capacity(arg.len() + 1);
                        v.extend_from_slice(arg);
                        v.push(0);
                        if let Ok(cs) = CString::from_vec_with_nul(v) {
                            args.push(cs);
                        }
                    }
                }

                let ptrs: Vec<*const i8> = args.iter()
                    .map(|s| s.as_ptr())
                    .chain(core::iter::once(core::ptr::null()))
                    .collect();

                unsafe { libc::execvp(ptrs[0], ptrs.as_ptr()) };
            } else {
                // Execute default shell
                let shell = b"/bin/sh\0";
                let args = [shell.as_ptr() as *const i8, core::ptr::null()];
                unsafe { libc::execv(shell.as_ptr() as *const i8, args.as_ptr()) };
            }
        }

        #[cfg(not(feature = "alloc"))]
        {
            let shell = b"/bin/sh\0";
            let args = [shell.as_ptr() as *const i8, core::ptr::null()];
            unsafe { libc::execv(shell.as_ptr() as *const i8, args.as_ptr()) };
        }

        unsafe { libc::_exit(127) };
    }

    // Parent process
    io::close(slave_fd);

    // Save session info
    let mut session_file = [0u8; 256];
    let mut len = 0;
    for &c in SESSION_DIR.iter().take(SESSION_DIR.len() - 1) {
        session_file[len] = c;
        len += 1;
    }
    session_file[len] = b'/';
    len += 1;
    for &c in session_name {
        session_file[len] = c;
        len += 1;
    }
    session_file[len] = 0;

    let info_fd = io::open(&session_file[..len], libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o600);
    if info_fd >= 0 {
        // Write PID
        let mut num_buf = [0u8; 16];
        let pid_str = sys::format_u64(child_pid as u64, &mut num_buf);
        io::write_all(info_fd, pid_str);
        io::write_str(info_fd, b"\n");

        // Write PTY master path (for reattaching we need to store master somehow)
        // In a full implementation, we'd use Unix sockets. For simplicity, store slave name
        let slave_len = io::strlen(slave_name.as_ptr());
        io::write_all(info_fd, &slave_name[..slave_len]);
        io::write_str(info_fd, b"\n");

        io::close(info_fd);
    }

    // Mark as attached
    let mut attached_path = [0u8; 280];
    for i in 0..len {
        attached_path[i] = session_file[i];
    }
    for &c in b".attached" {
        attached_path[len] = c;
        len += 1;
    }
    attached_path[len] = 0;

    let attach_fd = io::open(&attached_path[..len], libc::O_WRONLY | libc::O_CREAT, 0o600);
    if attach_fd >= 0 {
        io::close(attach_fd);
    }

    io::write_str(1, b"[screen ");
    io::write_all(1, session_name);
    io::write_str(1, b" started]\n");

    // Run main loop
    let result = run_screen_session(master_fd, child_pid, &session_file);

    // Cleanup
    io::unlink(&attached_path[..len - 9]); // Remove session file
    io::unlink(&attached_path); // Remove attached marker

    io::close(master_fd);

    result
}

fn run_screen_session(master_fd: i32, child_pid: i32, _session_file: &[u8]) -> i32 {
    // Set terminal to raw mode
    let mut orig_termios: libc::termios = unsafe { core::mem::zeroed() };
    let mut raw_termios: libc::termios = unsafe { core::mem::zeroed() };

    unsafe {
        libc::tcgetattr(0, &mut orig_termios);
        raw_termios = orig_termios;
        libc::cfmakeraw(&mut raw_termios);
        libc::tcsetattr(0, libc::TCSANOW, &raw_termios);
    }

    let mut detached = false;
    let mut ctrl_a_pressed = false;

    // Set up polling
    let mut fds = [
        libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 },
        libc::pollfd { fd: master_fd, events: libc::POLLIN, revents: 0 },
    ];

    loop {
        let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, 100) };

        if n < 0 {
            break;
        }

        // Check if child has exited
        let mut status: i32 = 0;
        let wait_result = unsafe { libc::waitpid(child_pid, &mut status, libc::WNOHANG) };
        if wait_result == child_pid {
            break;
        }

        // Read from stdin
        if fds[0].revents & libc::POLLIN != 0 {
            let mut buf = [0u8; 256];
            let n = io::read(0, &mut buf);

            if n > 0 {
                for i in 0..n as usize {
                    let c = buf[i];

                    if ctrl_a_pressed {
                        ctrl_a_pressed = false;
                        match c {
                            b'd' => {
                                // Detach
                                detached = true;
                                break;
                            }
                            b'k' | b'K' => {
                                // Kill window
                                unsafe { libc::kill(child_pid, libc::SIGTERM) };
                            }
                            b'?' => {
                                // Help
                                io::write_str(1, b"\r\n[screen key bindings]\r\n");
                                io::write_str(1, b"  C-a d  detach\r\n");
                                io::write_str(1, b"  C-a k  kill window\r\n");
                                io::write_str(1, b"  C-a ?  this help\r\n");
                                io::write_str(1, b"  C-a a  send C-a\r\n");
                            }
                            b'a' => {
                                // Send literal Ctrl+A
                                io::write_all(master_fd, &[1]);
                            }
                            1 => {
                                // Ctrl+A Ctrl+A - send Ctrl+A
                                io::write_all(master_fd, &[1]);
                            }
                            _ => {
                                // Unknown command, beep
                                io::write_str(1, b"\x07");
                            }
                        }
                    } else if c == 1 { // Ctrl+A
                        ctrl_a_pressed = true;
                    } else {
                        // Pass through to PTY
                        io::write_all(master_fd, &[c]);
                    }
                }

                if detached {
                    break;
                }
            }
        }

        // Read from PTY master
        if fds[1].revents & libc::POLLIN != 0 {
            let mut buf = [0u8; 4096];
            let n = io::read(master_fd, &mut buf);

            if n > 0 {
                io::write_all(1, &buf[..n as usize]);
            } else if n == 0 {
                break;
            }
        }

        // Check for hangup
        if fds[1].revents & libc::POLLHUP != 0 {
            break;
        }
    }

    // Restore terminal
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &orig_termios);
    }

    if detached {
        io::write_str(1, b"\r\n[detached]\r\n");
        0
    } else {
        io::write_str(1, b"\r\n[screen terminated]\r\n");
        0
    }
}

fn run_attached_session(master_fd: i32) -> i32 {
    // Similar to run_screen_session but for reattached sessions
    // Set terminal to raw mode
    let mut orig_termios: libc::termios = unsafe { core::mem::zeroed() };
    let mut raw_termios: libc::termios = unsafe { core::mem::zeroed() };

    unsafe {
        libc::tcgetattr(0, &mut orig_termios);
        raw_termios = orig_termios;
        libc::cfmakeraw(&mut raw_termios);
        libc::tcsetattr(0, libc::TCSANOW, &raw_termios);
    }

    let mut detached = false;
    let mut ctrl_a_pressed = false;

    let mut fds = [
        libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 },
        libc::pollfd { fd: master_fd, events: libc::POLLIN, revents: 0 },
    ];

    loop {
        let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, 100) };

        if n < 0 {
            break;
        }

        // Read from stdin
        if fds[0].revents & libc::POLLIN != 0 {
            let mut buf = [0u8; 256];
            let n = io::read(0, &mut buf);

            if n > 0 {
                for i in 0..n as usize {
                    let c = buf[i];

                    if ctrl_a_pressed {
                        ctrl_a_pressed = false;
                        match c {
                            b'd' => {
                                detached = true;
                                break;
                            }
                            b'?' => {
                                io::write_str(1, b"\r\n[C-a d to detach]\r\n");
                            }
                            b'a' | 1 => {
                                io::write_all(master_fd, &[1]);
                            }
                            _ => {
                                io::write_str(1, b"\x07");
                            }
                        }
                    } else if c == 1 {
                        ctrl_a_pressed = true;
                    } else {
                        io::write_all(master_fd, &[c]);
                    }
                }

                if detached {
                    break;
                }
            }
        }

        // Read from PTY
        if fds[1].revents & libc::POLLIN != 0 {
            let mut buf = [0u8; 4096];
            let n = io::read(master_fd, &mut buf);

            if n > 0 {
                io::write_all(1, &buf[..n as usize]);
            } else if n == 0 {
                break;
            }
        }

        if fds[1].revents & libc::POLLHUP != 0 {
            break;
        }
    }

    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &orig_termios);
    }

    if detached {
        io::write_str(1, b"\r\n[detached]\r\n");
    }

    0
}
