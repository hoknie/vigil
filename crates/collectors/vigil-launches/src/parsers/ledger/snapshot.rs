use std::collections::BTreeMap;

use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::origin::{AUDIT_LOG, AUDIT_PLUGIN, Origin};
use super::reading::LaunchReading;
use crate::helpers::moment;
use crate::parsers::audit::Execution;
use vigil_collect::redact;

pub const SOURCE: &str = "launches";

pub const UNNAMED: &str = "launches|unnamed";

pub const CAPPED: &str = "launches|capped";

pub const SOURCE_ROW: &str = "launches|source";

pub const DROPPING: &str = "launches|dropping";

const RUN: &str = "run|";

pub const LIMIT: usize = 20_000;

pub const RECENT_RUNS: usize = 8;

pub const RECENT: &str = "recent_runs";

pub const RAN_AT: &str = "id";

const ARGUMENTS_KEPT: usize = 120;

pub(super) const AUID_UNSET: u32 = u32::MAX;

pub fn launches_snapshot(
    taken_at: &str,
    known: &BTreeMap<String, Value>,
    reading: &LaunchReading<'_>,
) -> Snapshot {
    let origin = match reading.from_plugin {
        true => &AUDIT_PLUGIN,
        false => &AUDIT_LOG,
    };
    launches_snapshot_from(taken_at, known, reading, origin)
}

pub fn launches_snapshot_from(
    taken_at: &str,
    known: &BTreeMap<String, Value>,
    reading: &LaunchReading<'_>,
    origin: &Origin,
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

        if let Some(known) = snapshot.items.get_mut(&key) {
            ran_again(known, execution, reading.keep_arguments);
            continue;
        }
        if snapshot.items.len() >= LIMIT {
            capped = true;
            continue;
        }

        let (arguments, arguments_redacted) = recorded(execution, reading.keep_arguments);

        let seen = (reading.on_disk)(executable);
        snapshot.items.insert(
            key,
            json!({
                "user": login,
                "auid": auid,
                "exe": executable,
                "exe_lossy": execution.executable_lossy,
                "exe_present": seen.on_disk(),
                "exe_shown": seen.shown(),
                "writable_path": origin.writable(executable),
                "first_seen": taken_at,
                "runs": 1,
                "last_audit_id": execution.id,
                "audit_id": execution.id,
                RECENT: [ran(&execution.id, &arguments, arguments_redacted)],
                "arguments": arguments,
                "arguments_redacted": arguments_redacted,
            }),
        );
    }

    snapshot.items.insert(
        SOURCE_ROW.to_string(),
        json!({
            "named": false,
            "from": origin.from,
            "reason": origin.reason,
        }),
    );

    match reading.dropped {
        true => {
            snapshot.items.insert(
                DROPPING.to_string(),
                json!({
                    "named": false,
                    "reason": origin.dropped,
                }),
            );
        }
        false => {
            snapshot.items.remove(DROPPING);
        }
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

fn recorded(execution: &Execution, keep_arguments: bool) -> (Value, bool) {
    match keep_arguments {
        true if !execution.arguments.is_empty() => {
            let clean = redact(&execution.arguments);
            (
                Value::String(clean.text),
                clean.redacted || execution.arguments_redacted,
            )
        }
        _ => (Value::Null, false),
    }
}

fn ran(id: &str, arguments: &Value, redacted: bool) -> Value {
    json!({
        RAN_AT: id,
        "arguments": shortened(arguments),
        "arguments_redacted": redacted,
    })
}

fn shortened(arguments: &Value) -> Value {
    let Some(said) = arguments.as_str() else {
        return Value::Null;
    };
    match said.chars().count() > ARGUMENTS_KEPT {
        true => Value::String(
            said.chars()
                .take(ARGUMENTS_KEPT)
                .chain(std::iter::once('\u{2026}'))
                .collect(),
        ),
        false => Value::String(said.to_string()),
    }
}

fn ran_again(known: &mut Value, execution: &Execution, keep_arguments: bool) {
    let Some(fields) = known.as_object_mut() else {
        return;
    };
    let id = execution.id.as_str();
    let counted_id = fields
        .get("last_audit_id")
        .or_else(|| fields.get("audit_id"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let counted = counted_id.as_deref().and_then(moment);
    if let (Some(counted), Some(now)) = (counted, moment(id))
        && now <= counted
    {
        return;
    }

    let before = fields.get("runs").and_then(Value::as_u64).unwrap_or(1);
    fields.insert("runs".to_string(), json!(before.saturating_add(1)));
    fields.insert("last_audit_id".to_string(), json!(id));

    let mut recent = match fields.remove(RECENT) {
        Some(Value::Array(ids)) => ids,
        _ => counted_id.map(Value::String).into_iter().collect(),
    };
    let (arguments, redacted) = recorded(execution, keep_arguments);
    recent.push(ran(id, &arguments, redacted));
    let over = recent.len().saturating_sub(RECENT_RUNS);
    recent.drain(..over);
    fields.insert(RECENT.to_string(), Value::Array(recent));
}

pub fn any_launch_was_read(items: &BTreeMap<String, Value>) -> bool {
    items.keys().any(|key| key.starts_with(RUN))
}
