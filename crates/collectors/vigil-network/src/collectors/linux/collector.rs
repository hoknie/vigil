use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;

use vigil_model::{Rfc3339, Snapshot};

use vigil_collect::{CollectError, Collector, Health, parse_passwd};

use super::socket_owner::resolve_owners;
use crate::parsers::{
    Protocol, SocketRow, SocketsReading, UnixSocketRow, listening_snapshot, parse_net_table,
    parse_unix_table,
};

const TABLES: &[(&str, Protocol)] = &[
    ("/proc/net/tcp", Protocol::Tcp),
    ("/proc/net/tcp6", Protocol::Tcp6),
    ("/proc/net/udp", Protocol::Udp),
    ("/proc/net/udp6", Protocol::Udp6),
];

const UNIX_TABLE: &str = "/proc/net/unix";

pub struct PortsCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl PortsCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        PortsCollector { now: Box::new(now) }
    }
}

impl Collector for PortsCollector {
    fn name(&self) -> &'static str {
        "ports"
    }

    fn available(&self) -> Health {
        match fs::read_to_string("/proc/net/tcp") {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                return Health::Unavailable("/proc/net/tcp cannot be read".into());
            }
            Err(_) => {
                return Health::Unavailable(
                    "/proc/net/tcp is not present: this collector needs Linux procfs".into(),
                );
            }
        }

        let mut looked_at = 0usize;
        let mut refused = 0usize;
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let Some(pid) = entry
                    .file_name()
                    .to_str()
                    .and_then(|n| n.parse::<u32>().ok())
                else {
                    continue;
                };
                if pid == std::process::id() {
                    continue;
                }
                match probe_descriptors(pid) {
                    Some(true) => looked_at += 1,
                    Some(false) => refused += 1,
                    None => {}
                }
                if looked_at > 0 || refused >= 8 {
                    break;
                }
            }
        }

        if looked_at == 0 && refused > 0 {
            return Health::Degraded(format!(
                "socket owners not resolved: cannot read /proc/<pid>/fd for {refused} process(es) (needs CAP_SYS_PTRACE and CAP_DAC_READ_SEARCH, or root without a restricted capability set)"
            ));
        }

        Health::Ok
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let mut rows: Vec<SocketRow> = Vec::new();
        let mut unparsed = 0usize;
        let mut read_any = false;

        for (path, protocol) in TABLES {
            match fs::read_to_string(path) {
                Ok(text) => {
                    read_any = true;
                    let table = parse_net_table(&text, *protocol);
                    unparsed += table.unparsed;
                    rows.extend(table.listening);
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                    return Err(CollectError::Denied((*path).to_string()));
                }
                Err(error) => {
                    return Err(CollectError::Unreadable(format!("{path}: {error}")));
                }
            }
        }

        if !read_any {
            return Err(CollectError::Absent("/proc/net/tcp".into()));
        }

        let mut unix_rows: Vec<UnixSocketRow> = Vec::new();
        let mut unnamed_unix = 0usize;
        match fs::read_to_string(UNIX_TABLE) {
            Ok(text) => {
                let table = parse_unix_table(&text);
                unparsed += table.unparsed;
                unnamed_unix = table.unnamed;
                unix_rows = table.listening;
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                return Err(CollectError::Denied(UNIX_TABLE.to_string()));
            }
            Err(error) => {
                return Err(CollectError::Unreadable(format!("{UNIX_TABLE}: {error}")));
            }
        }

        if unparsed > 0 && rows.is_empty() && unix_rows.is_empty() {
            return Err(CollectError::Unreadable(format!(
                "{unparsed} row(s) in /proc/net/* in a format this build does not know"
            )));
        }

        let inodes: BTreeSet<u64> = rows
            .iter()
            .map(|row| row.inode)
            .chain(unix_rows.iter().map(|row| row.inode))
            .collect();
        let owners = resolve_owners(&inodes).by_inode;

        let users = fs::read_to_string("/etc/passwd")
            .map(|text| parse_passwd(&text))
            .unwrap_or_default();

        Ok(listening_snapshot(
            &(self.now)(),
            &SocketsReading {
                network: &rows,
                unix: &unix_rows,
                unnamed_unix,
                owners: &owners,
                users: &users,
            },
        ))
    }
}

fn probe_descriptors(pid: u32) -> Option<bool> {
    let entries = match fs::read_dir(format!("/proc/{pid}/fd")) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::PermissionDenied => return Some(false),
        Err(_) => return None,
    };

    for entry in entries.flatten() {
        match fs::read_link(entry.path()) {
            Ok(_) => return Some(true),
            Err(error) if error.kind() == ErrorKind::PermissionDenied => return Some(false),
            Err(_) => continue,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_this_host_and_names_itself_in_the_snapshot() {
        let collector = PortsCollector::new(|| "2026-09-08T12:00:00.000Z".to_string());

        let snapshot = collector.collect().expect("procfs is readable on Linux");

        assert_eq!(snapshot.source, "ports");
        assert_eq!(snapshot.taken_at, "2026-09-08T12:00:00.000Z");
        for (key, item) in &snapshot.items {
            assert!(item.get("owner_resolved").is_some(), "{key} lost its flag");
            match key.split_once('|') {
                Some(("tcp" | "tcp6" | "udp" | "udp6", rest)) => {
                    assert!(rest.contains(':'), "key shape: {key}")
                }
                Some(("unix", rest)) => assert!(
                    rest.starts_with('/') || rest.starts_with('@') || rest == "unnamed",
                    "key shape: {key}"
                ),
                _ => panic!("key shape: {key}"),
            }
        }
    }

    #[test]
    fn a_unix_socket_reaches_the_snapshot_under_the_name_it_is_reachable_by() {
        let collector = PortsCollector::new(|| "2026-09-08T12:00:00.000Z".to_string());
        let snapshot = collector.collect().expect("procfs is readable on Linux");

        for (key, item) in &snapshot.items {
            if key == "unix|unnamed" || !key.starts_with("unix|") {
                continue;
            }
            assert_eq!(item["protocol"], "unix");
            assert_eq!(
                item["path"],
                serde_json::Value::from(&key["unix|".len()..]),
                "the key and the item must name the same socket"
            );
            assert!(item.get("port").is_none(), "{key} is not a port");
        }
    }
}
