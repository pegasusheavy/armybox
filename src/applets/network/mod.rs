//! Network utilities

use crate::io;
use crate::sys;
use super::get_arg;

pub fn ping(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"ping: missing host\n");
        return 1;
    }

    let host = unsafe { get_arg(argv, argc - 1).unwrap() };
    let mut count = 4i32;

    // Parse -c option
    for i in 1..argc-1 {
        if let Some(arg) = unsafe { get_arg(argv, i) } {
            if arg == b"-c" && i + 1 < argc - 1 {
                if let Some(c) = unsafe { get_arg(argv, i + 1) } {
                    count = sys::parse_i64(c).unwrap_or(4) as i32;
                }
            }
        }
    }

    io::write_str(1, b"PING ");
    io::write_all(1, host);
    io::write_str(1, b" 56 data bytes\n");

    // Create raw socket for ICMP
    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_ICMP) };
    if sock < 0 {
        io::write_str(2, b"ping: socket: Operation not permitted\n");
        return 1;
    }

    // Resolve hostname
    let mut host_buf = [0u8; 256];
    host_buf[..host.len()].copy_from_slice(host);

    let mut hints: libc::addrinfo = unsafe { core::mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    let mut res: *mut libc::addrinfo = core::ptr::null_mut();

    if unsafe { libc::getaddrinfo(host_buf.as_ptr() as *const i8, core::ptr::null(), &hints, &mut res) } != 0 {
        io::write_str(2, b"ping: unknown host\n");
        unsafe { libc::close(sock) };
        return 1;
    }

    let addr = unsafe { *((*res).ai_addr as *const libc::sockaddr_in) };
    unsafe { libc::freeaddrinfo(res) };

    let mut sent = 0;
    let mut received = 0;

    for seq in 0..count {
        // Build ICMP echo request
        let mut packet = [0u8; 64];
        packet[0] = 8; // ICMP Echo Request
        packet[1] = 0; // Code
        // Checksum at 2-3, will compute
        packet[4] = 0; packet[5] = 1; // ID
        packet[6] = (seq >> 8) as u8; packet[7] = seq as u8; // Sequence

        // Compute checksum
        let mut sum: u32 = 0;
        for i in (0..64).step_by(2) {
            sum += ((packet[i] as u32) << 8) | (packet[i+1] as u32);
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }
        let checksum = !sum as u16;
        packet[2] = (checksum >> 8) as u8;
        packet[3] = checksum as u8;

        let start = unsafe { libc::time(core::ptr::null_mut()) };

        let sent_bytes = unsafe {
            libc::sendto(
                sock,
                packet.as_ptr() as *const libc::c_void,
                64,
                0,
                &addr as *const _ as *const libc::sockaddr,
                core::mem::size_of::<libc::sockaddr_in>() as u32,
            )
        };

        if sent_bytes > 0 {
            sent += 1;

            // Set receive timeout
            let tv = libc::timeval { tv_sec: 1, tv_usec: 0 };
            unsafe { libc::setsockopt(sock, libc::SOL_SOCKET, libc::SO_RCVTIMEO, &tv as *const _ as *const libc::c_void, core::mem::size_of::<libc::timeval>() as u32) };

            let mut recv_buf = [0u8; 128];
            let mut from: libc::sockaddr_in = unsafe { core::mem::zeroed() };
            let mut fromlen = core::mem::size_of::<libc::sockaddr_in>() as u32;

            let recv_bytes = unsafe {
                libc::recvfrom(
                    sock,
                    recv_buf.as_mut_ptr() as *mut libc::c_void,
                    recv_buf.len(),
                    0,
                    &mut from as *mut _ as *mut libc::sockaddr,
                    &mut fromlen,
                )
            };

            let end = unsafe { libc::time(core::ptr::null_mut()) };
            let rtt = (end - start) * 1000; // Approximate ms

            if recv_bytes > 0 {
                received += 1;
                io::write_str(1, b"64 bytes from ");
                io::write_all(1, host);
                io::write_str(1, b": icmp_seq=");
                let mut num_buf = [0u8; 20];
                io::write_all(1, sys::format_u64((seq + 1) as u64, &mut num_buf));
                io::write_str(1, b" time=");
                io::write_all(1, sys::format_u64(rtt as u64, &mut num_buf));
                io::write_str(1, b" ms\n");
            }
        }

        if seq < count - 1 {
            unsafe { libc::sleep(1) };
        }
    }

    unsafe { libc::close(sock) };

    // Print statistics
    io::write_str(1, b"\n--- ");
    io::write_all(1, host);
    io::write_str(1, b" ping statistics ---\n");
    let mut num_buf = [0u8; 20];
    io::write_all(1, sys::format_u64(sent as u64, &mut num_buf));
    io::write_str(1, b" packets transmitted, ");
    io::write_all(1, sys::format_u64(received as u64, &mut num_buf));
    io::write_str(1, b" received, ");
    let loss = if sent > 0 { ((sent - received) * 100 / sent) as u64 } else { 100 };
    io::write_all(1, sys::format_u64(loss, &mut num_buf));
    io::write_str(1, b"% packet loss\n");

    if received > 0 { 0 } else { 1 }
}

