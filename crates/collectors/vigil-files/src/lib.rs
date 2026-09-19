mod collectors;
pub mod fixture;
#[cfg_attr(
    not(any(target_os = "linux", target_os = "macos")),
    allow(dead_code, unused_imports)
)]
mod helpers;
mod modules;
mod parsers;
mod rules;
mod types;
mod views;

pub use collectors::FilesCollector;
pub use modules::Files;
pub use parsers::{watch_list_in, watching_in};
pub use types::{
    CEILING_BYTES, Devices, Layout, Listing, MAX_FILES, MOST_FILES, MOST_HASHED_BYTES, Named, Size,
    WATCHED_BY_DEFAULT, WatchList, Watched, Watching,
};
pub use views::{TheHostAndItsFiles, asked_for, path_watched};
