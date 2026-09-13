mod collectors;
pub mod fixture;
mod modules;
#[cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]
mod parsers;
mod rules;
mod types;
mod views;

pub use modules::{CEILING_BYTES, Files, WATCHED_BY_DEFAULT};
pub use views::TheHostAndItsFiles;

#[cfg(target_os = "linux")]
pub use collectors::FilesCollector;