pub fn ping6(argc: i32, argv: *const *const u8) -> i32 { ping(argc, argv) }

pub fn ifconfig(argc: i32, argv: *const *const u8) -> i32 {
    // Read from /sys/class/net
    let fd = io::open(b"/sys/class/net", libc::O_RDONLY | libc::O_DIRECTORY, 0);
    if fd < 0 {
        io::write_str(2, b"ifconfig: cannot read interfaces\n");
        return 1;
    }

    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 { break; }
        let mut offset = 0;
        while offset < n as usize {
            let dirent = unsafe { &*(buf.as_ptr().add(offset) as *const libc::dirent64) };
            let name = unsafe { io::cstr_to_slice(dirent.d_name.as_ptr() as *const u8) };

            if name != b"." && name != b".." {
                // If specific interface requested, filter
                if argc > 1 {
                    let req = unsafe { get_arg(argv, 1).unwrap() };
                    if name != req {
                        offset += dirent.d_reclen as usize;
                        continue;
                    }
                }

                io::write_all(1, name);
                io::write_str(1, b": ");

                // Read flags
                let mut path = [0u8; 128];
                let mut pi = 0;
                for &c in b"/sys/class/net/" { path[pi] = c; pi += 1; }
                for &c in name { path[pi] = c; pi += 1; }

                let path_base = pi;

                // Read operstate
                for &c in b"/operstate\0" { path[pi] = c; pi += 1; }
                let state_fd = io::open(&path, libc::O_RDONLY, 0);
                if state_fd >= 0 {
                    let mut state_buf = [0u8; 32];
                    let sn = io::read(state_fd, &mut state_buf);
                    io::close(state_fd);
                    if sn > 0 {
                        let state = &state_buf[..sn as usize];
                        let state = state.split(|&c| c == b'\n').next().unwrap_or(state);
                        io::write_str(1, b"flags=<");
                        if state == b"up" {
                            io::write_str(1, b"UP,RUNNING");
                        } else {
                            io::write_str(1, b"DOWN");
                        }
                        io::write_str(1, b">\n");
                    }
                }

                // Read address
                pi = path_base;
                for &c in b"/address\0" { path[pi] = c; pi += 1; }
                let addr_fd = io::open(&path, libc::O_RDONLY, 0);
                if addr_fd >= 0 {
                    let mut addr_buf = [0u8; 64];
                    let an = io::read(addr_fd, &mut addr_buf);
                    io::close(addr_fd);
                    if an > 0 {
                        io::write_str(1, b"        ether ");
                        io::write_all(1, &addr_buf[..an as usize - 1]); // Remove newline
                        io::write_str(1, b"\n");
                    }
                }

                // Read MTU
                pi = path_base;
                for &c in b"/mtu\0" { path[pi] = c; pi += 1; }
                let mtu_fd = io::open(&path, libc::O_RDONLY, 0);
                if mtu_fd >= 0 {
                    let mut mtu_buf = [0u8; 16];
                    let mn = io::read(mtu_fd, &mut mtu_buf);
                    io::close(mtu_fd);
                    if mn > 0 {
                        io::write_str(1, b"        mtu ");
                        io::write_all(1, &mtu_buf[..mn as usize - 1]);
                        io::write_str(1, b"\n");
                    }
                }

                io::write_str(1, b"\n");
            }
            offset += dirent.d_reclen as usize;
        }
    }
    io::close(fd);
    0
}

