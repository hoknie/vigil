#[cfg(test)]
mod tests;

mod macos;
mod persistence;
mod rows;

pub use macos::persistence_on_macos;
pub use persistence::persistence;
pub use rows::{cron_job, kernel_module, launchd_job, preload, script, timer, unit};
