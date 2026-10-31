use std::collections::BTreeMap;
use std::net::IpAddr;

use serde_json::{Value, json};

use super::listening::ProcessOwner;
use super::proc_net::{Connection, SocketRow};

pub const PAIR_CEILING: usize = 512;

pub const OUTBOUND: &str = "outbound";

pub const INBOUND: &str = "inbound";

pub const UNRESOLVED: &str = "unresolved";

pub struct PairsReading<'a> {
    pub listening: &'a [SocketRow],
    pub established: &'a [Connection],
    pub owners: &'a BTreeMap<u64, ProcessOwner>,
    pub users: &'a BTreeMap<u32, String>,
}

struct Held {
    pid: u32,
    row: Value,
}

pub fn connection_pairs(reading: &PairsReading<'_>) -> BTreeMap<String, Value> {
    let mut outbound: BTreeMap<String, Held> = BTreeMap::new();
    let mut inbound: BTreeMap<String, Held> = BTreeMap::new();

    for connection in reading.established {
        if on_loopback(&connection.remote_address) {
            continue;
        }
        let owner = reading.owners.get(&connection.inode);
        let program = owner.and_then(|owner| owner.executable.as_deref());
        let named = program.unwrap_or(UNRESOLVED);
        let protocol = connection.protocol.as_str();
        let pid = owner.and_then(|owner| owner.pid).unwrap_or(u32::MAX);
        let common = Common {
            program,
            deleted: owner.is_some_and(|owner| owner.executable_deleted),
            uid: connection.uid,
            user: reading.users.get(&connection.uid).map(String::as_str),
            resolved: owner.is_some(),
        };

        let (pairs, key, row) = match accepted_here(connection, reading.listening) {
            true => (
                &mut inbound,
                format!(
                    "{INBOUND}|{protocol}|{}|{}|{named}",
                    connection.local_port, connection.remote_address
                ),
                common.into_row(json!({
                    "protocol": protocol,
                    "local_port": connection.local_port,
                    "remote_address": connection.remote_address,
                })),
            ),
            false => (
                &mut outbound,
                format!(
                    "{OUTBOUND}|{protocol}|{}:{}|{named}",
                    connection.remote_address, connection.remote_port
                ),
                common.into_row(json!({
                    "protocol": protocol,
                    "remote_address": connection.remote_address,
                    "remote_port": connection.remote_port,
                })),
            ),
        };

        match pairs.get(&key) {
            Some(held) if held.pid <= pid => {}
            _ => {
                pairs.insert(key, Held { pid, row });
            }
        }
    }

    let mut rows = BTreeMap::new();
    for (direction, pairs) in [(OUTBOUND, outbound), (INBOUND, inbound)] {
        let beyond = pairs.len().saturating_sub(PAIR_CEILING);
        rows.extend(
            pairs
                .into_iter()
                .take(PAIR_CEILING)
                .map(|(key, held)| (key, held.row)),
        );
        if beyond > 0 {
            rows.insert(format!("{direction}|beyond"), json!({ "count": beyond }));
        }
    }
    rows
}

struct Common<'a> {
    program: Option<&'a str>,
    deleted: bool,
    uid: u32,
    user: Option<&'a str>,
    resolved: bool,
}

impl Common<'_> {
    fn into_row(self, mut row: Value) -> Value {
        row["program"] = json!(self.program);
        row["program_deleted"] = json!(self.deleted);
        row["uid"] = json!(self.uid);
        row["user"] = json!(self.user);
        row["owner_resolved"] = json!(self.resolved);
        row
    }
}

fn accepted_here(connection: &Connection, listening: &[SocketRow]) -> bool {
    listening.iter().any(|door| {
        door.protocol == connection.protocol
            && door.port == connection.local_port
            && (door.address == "0.0.0.0"
                || door.address == "::"
                || door.address == connection.local_address)
    })
}

