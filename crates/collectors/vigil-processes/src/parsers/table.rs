use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;
use vigil_model::Snapshot;

use crate::types::{LINUX, Platform};

pub const SOURCE: &str = "processes";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRow {
    pub pid: u32,
    pub parent: u32,
    pub uid: u32,
    pub executable: Option<String>,
    pub executable_deleted: bool,
    pub command_line: Option<String>,
    pub command_line_redacted: bool,
}

pub struct ProcessesReading<'a> {
    pub processes: &'a [ProcessRow],
    pub users: &'a BTreeMap<u32, String>,
    pub any_unresolved: bool,
}

pub const UNRESOLVED: &str = "processes|unresolved";

pub fn processes_snapshot(taken_at: &str, reading: &ProcessesReading<'_>) -> Snapshot {
    processes_snapshot_on(taken_at, reading, &LINUX)
}

pub fn processes_snapshot_on(
    taken_at: &str,
    reading: &ProcessesReading<'_>,
    platform: &Platform,
) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    let by_pid: BTreeMap<u32, &ProcessRow> = reading
        .processes
        .iter()
        .map(|process| (process.pid, process))
        .collect();

    let mut programs: BTreeMap<(String, u32), Program> = BTreeMap::new();
    for process in reading.processes {
        let Some(executable) = &process.executable else {
            continue;
        };
        let program = programs
            .entry((executable.clone(), process.uid))
            .or_default();

        program.executable_deleted |= process.executable_deleted;
        program.observe_command_line(process);

        if let Some(parent) = by_pid.get(&process.parent)
            && let Some(parent_executable) = &parent.executable
            && parent_executable != executable
        {
            program.parents.insert(parent_executable.clone());
        }
    }

    for ((executable, uid), program) in programs {
        let user = reading.users.get(&uid).cloned();
        let name = user.clone().unwrap_or_else(|| uid.to_string());

        snapshot.items.insert(
            format!("exec|{executable}|{name}"),
            json!({
                "exe": executable,
                "exe_deleted": program.executable_deleted,
                "uid": uid,
                "user": user,
                "parents": program.parents.iter().take(PARENT_LIMIT).collect::<Vec<_>>(),
                "parents_truncated": program.parents.len() > PARENT_LIMIT,
                "cmdline": program.command_line,
                "cmdline_varies": program.command_line_varies,
                "cmdline_redacted": program.command_line_redacted,
                "writable_path": platform.writable(&executable),
                "exe_resolved": true,
            }),
        );
    }

    if reading.any_unresolved {
        snapshot.items.insert(
            UNRESOLVED.to_string(),
            json!({
                "exe_resolved": false,
                "reason": platform.unresolved_reason,
            }),
        );
    }

    snapshot
}

const PARENT_LIMIT: usize = 16;

#[derive(Default)]
struct Program {
    executable_deleted: bool,
    parents: BTreeSet<String>,
    command_line: Option<String>,
    command_line_varies: bool,
    command_line_redacted: bool,
}

