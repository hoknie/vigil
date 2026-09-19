#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod proc_status;
mod table;

#[cfg(target_os = "linux")]
pub use proc_status::parse_status;
#[cfg(target_os = "macos")]
pub use table::processes_snapshot_on;
pub use table::{ProcessRow, ProcessesReading, processes_snapshot};
