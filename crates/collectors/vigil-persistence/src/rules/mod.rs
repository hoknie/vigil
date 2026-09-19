#[cfg(test)]
mod tests;
#[cfg(test)]
mod verdict;

mod cron;
mod kernel_module_loaded;
mod launchd_job_changed;
mod new_launchd_job;
mod new_timer;
mod new_unit;
mod persistence_finding;
mod preload_changed;
mod set;
mod shell_profile_changed;
mod unit_command_changed;

pub use cron::{CronJobChanged, CronJobRemoved, NewCronJob};
pub use kernel_module_loaded::KernelModuleLoaded;
pub use launchd_job_changed::LaunchdJobChanged;
pub use new_launchd_job::NewLaunchdJob;
pub use new_timer::NewTimer;
pub use new_unit::NewUnit;
pub use preload_changed::PreloadChanged;
pub use set::persistence_rules;
pub use shell_profile_changed::ShellProfileChanged;
pub use unit_command_changed::UnitCommandChanged;
