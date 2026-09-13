#[cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]
mod helpers;
#[cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]
mod parsers;
mod ports;
mod types;

pub use helpers::{
    Redacted, absent, decode, encode_unpadded, hex, holds_files_of_this_host, redact, sha256,
    shown_to_the_agent, split_command, steadied, unescaped,
};
pub use parsers::{PasswdEntry, parse_passwd, parse_passwd_entries};
pub use ports::Collector;
pub use types::{CollectError, Health, KnownCollector, Presence};
