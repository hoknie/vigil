mod collectors;
#[cfg(test)]
mod fixture;
#[cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]
mod helpers;
#[cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]
mod parsers;
mod ports;
mod spool;
mod types;

pub use helpers::{
    Redacted, absent, decode, encode_unpadded, hex, holds_files_of_this_host, redact, sha256,
    shown_to_the_agent, split_command, steadied, unescaped,
};
pub use parsers::{PasswdEntry, parse_passwd, parse_passwd_entries};
pub use ports::Collector;
pub use types::{
    COLLECTORS, CollectError, Health, KnownCollector, Presence, collector_names,
    every_seconds_of_collector, is_known_collector, subject_of_collector, unit_of_collector,
};

pub use spool::{
    CEILING_BYTES, Cursor, PLUGIN_CONFIG_PATH, Report, SPOOL_PATH, SpoolWriter, cursor_path,
    dropped_note,
};

#[cfg(target_os = "linux")]
pub use collectors::{
    FilesCollector, LaunchesCollector, PersistenceCollector, ProcessesCollector, ResourcesCollector,
};
