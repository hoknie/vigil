mod cron;
mod files;
mod modules;
mod preload;
mod scripts;
mod units;

#[cfg(test)]
mod tests;

use std::fs;
use std::io::ErrorKind;

use vigil_model::{Rfc3339, Snapshot};

use crate::parsers::{PersistenceReading, persistence_snapshot};
use vigil_collect::{CollectError, Collector, Health};

use cron::read_cron;
use modules::read_modules;
use preload::read_preload;
use scripts::read_scripts;
use units::read_units;

pub(super) const UNIT_DIRECTORIES: &[&str] = &[
    "/etc/systemd/system",
    "/run/systemd/system",
    "/usr/local/lib/systemd/system",
    "/usr/lib/systemd/system",
    "/lib/systemd/system",
];

pub(super) const CRONTAB: &str = "/etc/crontab";
pub(super) const CRON_DIRECTORY: &str = "/etc/cron.d";

pub(super) const CRON_SPOOLS: &[&str] = &["/var/spool/cron/crontabs", "/var/spool/cron"];

pub(super) const CRON_SCRIPT_DIRECTORIES: &[(&str, &str)] = &[
    ("/etc/cron.hourly", "@hourly"),
    ("/etc/cron.daily", "@daily"),
    ("/etc/cron.weekly", "@weekly"),
    ("/etc/cron.monthly", "@monthly"),
];

pub(super) const PRELOAD: &str = "/etc/ld.so.preload";

pub(super) const BOOT_SCRIPTS: &[&str] = &["/etc/rc.local", "/etc/rc.d/rc.local"];

pub(super) const SYSTEM_PROFILES: &[&str] = &[
    "/etc/profile",
    "/etc/bash.bashrc",
    "/etc/bashrc",
    "/etc/zsh/zshrc",
    "/etc/zsh/zprofile",
    "/etc/zshrc",
    "/etc/environment",
];

pub(super) const PROFILE_DIRECTORY: &str = "/etc/profile.d";

pub(super) const USER_PROFILES: &[&str] = &[
    ".bashrc",
    ".bash_profile",
    ".bash_login",
    ".bash_logout",
    ".profile",
    ".zshrc",
    ".zprofile",
];

pub(super) const NON_INTERACTIVE_SHELLS: &[&str] = &[
    "/usr/sbin/nologin",
    "/sbin/nologin",
    "/usr/bin/nologin",
    "/bin/false",
    "/usr/bin/false",
    "",
];

pub(super) const FILE_LIMIT: u64 = 1024 * 1024;

pub(super) const DIRECTORY_LIMIT: usize = 4096;

pub struct PersistenceCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl PersistenceCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        PersistenceCollector { now: Box::new(now) }
    }
}

impl Collector for PersistenceCollector {
    fn name(&self) -> &'static str {
        "persistence"
    }

    fn available(&self) -> Health {
        let mut missing: Vec<String> = Vec::new();
        let mut sources = 0usize;

        for directory in UNIT_DIRECTORIES
            .iter()
            .chain(std::iter::once(&CRON_DIRECTORY))
            .chain(CRON_SPOOLS)
            .chain(CRON_SCRIPT_DIRECTORIES.iter().map(|(path, _)| path))
        {
            match fs::read_dir(directory) {
                Ok(_) => sources += 1,
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) if error.kind() == ErrorKind::PermissionDenied => missing.push(format!(
                    "{directory} exists and cannot be listed: what is started from it will not be seen; run as root"
                )),
                Err(error) => missing.push(format!("{directory} could not be listed ({error})")),
            }
        }

        match fs::read_to_string("/proc/modules") {
            Ok(_) => sources += 1,
            Err(error) if error.kind() == ErrorKind::NotFound => {
            }
            Err(_) => missing.push(
                "/proc/modules could not be read: a module loaded into this kernel will not be seen"
                    .into(),
            ),
        }

        if sources == 0 && missing.is_empty() {
            return Health::Unavailable(
                "no systemd unit directory, no cron and no /proc/modules: nothing to watch".into(),
            );
        }

        match missing.is_empty() {
            true => Health::Ok,
            false => Health::Degraded(missing.join("; ")),
        }
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let units = read_units();
        let cron = read_cron();
        let modules = read_modules();
        let scripts = read_scripts();
        let preload = read_preload();

        if units.is_empty()
            && cron.is_empty()
            && modules.is_none()
            && scripts.is_empty()
            && !preload.readable
        {
            return Err(CollectError::Denied(
                "none of the places this host starts things from could be read".into(),
            ));
        }

        Ok(persistence_snapshot(
            &(self.now)(),
            &PersistenceReading {
                units: &units,
                cron: &cron,
                modules: modules.as_deref(),
                scripts: &scripts,
                preload: &preload,
            },
        ))
    }
}