impl Program {
    fn observe_command_line(&mut self, process: &ProcessRow) {
        self.command_line_redacted |= process.command_line_redacted;

        match (&self.command_line, &process.command_line) {
            (None, Some(line)) if !self.command_line_varies => {
                self.command_line = Some(line.clone());
            }
            (Some(known), Some(line)) if known != line => {
                self.command_line = None;
                self.command_line_varies = true;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(pid: u32, parent: u32, uid: u32, executable: &str, command: &str) -> ProcessRow {
        ProcessRow {
            pid,
            parent,
            uid,
            executable: Some(executable.to_string()),
            executable_deleted: false,
            command_line: Some(command.to_string()),
            command_line_redacted: false,
        }
    }

    fn users() -> BTreeMap<u32, String> {
        BTreeMap::from([(0, "root".to_string()), (33, "www-data".to_string())])
    }

    fn snapshot_of(processes: &[ProcessRow]) -> Snapshot {
        let users = users();
        processes_snapshot(
            "2026-09-09T12:00:00.000Z",
            &ProcessesReading {
                processes,
                users: &users,
                any_unresolved: false,
            },
        )
    }

    #[test]
    fn four_workers_of_one_service_are_one_row_not_four() {
        let snapshot = snapshot_of(&[
            process(812, 1, 0, "/usr/sbin/nginx", "nginx: master"),
            process(813, 812, 33, "/usr/sbin/nginx", "nginx: worker"),
            process(814, 812, 33, "/usr/sbin/nginx", "nginx: worker"),
            process(815, 812, 33, "/usr/sbin/nginx", "nginx: worker"),
        ]);

        assert_eq!(snapshot.items.len(), 2, "{:?}", snapshot.items.keys());
        assert!(snapshot.items.contains_key("exec|/usr/sbin/nginx|root"));
        assert!(snapshot.items.contains_key("exec|/usr/sbin/nginx|www-data"));
    }

    #[test]
    fn the_same_host_read_twice_with_different_pids_gives_the_same_snapshot() {
        let first = snapshot_of(&[
            process(812, 1, 0, "/usr/sbin/nginx", "nginx: master"),
            process(900, 812, 33, "/usr/sbin/nginx", "nginx: worker"),
        ]);
        let later = snapshot_of(&[
            process(41200, 1, 0, "/usr/sbin/nginx", "nginx: master"),
            process(41300, 41200, 33, "/usr/sbin/nginx", "nginx: worker"),
        ]);

        assert_eq!(first.items, later.items);
    }

    #[test]
    fn the_parent_is_recorded_by_what_it_runs_and_not_by_its_number() {
        let snapshot = snapshot_of(&[
            process(812, 1, 33, "/usr/sbin/php-fpm", "php-fpm: pool www"),
            process(9001, 812, 33, "/bin/sh", "sh -c id"),
        ]);

        assert_eq!(
            snapshot.items["exec|/bin/sh|www-data"]["parents"],
            json!(["/usr/sbin/php-fpm"])
        );
    }

    #[test]
    fn a_command_line_that_differs_between_instances_is_reported_as_differing() {
        let snapshot = snapshot_of(&[
            process(9001, 1, 0, "/bin/sh", "sh -c /usr/bin/one"),
            process(9002, 1, 0, "/bin/sh", "sh -c /usr/bin/two"),
        ]);

        let item = &snapshot.items["exec|/bin/sh|root"];
        assert_eq!(item["cmdline"], json!(null));
        assert_eq!(item["cmdline_varies"], json!(true));
    }

    #[test]
    fn one_command_line_shared_by_every_instance_is_kept() {
        let snapshot = snapshot_of(&[
            process(812, 1, 33, "/usr/sbin/nginx", "nginx: worker"),
            process(813, 1, 33, "/usr/sbin/nginx", "nginx: worker"),
        ]);

        let item = &snapshot.items["exec|/usr/sbin/nginx|www-data"];
        assert_eq!(item["cmdline"], "nginx: worker");
        assert_eq!(item["cmdline_varies"], json!(false));
    }

    #[test]
    fn an_account_with_no_name_is_still_a_row_under_its_number() {
        let users = BTreeMap::new();
        let snapshot = processes_snapshot(
            "2026-09-09T12:00:00.000Z",
            &ProcessesReading {
                processes: &[process(9001, 1, 4242, "/opt/app/server", "server")],
                users: &users,
                any_unresolved: false,
            },
        );

        assert!(snapshot.items.contains_key("exec|/opt/app/server|4242"));
        assert_eq!(
            snapshot.items["exec|/opt/app/server|4242"]["user"],
            json!(null)
        );
    }

    #[test]
    fn processes_the_agent_could_not_look_at_are_a_row_rather_than_a_silence() {
        let users = users();
        let snapshot = processes_snapshot(
            "2026-09-09T12:00:00.000Z",
            &ProcessesReading {
                processes: &[],
                users: &users,
                any_unresolved: true,
            },
        );

        assert_eq!(snapshot.items[UNRESOLVED]["exe_resolved"], json!(false));
        assert!(
            !snapshot.items.keys().any(|key| key.starts_with("exec|")),
            "the marker must not look like a program"
        );
    }

    #[test]
    fn a_reading_of_macos_marks_its_own_writable_directories_and_says_its_own_reason() {
        let users = users();
        let snapshot = processes_snapshot_on(
            "2026-09-19T12:00:00.000Z",
            &ProcessesReading {
                processes: &[process(9001, 1, 0, "/private/tmp/.x/nc", "nc -l 4444")],
                users: &users,
                any_unresolved: true,
            },
            &crate::types::MACOS,
        );

        assert_eq!(
            snapshot.items["exec|/private/tmp/.x/nc|root"]["writable_path"],
            json!(true)
        );
        assert!(
            snapshot.items[UNRESOLVED]["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("run as root"))
        );
    }

    #[test]
    fn a_linux_reading_is_built_as_it_was_before_macos_was_added() {
        let users = users();
        let snapshot = processes_snapshot(
            "2026-09-09T12:00:00.000Z",
            &ProcessesReading {
                processes: &[
                    process(9001, 1, 0, "/private/tmp/nc", "nc"),
                    process(9002, 1, 0, "/dev/shm/x", "x"),
                ],
                users: &users,
                any_unresolved: true,
            },
        );

        assert_eq!(
            snapshot.items["exec|/private/tmp/nc|root"]["writable_path"],
            json!(false)
        );
        assert_eq!(
            snapshot.items["exec|/dev/shm/x|root"]["writable_path"],
            json!(true)
        );
        assert_eq!(
            snapshot.items[UNRESOLVED]["reason"],
            "some executables could not be read: those programs are not in this reading (needs CAP_SYS_PTRACE, or root without a restricted capability set)"
        );
    }
}
