mod fib_trie;
mod if_inet6;
mod net_dev;
mod route;

pub use fib_trie::{FIB_TRIE, parse_fib_trie};
pub use if_inet6::{IF_INET6, parse_if_inet6};
pub use net_dev::{NET_DEV, Traffic, parse_net_dev};
pub use route::{ROUTE, Route, parse_route, written_out};
