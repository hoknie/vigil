mod collectors;
pub mod fixture;
mod modules;
mod parsers;
mod rules;
mod types;
mod views;

pub use modules::Files;
pub use parsers::watching_in;
pub use types::{CEILING_BYTES, MOST_HASHED_BYTES, WATCHED_BY_DEFAULT, Watched, Watching};
pub use views::{TheHostAndItsFiles, asked_for, path_watched};

#[cfg(target_os = "linux")]
pub use collectors::FilesCollector;
