#[cfg(test)]
mod tests;

#[cfg_attr(target_os = "linux", allow(dead_code))]
mod application_firewall;
mod expression;
mod interfaces;
mod ip_tables_names;
#[cfg_attr(target_os = "linux", allow(dead_code))]
mod macos_dump;
#[cfg_attr(target_os = "linux", allow(dead_code))]
mod macos_reading;
mod nft_json;
mod operand;
#[cfg_attr(target_os = "linux", allow(dead_code))]
mod pf;
mod reading;
#[cfg_attr(target_os = "linux", allow(dead_code))]
mod route_messages;

#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use application_firewall::{ApplicationFirewall, parse_application_firewall};
pub use interfaces::{
    FIB_TRIE, IF_INET6, NET_DEV, ROUTE, Route, Traffic, parse_fib_trie, parse_if_inet6,
    parse_net_dev, parse_route, written_out,
};
pub use ip_tables_names::{IP_TABLES_NAMES, IP6_TABLES_NAMES, parse_ip_tables_names};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use macos_dump::{PfRead, Understood, parse_firewall_dump, understood};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use macos_reading::{
    APPLICATION_FIREWALL_ROW, MacosFirewallReading, PF_SUMMARY, Pf, macos_firewall_snapshot,
};
pub use nft_json::{NftRuleset, parse_nft_ruleset};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use pf::{MAIN, PfRuleset, parse_pf_anchors, parse_pf_info, parse_pf_rules};
pub use reading::{FirewallReading, firewall_snapshot};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use route_messages::{ROUTE_MESSAGE_HEADER, parse_default_routes};
