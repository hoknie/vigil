#[cfg(test)]
mod tests;

mod first_launch_for_user;
mod launch_finding;
mod launch_from_writable_path;
mod launch_spool_drained;
mod launch_spool_dropping;
mod launch_view;
mod launched_binary_missing;
mod set;

pub use first_launch_for_user::FirstLaunchForUser;
pub use launch_from_writable_path::LaunchFromWritablePath;
pub use launch_spool_drained::LaunchSpoolDrained;
pub use launch_spool_dropping::LaunchSpoolDropping;
pub use launched_binary_missing::LaunchedBinaryMissing;
pub use set::launch_rules;
