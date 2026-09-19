mod collectors;
pub mod fixture;
mod modules;
#[cfg_attr(
    not(any(target_os = "linux", target_os = "macos")),
    allow(dead_code, unused_imports)
)]
mod parsers;
mod rules;
mod types;
mod views;

pub use collectors::{ProcessesCollector, running, still_running};
pub use modules::Processes;
pub use types::ProcessView;
pub use views::{Running, WhatHasRunHere};
