use std::collections::BTreeMap;

use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::reading::LaunchReading;
use crate::parsers::processes::redact;

pub const SOURCE: &str = "launches";

pub const UNNAMED: &str = "launches|unnamed";

pub const CAPPED: &str = "launches|capped";

pub const SOURCE_ROW: &str = "launches|source";

pub const DROPPING: &str = "launches|dropping";

pub const LIMIT: usize = 20_000;

pub(super) const AUID_UNSET: u32 = u32::MAX;

const WRITABLE_PATHS: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"];

pub fn launches_snapshot(
    taken_at: &str,
    known: &BTreeMap<String, Value>,
    reading: &LaunchReading<'_>,
) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());
    snapshot.items = known.clone();

    let was_capped = snapshot.items.contains_key(CAPPED);
    let mut capped = was_capped;

    for execution in reading.executions {
        let Some(auid) = execution.auid.filter(|auid| *auid != AUID_UNSET) else {
            continue;
        };
        let Some(executable) = execution.executable.as_deref() else {
            continue;
        };

        let login = reading.logins.get(&auid).cloned();
        let name = login.clone().unwrap_or_else(|| auid.to_string());
        let key = format!("run|{name}|{executable}");

        if snapshot.items.contains_key(&key) {
            continue;
        }
        if snapshot.items.len() >= LIMIT {
            capped = true;
            continue;
        }

        let (arguments, arguments_redacted) = match reading.keep_arguments {
            true if !execution.arguments.is_empty() => {
                let clean = redact(&execution.arguments);
                (Value::String(clean.text), clean.redacted)
            }
            _ => (Value::Null, false),
        };

        snapshot.items.insert(
            key,
            json!({
                "user": login,
                "auid": auid,
                "exe": executable,
                "exe_lossy": execution.executable_lossy,
                "exe_present": (reading.on_disk)(executable),
                "writable_path": is_writable_path(executable),
                "first_seen": taken_at,
                "audit_id": execution.id,
                "arguments": arguments,
                "arguments_redacted": arguments_redacted,
            }),
        );
    }

    snapshot.items.insert(
        SOURCE_ROW.to_string(),
        json!({
            "named": false,
            "from": match reading.from_plugin {
                true => "audit plugin",
                false => "audit log",
            },
            "reason": match reading.from_plugin {
                true => "launches arrive through the plugin auditd starts",
                false => "no plugin is delivering: launches are read from the log file, one reading late, and a rotation between two readings takes what it held",
            },
        }),
    );

    if reading.dropped {
        snapshot.items.entry(DROPPING.to_string()).or_insert_with(|| {
            json!({
                "named": false,
                "reason": "the audit plugin dropped the oldest events to stay under its spool size; launches from that window were never read",
            })
        });
    }

    if reading.any_unnamed {
        snapshot.items.insert(
            UNNAMED.to_string(),
            json!({
                "named": false,
                "reason": "some launches carried no path this build could resolve: those programs are not in this reading",
            }),
        );
    }

    if capped {
        snapshot.items.insert(
            CAPPED.to_string(),
            json!({
                "named": false,
                "reason": format!("the limit of {LIMIT} pairs of person and program is reached: launches are no longer being recorded"),
            }),
        );
    }

    snapshot
}

fn is_writable_path(executable: &str) -> bool {
    WRITABLE_PATHS
        .iter()
        .any(|writable| executable.starts_with(writable))
}