pub fn netstat(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Active Internet connections (servers and established)\n");
    io::write_str(1, b"Proto Recv-Q Send-Q Local Address           Foreign Address         State\n");

    // Read /proc/net/tcp
    let fd = io::open(b"/proc/net/tcp", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);
        if n > 0 {
            let content = &buf[..n as usize];
            for (i, line) in content.split(|&c| c == b'\n').enumerate() {
                if i == 0 { continue; } // Skip header
                if line.is_empty() { continue; }
                io::write_str(1, b"tcp    0      0 ");
                // Parse and format the line (simplified)
                io::write_all(1, line);
                io::write_str(1, b"\n");
            }
        }
    }
    0
}

pub fn route(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Kernel IP routing table\n");
    io::write_str(1, b"Destination     Gateway         Genmask         Flags Metric Ref    Use Iface\n");

    let fd = io::open(b"/proc/net/route", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);
        if n > 0 {
            let content = &buf[..n as usize];
            for (i, line) in content.split(|&c| c == b'\n').enumerate() {
                if i == 0 { continue; } // Skip header
                if line.is_empty() { continue; }
                io::write_all(1, line);
                io::write_str(1, b"\n");
            }
        }
    }
    0
}

pub fn ip(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"Usage: ip OBJECT { COMMAND }\n");
        io::write_str(2, b"OBJECT := { link | addr | route | neigh }\n");
        return 1;
    }

    let obj = unsafe { get_arg(argv, 1).unwrap() };

    if obj == b"link" || obj == b"l" {
        return ifconfig(1, argv);
    }

    if obj == b"addr" || obj == b"a" {
        return ifconfig(1, argv);
    }

    if obj == b"route" || obj == b"r" {
        return route(1, argv);
    }

    if obj == b"neigh" || obj == b"n" {
        return arp(1, argv);
    }

    io::write_str(2, b"ip: unknown object\n");
    1
}

pub fn arp(_argc: i32, _argv: *const *const u8) -> i32 {
    io::write_str(1, b"Address                  HWtype  HWaddress           Flags Mask            Iface\n");

    let fd = io::open(b"/proc/net/arp", libc::O_RDONLY, 0);
    if fd >= 0 {
        let mut buf = [0u8; 4096];
        let n = io::read(fd, &mut buf);
        io::close(fd);
        if n > 0 {
            let content = &buf[..n as usize];
            for (i, line) in content.split(|&c| c == b'\n').enumerate() {
                if i == 0 { continue; } // Skip header
                if line.is_empty() { continue; }
                io::write_all(1, line);
                io::write_str(1, b"\n");
            }
        }
    }
    0
}

pub fn host(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"host: missing hostname\n");
        return 1;
    }

    let hostname = unsafe { get_arg(argv, 1).unwrap() };
    let mut host_buf = [0u8; 256];
    host_buf[..hostname.len()].copy_from_slice(hostname);

    let mut hints: libc::addrinfo = unsafe { core::mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    let mut res: *mut libc::addrinfo = core::ptr::null_mut();

    if unsafe { libc::getaddrinfo(host_buf.as_ptr() as *const i8, core::ptr::null(), &hints, &mut res) } != 0 {
        io::write_str(2, b"host: ");
        io::write_all(2, hostname);
        io::write_str(2, b" not found\n");
        return 1;
    }

    io::write_all(1, hostname);
    io::write_str(1, b" has address ");

    let mut current = res;
    while !current.is_null() {
        let info = unsafe { &*current };
        if info.ai_family == libc::AF_INET {
            let addr = unsafe { &*(info.ai_addr as *const libc::sockaddr_in) };
            let ip = addr.sin_addr.s_addr.to_be();
            let mut num_buf = [0u8; 20];
            io::write_all(1, sys::format_u64(((ip >> 24) & 0xff) as u64, &mut num_buf));
            io::write_str(1, b".");
            io::write_all(1, sys::format_u64(((ip >> 16) & 0xff) as u64, &mut num_buf));
            io::write_str(1, b".");
            io::write_all(1, sys::format_u64(((ip >> 8) & 0xff) as u64, &mut num_buf));
            io::write_str(1, b".");
            io::write_all(1, sys::format_u64((ip & 0xff) as u64, &mut num_buf));
            io::write_str(1, b"\n");
            break;
        }
        current = info.ai_next;
    }

    unsafe { libc::freeaddrinfo(res) };
    0
}

