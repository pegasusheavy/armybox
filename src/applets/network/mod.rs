//! Network utilities

use super::get_arg;

// Batch 1: Core networking utilities
mod arp;
mod host;
mod ifconfig;
mod ip;
mod nc;
mod netcat;
mod netstat;
mod nslookup;
mod ping;
mod ping6;
mod route;

// Batch 2: Additional networking utilities
mod ipcalc;
mod ss;
mod traceroute;
mod traceroute6;
mod wget;

// Batch 3: Remaining networking utilities
mod arping;
mod brctl;
mod ether_wake;
mod ftpget;
mod ftpput;
mod httpd;
mod ifdown;
mod ifup;
mod ipaddr;
mod iplink;
mod ipneigh;
mod iproute;
mod iprule;
mod microcom;
mod nameif;
mod nbd_client;
mod nbd_server;
mod slattach;
mod sntp;
mod telnet;
mod tftp;
mod tunctl;
mod vconfig;

// Re-export batch 1 utilities
pub use arp::arp;
pub use host::host;
pub use ifconfig::ifconfig;
pub use ip::ip;
pub use nc::nc;
pub use netcat::netcat;
pub use netstat::netstat;
pub use nslookup::nslookup;
pub use ping::ping;
pub use ping6::ping6;
pub use route::route;

// Re-export batch 2 utilities
pub use ipcalc::ipcalc;
pub use ss::ss;
pub use traceroute::traceroute;
pub use traceroute6::traceroute6;
pub use wget::wget;

// Re-export batch 3 utilities
pub use arping::arping;
pub use brctl::brctl;
pub use ether_wake::ether_wake;
pub use ftpget::ftpget;
pub use ftpput::ftpput;
pub use httpd::httpd;
pub use ifdown::ifdown;
pub use ifup::ifup;
pub use ipaddr::ipaddr;
pub use iplink::iplink;
pub use ipneigh::ipneigh;
pub use iproute::iproute;
pub use iprule::iprule;
pub use microcom::microcom;
pub use nameif::nameif;
pub use nbd_client::nbd_client;
pub use nbd_server::nbd_server;
pub use slattach::slattach;
pub use sntp::sntp;
pub use telnet::telnet;
pub use tftp::tftp;
pub use tunctl::tunctl;
pub use vconfig::vconfig;
