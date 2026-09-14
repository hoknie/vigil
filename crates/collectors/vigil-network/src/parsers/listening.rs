use std::collections::BTreeMap;

use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::proc_net::SocketRow;
use super::proc_net_unix::UnixSocketRow;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessOwner {
    pub pid: Option<u32>,
    pub executable: Option<String>,
    pub executable_deleted: bool,
    pub command_line: Option<String>,
    pub command_line_redacted: bool,
    pub uid: Option<u32>,
}

pub const SOURCE: &str = "ports";

const UNNAMED_KEY: &str = "unix|unnamed";

pub struct SocketsReading<'a> {
    pub network: &'a [SocketRow],
    pub unix: &'a [UnixSocketRow],
    pub unnamed_unix: usize,
    pub owners: &'a BTreeMap<u64, ProcessOwner>,
    pub users: &'a BTreeMap<u32, String>,
}

pub fn listening_snapshot(taken_at: &str, reading: &SocketsReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    add_network(&mut snapshot, reading);
    add_unix(&mut snapshot, reading);

    snapshot
}

fn add_network(snapshot: &mut Snapshot, reading: &SocketsReading<'_>) {
    for row in reading.network {
        let key = format!("{}|{}:{}", row.protocol.as_str(), row.address, row.port);
        let owner = reading.owners.get(&row.inode);

        snapshot.items.insert(
            key,
            json!({
                "protocol": row.protocol.as_str(),
                "address": row.address,
                "port": row.port,
                "uid": row.uid,
                "user": reading.users.get(&row.uid),
                "process": describe(owner),
                "owner_resolved": owner.is_some(),
            }),
        );
    }
}

fn add_unix(snapshot: &mut Snapshot, reading: &SocketsReading<'_>) {
    for row in reading.unix {
        let owner = reading.owners.get(&row.inode);
        let uid = owner.and_then(|owner| owner.uid);

        snapshot.items.insert(
            format!("unix|{}", row.name),
            json!({
                "protocol": "unix",
                "type": row.kind.as_str(),
                "path": row.name,
                "abstract": row.is_abstract,
                "uid": uid,
                "user": uid.and_then(|uid| reading.users.get(&uid)),
                "process": describe(owner),
                "owner_resolved": owner.is_some(),
            }),
        );
    }

    if reading.unnamed_unix > 0 {
        snapshot.items.insert(
            UNNAMED_KEY.to_string(),
            json!({
                "protocol": "unix",
                "count": reading.unnamed_unix,
                "owner_resolved": false,
            }),
        );
    }
}

fn describe(owner: Option<&ProcessOwner>) -> Value {
    match owner {
        Some(owner) => json!({
            "pid": owner.pid,
            "exe": owner.executable,
            "exe_deleted": owner.executable_deleted,
            "cmdline": owner.command_line,
            "cmdline_redacted": owner.command_line_redacted,
        }),
        None => Value::Null,
    }
}