pub fn nslookup(argc: i32, argv: *const *const u8) -> i32 { host(argc, argv) }

pub fn nc(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 3 {
        io::write_str(2, b"nc: usage: nc HOST PORT\n");
        return 1;
    }

    let host = unsafe { get_arg(argv, 1).unwrap() };
    let port = unsafe { get_arg(argv, 2).unwrap() };

    let mut host_buf = [0u8; 256];
    let mut port_buf = [0u8; 16];
    host_buf[..host.len()].copy_from_slice(host);
    port_buf[..port.len()].copy_from_slice(port);

    let mut hints: libc::addrinfo = unsafe { core::mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_STREAM;
    let mut res: *mut libc::addrinfo = core::ptr::null_mut();

    if unsafe { libc::getaddrinfo(host_buf.as_ptr() as *const i8, port_buf.as_ptr() as *const i8, &hints, &mut res) } != 0 {
        io::write_str(2, b"nc: cannot resolve host\n");
        return 1;
    }

    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    if sock < 0 {
        unsafe { libc::freeaddrinfo(res) };
        io::write_str(2, b"nc: socket failed\n");
        return 1;
    }

    let info = unsafe { &*res };
    if unsafe { libc::connect(sock, info.ai_addr, info.ai_addrlen) } < 0 {
        unsafe { libc::close(sock); libc::freeaddrinfo(res) };
        io::write_str(2, b"nc: connection failed\n");
        return 1;
    }

    unsafe { libc::freeaddrinfo(res) };

    // Simple relay: stdin -> socket, socket -> stdout
    let mut buf = [0u8; 4096];

    // Set socket non-blocking
    unsafe { libc::fcntl(sock, libc::F_SETFL, libc::O_NONBLOCK) };
    unsafe { libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK) };

    loop {
        // Read from stdin, write to socket
        let n = io::read(0, &mut buf);
        if n > 0 {
            let _ = unsafe { libc::send(sock, buf.as_ptr() as *const libc::c_void, n as usize, 0) };
        }

        // Read from socket, write to stdout
        let n = unsafe { libc::recv(sock, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0) };
        if n > 0 {
            io::write_all(1, &buf[..n as usize]);
        } else if n == 0 {
            break; // Connection closed
        }

        unsafe { libc::usleep(10000) }; // 10ms
    }

    unsafe { libc::close(sock) };
    0
}

pub fn netcat(argc: i32, argv: *const *const u8) -> i32 { nc(argc, argv) }

