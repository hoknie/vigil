mod configuration;
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod mountinfo;
mod reading;
mod watch_list;

pub use configuration::watching_in;
#[cfg(target_os = "linux")]
pub use mountinfo::mounts_in;
pub use reading::{
    FilesReading, Found, WalkRow, Walked, WatchedDirectory, WatchedFile, files_snapshot,
};
pub use watch_list::watch_list_in;
