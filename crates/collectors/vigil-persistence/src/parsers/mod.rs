#[cfg(test)]
mod tests;

mod crontab;
mod entries;
#[cfg_attr(target_os = "linux", allow(dead_code))]
mod launchd;
mod links;
#[cfg_attr(target_os = "linux", allow(dead_code))]
mod macos_entries;
mod modules;
#[cfg_attr(target_os = "linux", allow(dead_code))]
mod plist;
mod pulled;
mod unit;

pub use crontab::{CronEntry, CronFormat, cron_script, parse_crontab};
pub use entries::{
    PersistenceReading, PreloadFile, ScriptFamily, UnitFile, WatchedScript, persistence_snapshot,
};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use launchd::{LaunchdFacts, launchd_facts};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use macos_entries::{
    Domain, LAUNCHD, LaunchdJob, MacosPersistenceReading, Scope, macos_persistence_snapshot,
};
pub use modules::{KernelModule, parse_modules};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
pub use plist::{PlistRefusal, PlistValue, parse_plist};
pub use unit::parse_unit;
