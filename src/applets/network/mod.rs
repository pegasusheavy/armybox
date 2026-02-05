//! Network utilities

use super::get_arg;

// Batch 1: Core networking utilities
#[cfg(feature = "arp")]
mod arp;
#[cfg(feature = "host")]
mod host;
#[cfg(feature = "ifconfig")]
mod ifconfig;
#[cfg(feature = "ip")]
mod ip;
#[cfg(feature = "nc")]
mod nc;
#[cfg(feature = "nc")]
mod netcat;
#[cfg(feature = "netstat")]
mod netstat;
#[cfg(feature = "nslookup")]
mod nslookup;
#[cfg(feature = "ping")]
mod ping;
#[cfg(feature = "ping")]
mod ping6;
#[cfg(feature = "route")]
mod route;

// Batch 2: Additional networking utilities
#[cfg(feature = "ipcalc")]
mod ipcalc;
#[cfg(feature = "ss")]
mod ss;
#[cfg(feature = "traceroute")]
mod traceroute;
#[cfg(feature = "traceroute")]
mod traceroute6;
#[cfg(feature = "wget")]
mod wget;

// Batch 3: Remaining networking utilities
#[cfg(feature = "arping")]
mod arping;
#[cfg(feature = "brctl")]
mod brctl;
#[cfg(feature = "ether_wake")]
mod ether_wake;
#[cfg(feature = "ftpget")]
mod ftpget;
#[cfg(feature = "ftpput")]
mod ftpput;
#[cfg(feature = "httpd")]
mod httpd;
#[cfg(feature = "ifdown")]
mod ifdown;
#[cfg(feature = "ifup")]
mod ifup;
#[cfg(feature = "ipaddr")]
mod ipaddr;
#[cfg(feature = "iplink")]
mod iplink;
#[cfg(feature = "ipneigh")]
mod ipneigh;
#[cfg(feature = "iproute")]
mod iproute;
#[cfg(feature = "iprule")]
mod iprule;
#[cfg(feature = "microcom")]
mod microcom;
#[cfg(feature = "nameif")]
mod nameif;
#[cfg(feature = "nbd_client")]
mod nbd_client;
#[cfg(feature = "nbd_server")]
mod nbd_server;
#[cfg(feature = "slattach")]
mod slattach;
#[cfg(feature = "sntp")]
mod sntp;
#[cfg(feature = "telnet")]
mod telnet;
#[cfg(feature = "tftp")]
mod tftp;
#[cfg(feature = "tunctl")]
mod tunctl;
#[cfg(feature = "vconfig")]
mod vconfig;

// Re-export batch 1 utilities
#[cfg(feature = "arp")]
pub use arp::arp;
#[cfg(feature = "host")]
pub use host::host;
#[cfg(feature = "ifconfig")]
pub use ifconfig::ifconfig;
#[cfg(feature = "ip")]
pub use ip::ip;
#[cfg(feature = "nc")]
pub use nc::nc;
#[cfg(feature = "nc")]
pub use netcat::netcat;
#[cfg(feature = "netstat")]
pub use netstat::netstat;
#[cfg(feature = "nslookup")]
pub use nslookup::nslookup;
#[cfg(feature = "ping")]
pub use ping::ping;
#[cfg(feature = "ping")]
pub use ping6::ping6;
#[cfg(feature = "route")]
pub use route::route;

// Re-export batch 2 utilities
#[cfg(feature = "ipcalc")]
pub use ipcalc::ipcalc;
#[cfg(feature = "ss")]
pub use ss::ss;
#[cfg(feature = "traceroute")]
pub use traceroute::traceroute;
#[cfg(feature = "traceroute")]
pub use traceroute6::traceroute6;
#[cfg(feature = "wget")]
pub use wget::wget;

// Re-export batch 3 utilities
#[cfg(feature = "arping")]
pub use arping::arping;
#[cfg(feature = "brctl")]
pub use brctl::brctl;
#[cfg(feature = "ether_wake")]
pub use ether_wake::ether_wake;
#[cfg(feature = "ftpget")]
pub use ftpget::ftpget;
#[cfg(feature = "ftpput")]
pub use ftpput::ftpput;
#[cfg(feature = "httpd")]
pub use httpd::httpd;
#[cfg(feature = "ifdown")]
pub use ifdown::ifdown;
#[cfg(feature = "ifup")]
pub use ifup::ifup;
#[cfg(feature = "ipaddr")]
pub use ipaddr::ipaddr;
#[cfg(feature = "iplink")]
pub use iplink::iplink;
#[cfg(feature = "ipneigh")]
pub use ipneigh::ipneigh;
#[cfg(feature = "iproute")]
pub use iproute::iproute;
#[cfg(feature = "iprule")]
pub use iprule::iprule;
#[cfg(feature = "microcom")]
pub use microcom::microcom;
#[cfg(feature = "nameif")]
pub use nameif::nameif;
#[cfg(feature = "nbd_client")]
pub use nbd_client::nbd_client;
#[cfg(feature = "nbd_server")]
pub use nbd_server::nbd_server;
#[cfg(feature = "slattach")]
pub use slattach::slattach;
#[cfg(feature = "sntp")]
pub use sntp::sntp;
#[cfg(feature = "telnet")]
pub use telnet::telnet;
#[cfg(feature = "tftp")]
pub use tftp::tftp;
#[cfg(feature = "tunctl")]
pub use tunctl::tunctl;
#[cfg(feature = "vconfig")]
pub use vconfig::vconfig;