pub fn wget(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"wget: missing URL\n");
        return 1;
    }

    let url = unsafe { get_arg(argv, argc - 1).unwrap() };

    // Parse URL (simplified - only http)
    if !url.starts_with(b"http://") {
        io::write_str(2, b"wget: only http:// URLs supported\n");
        return 1;
    }

    let url_rest = &url[7..]; // Skip "http://"

    // Find host and path
    let (host_port, path) = if let Some(pos) = url_rest.iter().position(|&c| c == b'/') {
        (&url_rest[..pos], &url_rest[pos..])
    } else {
        (url_rest, b"/".as_slice())
    };

    // Parse host:port
    let (host, port) = if let Some(pos) = host_port.iter().position(|&c| c == b':') {
        (&host_port[..pos], &host_port[pos+1..])
    } else {
        (host_port, b"80".as_slice())
    };

    // Connect
    let mut host_buf = [0u8; 256];
    let mut port_buf = [0u8; 16];
    host_buf[..host.len()].copy_from_slice(host);
    port_buf[..port.len()].copy_from_slice(port);

    let mut hints: libc::addrinfo = unsafe { core::mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_STREAM;
    let mut res: *mut libc::addrinfo = core::ptr::null_mut();

    if unsafe { libc::getaddrinfo(host_buf.as_ptr() as *const i8, port_buf.as_ptr() as *const i8, &hints, &mut res) } != 0 {
        io::write_str(2, b"wget: cannot resolve host\n");
        return 1;
    }

    let sock = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    if sock < 0 {
        unsafe { libc::freeaddrinfo(res) };
        return 1;
    }

    let info = unsafe { &*res };
    if unsafe { libc::connect(sock, info.ai_addr, info.ai_addrlen) } < 0 {
        unsafe { libc::close(sock); libc::freeaddrinfo(res) };
        io::write_str(2, b"wget: connection failed\n");
        return 1;
    }

    unsafe { libc::freeaddrinfo(res) };

    // Send HTTP request
    let mut request = [0u8; 1024];
    let mut ri = 0;
    for &c in b"GET " { request[ri] = c; ri += 1; }
    for &c in path { request[ri] = c; ri += 1; }
    for &c in b" HTTP/1.0\r\nHost: " { request[ri] = c; ri += 1; }
    for &c in host { request[ri] = c; ri += 1; }
    for &c in b"\r\nConnection: close\r\n\r\n" { request[ri] = c; ri += 1; }

    let _ = unsafe { libc::send(sock, request.as_ptr() as *const libc::c_void, ri, 0) };

    // Determine output filename
    let filename = if let Some(pos) = path.iter().rposition(|&c| c == b'/') {
        if pos + 1 < path.len() { &path[pos+1..] } else { b"index.html" }
    } else { b"index.html" };

    let out_fd = io::open(filename, libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644);
    if out_fd < 0 {
        unsafe { libc::close(sock) };
        io::write_str(2, b"wget: cannot create output file\n");
        return 1;
    }

    // Receive response
    let mut buf = [0u8; 4096];
    let mut header_done = false;
    let mut body_start = 0usize;

    loop {
        let n = unsafe { libc::recv(sock, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0) };
        if n <= 0 { break; }

        let data = &buf[..n as usize];

        if !header_done {
            // Find end of headers
            for i in 0..data.len().saturating_sub(3) {
                if data[i..].starts_with(b"\r\n\r\n") {
                    header_done = true;
                    body_start = i + 4;
                    break;
                }
            }
            if header_done && body_start < data.len() {
                io::write_all(out_fd, &data[body_start..]);
            }
        } else {
            io::write_all(out_fd, data);
        }
    }

    io::close(out_fd);
    unsafe { libc::close(sock) };

    io::write_str(2, b"'");
    io::write_all(2, filename);
    io::write_str(2, b"' saved\n");
    0
}

pub fn traceroute(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 {
        io::write_str(2, b"traceroute: missing host\n");
        return 1;
    }
    io::write_str(2, b"traceroute: not implemented\n");
    1
}
pub fn traceroute6(argc: i32, argv: *const *const u8) -> i32 { traceroute(argc, argv) }
pub fn tftp(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"tftp: not implemented\n"); 1 }
pub fn ftpget(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"ftpget: not implemented\n"); 1 }
pub fn ftpput(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"ftpput: not implemented\n"); 1 }

pub fn ipcalc(argc: i32, argv: *const *const u8) -> i32 {
    if argc < 2 { return 1; }
    let addr = unsafe { get_arg(argv, 1).unwrap() };
    io::write_str(1, b"Address: ");
    io::write_all(1, addr);
    io::write_str(1, b"\n");
    0
}

pub fn brctl(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"brctl: not implemented\n"); 1 }
pub fn tunctl(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"tunctl: not implemented\n"); 1 }
pub fn ether_wake(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"ether-wake: not implemented\n"); 1 }
pub fn ifup(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"ifup: not implemented\n"); 1 }
pub fn ifdown(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"ifdown: not implemented\n"); 1 }
pub fn ss(argc: i32, argv: *const *const u8) -> i32 { netstat(argc, argv) }
pub fn arping(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"arping: not implemented\n"); 1 }
pub fn ipaddr(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn iplink(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn ipneigh(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn iproute(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn iprule(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn nameif(_argc: i32, _argv: *const *const u8) -> i32 { 0 }
pub fn slattach(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"slattach: not implemented\n"); 1 }
pub fn vconfig(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"vconfig: not implemented\n"); 1 }
pub fn telnet(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"telnet: not implemented\n"); 1 }
pub fn httpd(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"httpd: not implemented\n"); 1 }
pub fn sntp(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"sntp: not implemented\n"); 1 }
pub fn microcom(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"microcom: not implemented\n"); 1 }
pub fn nbd_client(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"nbd-client: not implemented\n"); 1 }
pub fn nbd_server(_argc: i32, _argv: *const *const u8) -> i32 { io::write_str(2, b"nbd-server: not implemented\n"); 1 }
