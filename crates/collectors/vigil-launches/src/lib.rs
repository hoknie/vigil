mod collectors;
pub mod fixture;
mod modules;
#[cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]
mod parsers;
mod rules;
mod spool;
mod types;
mod views;

pub use modules::Launches;
pub use spool::{
    CEILING_BYTES, Cursor, PLUGIN_CONFIG_PATH, Report, SPOOL_PATH, SpoolWriter, cursor_path,
    dropped_note,
};
pub use views::{Launches as LaunchesPane, WhatHasRunHere};

#[cfg(target_os = "linux")]
pub use collectors::LaunchesCollector;
