use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use vigil_model::{Rfc3339, Snapshot};

use crate::helpers::{hex, sha256};
use crate::parsers::{
    CronEntry, CronFormat, PersistenceReading, PreloadFile, ScriptFamily, UnitFile, WatchedScript,
    cron_script, parse_crontab, parse_modules, parse_passwd_entries, parse_unit,
    persistence_snapshot,
};
use crate::{CollectError, Collector, Health};

const UNIT_DIRECTORIES: &[&str] = &[
    "/etc/systemd/system",
    "/run/systemd/system",
    "/usr/local/lib/systemd/system",
    "/usr/lib/systemd/system",
    "/lib/systemd/system",
];

const CRONTAB: &str = "/etc/crontab";
const CRON_DIRECTORY: &str = "/etc/cron.d";

const CRON_SPOOLS: &[&str] = &["/var/spool/cron/crontabs", "/var/spool/cron"];

const CRON_SCRIPT_DIRECTORIES: &[(&str, &str)] = &[
    ("/etc/cron.hourly", "@hourly"),
    ("/etc/cron.daily", "@daily"),
    ("/etc/cron.weekly", "@weekly"),
    ("/etc/cron.monthly", "@monthly"),
];

const PRELOAD: &str = "/etc/ld.so.preload";

const BOOT_SCRIPTS: &[&str] = &["/etc/rc.local", "/etc/rc.d/rc.local"];

const SYSTEM_PROFILES: &[&str] = &[
    "/etc/profile",
    "/etc/bash.bashrc",
    "/etc/bashrc",
    "/etc/zsh/zshrc",
    "/etc/zsh/zprofile",
    "/etc/zshrc",
    "/etc/environment",
];

const PROFILE_DIRECTORY: &str = "/etc/profile.d";

const USER_PROFILES: &[&str] = &[
    ".bashrc",
    ".bash_profile",
    ".bash_login",
    ".bash_logout",
    ".profile",
    ".zshrc",
    ".zprofile",
];

const NON_INTERACTIVE_SHELLS: &[&str] = &[
    "/usr/sbin/nologin",
    "/sbin/nologin",
    "/usr/bin/nologin",
    "/bin/false",
    "/usr/bin/false",
    "",
];

const FILE_LIMIT: u64 = 1024 * 1024;

const DIRECTORY_LIMIT: usize = 4096;

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

fn read_units() -> Vec<UnitFile> {
    let mut units: Vec<UnitFile> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for directory in UNIT_DIRECTORIES {
        for path in sorted_files(Path::new(directory)) {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.contains('.') || !seen.insert(name.to_string()) {
                continue;
            }

            let text = read_capped(&path);
            units.push(UnitFile {
                name: name.to_string(),
                path: path.to_string_lossy().into_owned(),
                readable: text.is_some(),
                facts: text.map(|text| parse_unit(&text)).unwrap_or_default(),
            });
        }
    }

    units
}

fn read_cron() -> Vec<CronEntry> {
    let mut jobs = Vec::new();

    if let Some(text) = read_capped(Path::new(CRONTAB)) {
        jobs.extend(parse_crontab(&text, CRONTAB, CronFormat::WithUser, "root"));
    }

    for path in sorted_files(Path::new(CRON_DIRECTORY)) {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.contains('.') || name.ends_with('~') {
            continue;
        }
        if let Some(text) = read_capped(&path) {
            let source = path.to_string_lossy().into_owned();
            jobs.extend(parse_crontab(&text, &source, CronFormat::WithUser, "root"));
        }
    }

    for spool in CRON_SPOOLS {
        for path in sorted_files(Path::new(spool)) {
            let Some(user) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if let Some(text) = read_capped(&path) {
                let source = path.to_string_lossy().into_owned();
                jobs.extend(parse_crontab(&text, &source, CronFormat::ForOneUser, user));
            }
        }
    }

    for (directory, schedule) in CRON_SCRIPT_DIRECTORIES {
        for path in sorted_files(Path::new(directory)) {
            jobs.push(cron_script(directory, &path.to_string_lossy(), schedule));
        }
    }

    jobs
}

fn read_modules() -> Option<Vec<crate::parsers::KernelModule>> {
    fs::read_to_string("/proc/modules")
        .ok()
        .map(|text| parse_modules(&text))
}

fn read_scripts() -> Vec<WatchedScript> {
    let mut scripts = Vec::new();

    for path in BOOT_SCRIPTS {
        scripts.push(describe_script(Path::new(path), ScriptFamily::Boot));
    }
    for path in SYSTEM_PROFILES {
        scripts.push(describe_script(Path::new(path), ScriptFamily::Profile));
    }
    for path in sorted_files(Path::new(PROFILE_DIRECTORY)) {
        scripts.push(describe_script(&path, ScriptFamily::Profile));
    }

    if let Ok(text) = fs::read_to_string("/etc/passwd") {
        for entry in parse_passwd_entries(&text) {
            if entry.home.is_empty() || NON_INTERACTIVE_SHELLS.contains(&entry.shell.as_str()) {
                continue;
            }
            for name in USER_PROFILES {
                let path = Path::new(&entry.home).join(name);
                if path.exists() {
                    scripts.push(describe_script(&path, ScriptFamily::Profile));
                }
            }
        }
    }

    scripts
}

fn describe_script(path: &Path, family: ScriptFamily) -> WatchedScript {
    let mut script = WatchedScript {
        path: path.to_string_lossy().into_owned(),
        family,
        present: false,
        readable: true,
        digest: None,
        size: 0,
        mode: String::new(),
        uid: 0,
        gid: 0,
    };

    let Ok(metadata) = fs::metadata(path) else {
        return script;
    };
    script.present = true;
    script.size = metadata.len();
    script.mode = format!("{:04o}", metadata.permissions().mode() & 0o7777);
    script.uid = metadata.uid();
    script.gid = metadata.gid();

    match read_capped(path) {
        Some(text) => script.digest = Some(hex(&sha256(text.as_bytes()))),
        None => script.readable = false,
    }

    script
}

fn read_preload() -> PreloadFile {
    let mut preload = PreloadFile {
        path: PRELOAD.to_string(),
        present: false,
        readable: true,
        entries: Vec::new(),
        digest: None,
    };

    match fs::read_to_string(PRELOAD) {
        Ok(text) => {
            preload.present = true;
            preload.digest = Some(hex(&sha256(text.as_bytes())));
            preload.entries = text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_string)
                .collect();
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => {
            preload.present = true;
            preload.readable = false;
        }
    }

    preload
}

fn sorted_files(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .take(DIRECTORY_LIMIT)
        .collect();
    files.sort();

    files
}

fn read_capped(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > FILE_LIMIT {
        return None;
    }
    fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_this_host_and_names_itself_in_the_snapshot() {
        let collector = PersistenceCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

        let snapshot = collector.collect().expect("something is always readable");

        assert_eq!(snapshot.source, "persistence");
        assert_eq!(snapshot.taken_at, "2026-09-09T12:00:00.000Z");
        assert!(
            snapshot.items.contains_key("preload|/etc/ld.so.preload"),
            "the preload file is an item whether or not it exists"
        );
        for key in snapshot.items.keys() {
            assert!(key.contains('|'), "key shape: {key}");
        }
    }

    #[test]
    fn a_preload_file_that_is_not_there_is_recorded_as_not_being_there() {
        let preload = read_preload();

        assert_eq!(preload.path, PRELOAD);
        assert!(preload.present || preload.readable);
    }
}