fn on_loopback(address: &str) -> bool {
    match address.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => v4.is_loopback(),
        Ok(IpAddr::V6(v6)) => {
            v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::Protocol;

    fn door(port: u16) -> SocketRow {
        SocketRow {
            protocol: Protocol::Tcp,
            address: "0.0.0.0".into(),
            port,
            uid: 0,
            inode: 1,
        }
    }

    fn talking(inode: u64, local_port: u16, remote: &str, remote_port: u16) -> Connection {
        Connection {
            protocol: Protocol::Tcp,
            local_address: "10.0.4.17".into(),
            local_port,
            remote_address: remote.into(),
            remote_port,
            uid: 33,
            inode,
        }
    }

    fn owned(pairs: &[(u64, u32, &str)]) -> BTreeMap<u64, ProcessOwner> {
        pairs
            .iter()
            .map(|(inode, pid, program)| {
                (
                    *inode,
                    ProcessOwner {
                        pid: Some(*pid),
                        executable: Some(program.to_string()),
                        uid: Some(33),
                        ..ProcessOwner::default()
                    },
                )
            })
            .collect()
    }

    fn read(
        listening: &[SocketRow],
        established: &[Connection],
        owners: &BTreeMap<u64, ProcessOwner>,
    ) -> BTreeMap<String, Value> {
        connection_pairs(&PairsReading {
            listening,
            established,
            owners,
            users: &BTreeMap::from([(33, "www-data".to_string())]),
        })
    }

    #[test]
    fn ten_connections_of_one_program_to_one_peer_are_one_row() {
        let connections: Vec<Connection> = (0..10)
            .map(|n| talking(100 + n, 50_000 + n as u16, "203.0.113.10", 443))
            .collect();
        let owners = owned(
            &(0..10)
                .map(|n| (100 + n, 900 - n as u32, "/usr/bin/curl"))
                .collect::<Vec<_>>(),
        );

        let pairs = read(&[], &connections, &owners);

        assert_eq!(
            pairs.keys().collect::<Vec<_>>(),
            ["outbound|tcp|203.0.113.10:443|/usr/bin/curl"]
        );
        let row = &pairs["outbound|tcp|203.0.113.10:443|/usr/bin/curl"];
        assert_eq!(row["user"], "www-data");
        assert!(row.get("pid").is_none() && row.get("local_port").is_none());
    }

    #[test]
    fn a_client_of_a_port_this_host_listens_on_is_not_a_peer() {
        let pairs = read(
            &[door(443)],
            &[talking(7, 443, "198.51.100.5", 51_514)],
            &owned(&[(7, 1200, "/usr/sbin/nginx")]),
        );

        assert!(
            !pairs.keys().any(|key| key.starts_with("outbound|")),
            "{pairs:?}"
        );
    }

    #[test]
    fn a_client_of_a_port_this_host_listens_on_is_an_inbound_pair() {
        let pairs = read(
            &[door(443)],
            &[talking(7, 443, "198.51.100.5", 51_514)],
            &owned(&[(7, 1200, "/usr/sbin/nginx")]),
        );

        let row = &pairs["inbound|tcp|443|198.51.100.5|/usr/sbin/nginx"];
        assert_eq!(row["local_port"], 443);
        assert!(
            row.get("remote_port").is_none(),
            "the client's port is new on every connection, and a row carrying it would be new too"
        );
    }

    #[test]
    fn a_client_reconnecting_from_another_port_leaves_the_inbound_row_as_it_was() {
        let owners = owned(&[(7, 1200, "/usr/sbin/nginx"), (8, 1200, "/usr/sbin/nginx")]);

        let before = read(
            &[door(443)],
            &[talking(7, 443, "198.51.100.5", 51_514)],
            &owners,
        );
        let after = read(
            &[door(443)],
            &[talking(8, 443, "198.51.100.5", 60_001)],
            &owners,
        );

        assert_eq!(before, after);
    }

    #[test]
    fn a_peer_on_loopback_is_not_a_row() {
        let pairs = read(
            &[],
            &[
                talking(1, 50_000, "127.0.0.1", 5432),
                talking(2, 50_001, "::1", 5432),
                talking(3, 50_002, "::ffff:127.0.0.1", 5432),
            ],
            &BTreeMap::new(),
        );

        assert!(pairs.is_empty(), "{pairs:?}");
    }

    #[test]
    fn pairs_past_the_ceiling_are_counted_in_one_row() {
        let connections: Vec<Connection> = (0..900)
            .map(|n| talking(n, 40_000, &format!("203.0.{}.{}", n / 250, n % 250), 443))
            .collect();

        let pairs = read(&[], &connections, &BTreeMap::new());

        assert_eq!(
            pairs
                .keys()
                .filter(|key| key.starts_with("outbound|tcp"))
                .count(),
            PAIR_CEILING
        );
        assert_eq!(pairs["outbound|beyond"]["count"], 388);
        assert!(!pairs.contains_key("inbound|beyond"));
    }

    #[test]
    fn inbound_pairs_past_the_ceiling_are_counted_in_their_own_row() {
        let connections: Vec<Connection> = (0..700)
            .map(|n| talking(n, 443, &format!("198.51.{}.{}", n / 250, n % 250), 40_000))
            .collect();

        let pairs = read(&[door(443)], &connections, &BTreeMap::new());

        assert_eq!(pairs["inbound|beyond"]["count"], 188);
        assert!(!pairs.contains_key("outbound|beyond"));
    }

    #[test]
    fn a_reconnect_from_another_local_port_leaves_the_row_as_it_was() {
        let owners = owned(&[(1, 300, "/usr/bin/curl"), (2, 300, "/usr/bin/curl")]);

        let before = read(&[], &[talking(1, 50_000, "203.0.113.10", 443)], &owners);
        let after = read(&[], &[talking(2, 61_234, "203.0.113.10", 443)], &owners);

        assert_eq!(before, after);
    }

    #[test]
    fn a_pair_whose_owner_is_out_of_reach_is_recorded_as_unresolved() {
        let pairs = read(
            &[],
            &[talking(1, 50_000, "203.0.113.10", 443)],
            &BTreeMap::new(),
        );

        let row = &pairs["outbound|tcp|203.0.113.10:443|unresolved"];
        assert_eq!(row["owner_resolved"], false);
        assert_eq!(row["uid"], 33, "the uid comes from the kernel's table");
        assert!(row["program"].is_null());
    }
}
