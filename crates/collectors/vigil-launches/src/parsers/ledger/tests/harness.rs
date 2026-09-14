use std::collections::BTreeMap;

use serde_json::Value;
use vigil_model::Snapshot;

use super::super::reading::LaunchReading;
use super::super::snapshot::launches_snapshot;
use crate::parsers::audit::Execution;
use vigil_collect::Presence;

pub(super) fn logins() -> BTreeMap<u32, String> {
    BTreeMap::from([(1000, "alice".to_string()), (0, "root".to_string())])
}

pub(super) fn launch(auid: u32, executable: &str, arguments: &[&str]) -> Execution {
    Execution {
        id: "1757419203.412:3421".into(),
        auid: Some(auid),
        executable: Some(executable.to_string()),
        executable_lossy: false,
        arguments: arguments.iter().map(|word| (*word).to_string()).collect(),
        arguments_lossy: false,
    }
}

pub(super) fn later(auid: u32, executable: &str, arguments: &[&str], serial: u64) -> Execution {
    Execution {
        id: format!("1757419300.000:{}", 4000 + serial),
        ..launch(auid, executable, arguments)
    }
}

pub(super) fn everything_is_there(_path: &str) -> Presence {
    Presence::OnDisk
}

pub(super) fn snapshot_of(known: &BTreeMap<String, Value>, executions: &[Execution]) -> Snapshot {
    let logins = logins();
    launches_snapshot(
        "2026-09-09T12:00:00.000Z",
        known,
        &LaunchReading {
            executions,
            logins: &logins,
            any_unnamed: false,
            keep_arguments: false,
            on_disk: &everything_is_there,
            from_plugin: true,
            dropped: false,
        },
    )
}

pub(super) fn fresh(executions: &[Execution]) -> Snapshot {
    snapshot_of(&BTreeMap::new(), executions)
}

pub(super) fn people_and_programs(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .items
        .keys()
        .filter(|key| key.starts_with("run|"))
        .map(String::as_str)
        .collect()
}
