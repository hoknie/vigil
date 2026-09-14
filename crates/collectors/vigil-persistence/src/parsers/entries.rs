use serde_json::json;
use vigil_model::Snapshot;

use super::crontab::CronEntry;
use super::modules::KernelModule;
use super::pulled::pulled_in_by;
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
    pub present: Option<bool>,
    pub shown: bool,
    pub readable: Option<bool>,
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
    let pulled = pulled_in_by(reading.units);
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
            kind => {
                let mut value = json!({
                    "name": unit.name,
                    "type": kind,
                    "path": unit.path,
                    "readable": unit.readable,
                    "description": unit.facts.description,
                    "commands": unit.facts.commands,
                    "commands_redacted": unit.facts.commands_redacted,
                    "run_as": unit.facts.run_as.clone().unwrap_or_else(|| "root".into()),
                });
                for (setting, named) in unit.facts.links.named() {
                    if !named.is_empty() {
                        value[setting] = json!(named);
                    }
                }
                let by: Vec<&str> = pulled
                    .get(unit.name.as_str())
                    .into_iter()
                    .flatten()
                    .copied()
                    .collect();
                value["pulled_in_by"] = json!(by);
                ("unit", value)
            }
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
                "shown": script.shown,
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
