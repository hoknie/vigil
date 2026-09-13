use std::fs;
use std::io::ErrorKind;

use vigil_model::{Rfc3339, Snapshot};

use crate::parsers::{ProcessRow, ProcessesReading, parse_status, processes_snapshot};
use vigil_collect::{CollectError, Collector, Health};
use vigil_collect::{parse_passwd, redact};

pub struct ProcessesCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl ProcessesCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ProcessesCollector { now: Box::new(now) }
    }
}

impl Collector for ProcessesCollector {
    fn name(&self) -> &'static str {
        "processes"
    }

    fn available(&self) -> Health {
        if fs::read_dir("/proc").is_err() {
            return Health::Unavailable(
                "/proc cannot be listed: this collector needs Linux procfs".into(),
            );
        }

        let mut resolved = 0usize;
        let mut refused = 0usize;
        for pid in pids() {
            if pid == std::process::id() {
                continue;
            }
            match fs::read_link(format!("/proc/{pid}/exe")) {
                Ok(_) => resolved += 1,
                Err(error) if error.kind() == ErrorKind::PermissionDenied => refused += 1,
                Err(_) => {}
            }
            if resolved > 0 || refused >= 8 {
                break;
            }
        }

        if resolved == 0 && refused > 0 {
            return Health::Degraded(format!(
                "executable not read for {refused} process(es): permission denied (needs CAP_SYS_PTRACE and CAP_DAC_READ_SEARCH, or root without a restricted capability set)"
            ));
        }

        Health::Ok
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let Ok(entries) = fs::read_dir("/proc") else {
            return Err(CollectError::Absent("/proc".into()));
        };

        let mut processes: Vec<ProcessRow> = Vec::new();
        let mut unresolved = 0usize;
        let mut seen = 0usize;

        for entry in entries.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            seen += 1;

            let Ok(text) = fs::read_to_string(format!("/proc/{pid}/status")) else {
                continue;
            };
            let Some(status) = parse_status(&text) else {
                continue;
            };

            let executable = match fs::read_link(format!("/proc/{pid}/exe")) {
                Ok(path) => Some(path.to_string_lossy().into_owned()),
                Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                    unresolved += 1;
                    None
                }
                Err(_) => continue,
            };

            let (executable, deleted) = match executable {
                Some(path) => match path.strip_suffix(" (deleted)") {
                    Some(real) => (Some(real.to_string()), true),
                    None => (Some(path), false),
                },
                None => (None, false),
            };

            let (command_line, command_line_redacted) = read_command_line(pid);

            processes.push(ProcessRow {
                pid,
                parent: status.parent,
                uid: status.uid,
                executable,
                executable_deleted: deleted,
                command_line,
                command_line_redacted,
            });
        }

        if seen == 0 {
            return Err(CollectError::Unreadable(
                "/proc holds no process this build can read".into(),
            ));
        }

        let users = fs::read_to_string("/etc/passwd")
            .map(|text| parse_passwd(&text))
            .unwrap_or_default();

        Ok(processes_snapshot(
            &(self.now)(),
            &ProcessesReading {
                processes: &processes,
                users: &users,
                any_unresolved: unresolved > 0,
            },
        ))
    }
}

fn read_command_line(pid: u32) -> (Option<String>, bool) {
    let Ok(raw) = fs::read(format!("/proc/{pid}/cmdline")) else {
        return (None, false);
    };

    let arguments: Vec<String> = raw
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).to_string())
        .collect();
    if arguments.is_empty() {
        return (None, false);
    }

    let clean = redact(&arguments);
    (Some(clean.text), clean.redacted)
}

fn pids() -> Vec<u32> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_this_host_and_names_itself_in_the_snapshot() {
        let collector = ProcessesCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

        let snapshot = collector.collect().expect("procfs is readable on Linux");

        assert_eq!(snapshot.source, "processes");
        assert!(
            snapshot.items.keys().any(|key| key.starts_with("exec|")),
            "this process is running, so at least one program is"
        );
        for (key, item) in &snapshot.items {
            assert!(item.get("exe_resolved").is_some(), "{key} lost its flag");
            if key.starts_with("exec|") {
                assert_eq!(
                    key.matches('|').count(),
                    2,
                    "the key is exec|<executable>|<account>: {key}"
                );
            }
        }
    }

    #[test]
    fn two_readings_a_moment_apart_describe_the_same_host() {
        let collector = ProcessesCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

        let first = collector.collect().expect("readable");
        let second = collector.collect().expect("readable");

        let changed = first
            .items
            .keys()
            .filter(|key| first.items.get(*key) != second.items.get(*key))
            .count();
        assert!(
            changed <= 2,
            "two readings a moment apart differed in {changed} rows: {:?}",
            first
                .items
                .keys()
                .filter(|key| first.items.get(*key) != second.items.get(*key))
                .collect::<Vec<_>>()
        );
    }
}
