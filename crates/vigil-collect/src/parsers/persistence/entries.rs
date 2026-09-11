use serde_json::json;
use vigil_model::Snapshot;

use super::crontab::CronEntry;
use super::modules::KernelModule;
use super::unit::UnitFacts;

pub const SOURCE: &str = "persistence";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitFile {
    pub name: String,
    pub path: String,
    pub readable: bool,
    pub facts: UnitFacts,
}

impl UnitFile {
    pub fn kind(&self) -> &str {
        self.name.rsplit_once('.').map(|(_, k)| k).unwrap_or("unit")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptFamily {
    Profile,
    Boot,
}

impl ScriptFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            ScriptFamily::Profile => "profile",
            ScriptFamily::Boot => "boot",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchedScript {
    pub path: String,
    pub family: ScriptFamily,
    pub present: bool,
    pub readable: bool,
    pub digest: Option<String>,
    pub size: u64,
    pub mode: String,
    pub uid: u32,
    pub gid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreloadFile {
    pub path: String,
    pub present: bool,
    pub readable: bool,
    pub entries: Vec<String>,
    pub digest: Option<String>,
}

pub struct PersistenceReading<'a> {
    pub units: &'a [UnitFile],
    pub cron: &'a [CronEntry],
    pub modules: Option<&'a [KernelModule]>,
    pub scripts: &'a [WatchedScript],
    pub preload: &'a PreloadFile,
}

pub const MODULES_UNREADABLE: &str = "modules|unreadable";

pub fn persistence_snapshot(taken_at: &str, reading: &PersistenceReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    add_units(&mut snapshot, reading);
    add_cron(&mut snapshot, reading);
    add_modules(&mut snapshot, reading);
    add_scripts(&mut snapshot, reading);
    add_preload(&mut snapshot, reading);

    snapshot
}

fn add_units(snapshot: &mut Snapshot, reading: &PersistenceReading<'_>) {
    for unit in reading.units {
        let (prefix, value) = match unit.kind() {
            "timer" => (
                "timer",
                json!({
                    "name": unit.name,
                    "type": "timer",
                    "path": unit.path,
                    "readable": unit.readable,
                    "description": unit.facts.description,
                    "on_calendar": unit.facts.on_calendar,
                    "on_boot": unit.facts.on_boot,
                    "activates": unit.facts.activates.clone().unwrap_or_else(|| {
                        unit.name.trim_end_matches(".timer").to_string() + ".service"
                    }),
                }),
            ),
            kind => (
                "unit",
                json!({
                    "name": unit.name,
                    "type": kind,
                    "path": unit.path,
                    "readable": unit.readable,
                    "description": unit.facts.description,
                    "commands": unit.facts.commands,
                    "commands_redacted": unit.facts.commands_redacted,
                    "run_as": unit.facts.run_as.clone().unwrap_or_else(|| "root".into()),
                }),
            ),
        };

        snapshot
            .items
            .insert(format!("{prefix}|{}", unit.name), value);
    }
}

fn add_cron(snapshot: &mut Snapshot, reading: &PersistenceReading<'_>) {
    for job in reading.cron {
        snapshot.items.insert(
            format!("cron|{}|{}|{}", job.source, job.user, job.command),
            json!({
                "source": job.source,
                "user": job.user,
                "schedule": job.schedule,
                "command": job.command,
                "command_redacted": job.command_redacted,
            }),
        );
    }
}

fn add_modules(snapshot: &mut Snapshot, reading: &PersistenceReading<'_>) {
    let Some(modules) = reading.modules else {
        snapshot.items.insert(
            MODULES_UNREADABLE.to_string(),
            json!({
                "readable": false,
                "reason": "/proc/modules could not be read: loaded kernel modules are not being watched",
            }),
        );
        return;
    };

    for module in modules {
        snapshot.items.insert(
            format!("module|{}", module.name),
            json!({
                "name": module.name,
                "size": module.size,
                "dependencies": module.dependencies,
                "state": module.state,
            }),
        );
    }
}

fn add_scripts(snapshot: &mut Snapshot, reading: &PersistenceReading<'_>) {
    for script in reading.scripts {
        snapshot.items.insert(
            format!("script|{}", script.path),
            json!({
                "path": script.path,
                "family": script.family.as_str(),
                "present": script.present,
                "readable": script.readable,
                "sha256": script.digest,
                "size": script.size,
                "mode": script.mode,
                "uid": script.uid,
                "gid": script.gid,
            }),
        );
    }
}

fn add_preload(snapshot: &mut Snapshot, reading: &PersistenceReading<'_>) {
    let preload = reading.preload;
    snapshot.items.insert(
        format!("preload|{}", preload.path),
        json!({
            "path": preload.path,
            "present": preload.present,
            "readable": preload.readable,
            "entries": preload.entries,
            "sha256": preload.digest,
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::super::crontab::{CronFormat, parse_crontab};
    use super::super::unit::parse_unit;
    use super::*;

    fn preload_absent() -> PreloadFile {
        PreloadFile {
            path: "/etc/ld.so.preload".into(),
            present: false,
            readable: true,
            entries: Vec::new(),
            digest: None,
        }
    }

    fn reading<'a>(
        units: &'a [UnitFile],
        cron: &'a [CronEntry],
        preload: &'a PreloadFile,
    ) -> PersistenceReading<'a> {
        PersistenceReading {
            units,
            cron,
            modules: Some(&[]),
            scripts: &[],
            preload,
        }
    }

    #[test]
    fn a_timer_and_a_service_are_different_kinds_of_key() {
        let units = vec![
            UnitFile {
                name: "nginx.service".into(),
                path: "/lib/systemd/system/nginx.service".into(),
                readable: true,
                facts: parse_unit("[Service]\nExecStart=/usr/sbin/nginx\n"),
            },
            UnitFile {
                name: "certbot.timer".into(),
                path: "/lib/systemd/system/certbot.timer".into(),
                readable: true,
                facts: parse_unit("[Timer]\nOnCalendar=daily\n"),
            },
        ];

        let preload = preload_absent();
        let snapshot =
            persistence_snapshot("2026-09-09T12:00:00.000Z", &reading(&units, &[], &preload));

        assert_eq!(snapshot.source, "persistence");
        assert_eq!(snapshot.items["unit|nginx.service"]["type"], "service");
        assert_eq!(snapshot.items["unit|nginx.service"]["run_as"], "root");
        assert_eq!(
            snapshot.items["timer|certbot.timer"]["activates"], "certbot.service",
            "a timer with no Unit= starts the service of the same name"
        );
        assert!(
            !snapshot.items.contains_key("unit|certbot.timer"),
            "a timer must not also be a unit, or one file is two findings"
        );
    }

    #[test]
    fn a_cron_key_is_the_file_the_user_and_the_command() {
        let jobs = parse_crontab(
            "@reboot /tmp/.x/implant\n",
            "/var/spool/cron/crontabs/www-data",
            CronFormat::ForOneUser,
            "www-data",
        );

        let preload = preload_absent();
        let snapshot =
            persistence_snapshot("2026-09-09T12:00:00.000Z", &reading(&[], &jobs, &preload));

        assert!(
            snapshot
                .items
                .contains_key("cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant"),
            "{:?}",
            snapshot.items.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_kernel_whose_module_list_we_could_not_read_says_so_instead_of_looking_empty() {
        let snapshot = persistence_snapshot(
            "2026-09-09T12:00:00.000Z",
            &PersistenceReading {
                units: &[],
                cron: &[],
                modules: None,
                scripts: &[],
                preload: &preload_absent(),
            },
        );

        assert_eq!(snapshot.items[MODULES_UNREADABLE]["readable"], false);
        assert!(
            !snapshot.items.keys().any(|key| key.starts_with("module|")),
            "the marker must not look like a module"
        );
    }

    #[test]
    fn the_preload_file_is_an_item_even_when_the_host_does_not_have_one() {
        let preload = preload_absent();
        let snapshot =
            persistence_snapshot("2026-09-09T12:00:00.000Z", &reading(&[], &[], &preload));

        assert_eq!(
            snapshot.items["preload|/etc/ld.so.preload"]["present"],
            false
        );
    }
}
