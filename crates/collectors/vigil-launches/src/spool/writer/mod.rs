mod on_disk;
mod report;
mod spool_writer;
#[cfg(test)]
mod tests;
mod writing;
pub use report::Report;

pub use spool_writer::*;
