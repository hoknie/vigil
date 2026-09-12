mod reading;
mod snapshot;

#[cfg(test)]
mod tests;

pub use reading::LaunchReading;
pub use snapshot::{any_launch_was_read, launches_snapshot};
