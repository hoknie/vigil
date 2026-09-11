use std::collections::BTreeMap;

use vigil_model::Snapshot;

use crate::parsers::{ProcessRow, ProcessesReading, processes_snapshot};

fn process(
    pid: u32,
    parent: u32,
    uid: u32,
    executable: &str,
    command_line: Option<&str>,
) -> ProcessRow {
    ProcessRow {
        pid,
        parent,
        uid,
        executable: Some(executable.to_string()),
        executable_deleted: executable.starts_with("/tmp/"),
        command_line: command_line.map(str::to_string),
        command_line_redacted: command_line.is_some_and(|line| line.contains("[redacted]")),
    }
}

pub fn processes() -> Snapshot {
    let processes = [
        process(1, 0, 0, "/lib/systemd/systemd", Some("/sbin/init")),
        process(812, 1, 0, "/usr/sbin/nginx", Some("nginx: master process")),
        process(813, 812, 33, "/usr/sbin/nginx", Some("nginx: worker one")),
        process(814, 812, 33, "/usr/sbin/nginx", Some("nginx: worker two")),
        process(
            9001,
            813,
            33,
            "/tmp/.x/nc",
            Some("nc -l -p 4444 -e [redacted]"),
        ),
        process(9100, 1, 4242, "/opt/app/server", None),
    ];
    let users = BTreeMap::from([(0, "root".to_string()), (33, "www-data".to_string())]);

    processes_snapshot(
        "2026-09-09T09:00:00.000Z",
        &ProcessesReading {
            processes: &processes,
            users: &users,
            any_unresolved: true,
        },
    )
}
