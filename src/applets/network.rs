//! Network utilities

use crate::io;
use super::{get_arg, has_opt};

pub fn wget(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"wget: stub\n"); 0 }
pub fn nc(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"nc: stub\n"); 0 }
pub fn netcat(argc: i32, argv: *const *const u8) -> i32 { nc(argc, argv) }
pub fn ping(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"ping: stub\n"); 0 }
pub fn ping6(argc: i32, argv: *const *const u8) -> i32 { ping(argc, argv) }
pub fn traceroute(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn traceroute6(argc: i32, argv: *const *const u8) -> i32 { traceroute(argc, argv) }
pub fn host(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn nslookup(argc: i32, argv: *const *const u8) -> i32 { host(argc, argv) }
pub fn ifconfig(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(1, b"lo: flags=73<UP,LOOPBACK,RUNNING>\n"); 0 }
pub fn netstat(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn route(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn tftp(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ftpget(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ftpput(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ipcalc(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn brctl(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn tunctl(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ether_wake(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ifup(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ifdown(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ss(argc: i32, argv: *const *const u8) -> i32 { netstat(argc, argv) }
pub fn arp(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn arping(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ip(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn ipaddr(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn iplink(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn ipneigh(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn iproute(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn iprule(argc: i32, argv: *const *const u8) -> i32 { ip(argc, argv) }
pub fn nameif(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn slattach(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn vconfig(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn telnet(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn httpd(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"httpd: stub\n"); 0 }
pub fn sntp(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }
pub fn microcom(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; 0 }

// Additional toybox applets
pub fn nbd_client(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"nbd-client: stub\n"); 0 }
pub fn nbd_server(argc: i32, argv: *const *const u8) -> i32 { let _ = argc; let _ = argv; io::write_str(2, b"nbd-server: stub\n"); 0 }
