mod proc_status;
mod table;

pub use proc_status::parse_status;
pub use table::{ProcessRow, ProcessesReading, processes_snapshot};
