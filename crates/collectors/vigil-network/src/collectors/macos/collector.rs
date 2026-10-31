use vigil_collect::{CollectError, Collector, Health, arguments_buffer, names_of_users};
use vigil_model::{Rfc3339, Snapshot};

use super::gathering::gathered;
use super::health::health;
use super::owners::described;
use crate::parsers::{
    Connection, PairsReading, SocketRow, SocketsReading, UnixSocketRow, connection_pairs,
    listening_snapshot,
};

pub struct NetworkCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    connections: bool,
}

impl NetworkCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        NetworkCollector {
            now: Box::new(now),
            connections: true,
        }
    }

    pub fn reading_connections(self, connections: bool) -> Self {
        NetworkCollector {
            connections,
            ..self
        }
    }
}

impl Collector for NetworkCollector {
    fn name(&self) -> &'static str {
        "network"
    }

    fn available(&self) -> Health {
        health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let gathered = gathered()?;
        if gathered.looked_at == 0 {
            return Err(CollectError::Denied(format!(
                "the open files of any of {} process(es)",
                gathered.refused
            )));
        }
        if gathered.unparsed > 0 && gathered.network.is_empty() && gathered.unix.is_empty() {
            return Err(CollectError::Unreadable(format!(
                "{} socket record(s) in a shape this build does not know",
                gathered.unparsed
            )));
        }

        let mut buffer = arguments_buffer();
        let owners = described(&gathered.holders, &gathered.uids, &mut buffer);

        let network: Vec<SocketRow> = gathered
            .network
            .into_iter()
            .map(|(handle, row)| SocketRow {
                uid: owners
                    .get(&handle)
                    .and_then(|owner| owner.uid)
                    .unwrap_or(row.uid),
                ..row
            })
            .collect();
        let established: Vec<Connection> = gathered
            .established
            .into_iter()
            .map(|(handle, connection)| Connection {
                uid: owners
                    .get(&handle)
                    .and_then(|owner| owner.uid)
                    .unwrap_or(connection.uid),
                ..connection
            })
            .collect();
        let unix: Vec<UnixSocketRow> = gathered.unix.into_values().collect();
        let users = names_of_users(
            network
                .iter()
                .map(|row| row.uid)
                .chain(established.iter().map(|connection| connection.uid))
                .chain(owners.values().filter_map(|owner| owner.uid)),
        );
        let pairs = self.connections.then(|| {
            connection_pairs(&PairsReading {
                listening: &network,
                established: &established,
                owners: &owners,
                users: &users,
            })
        });

        Ok(listening_snapshot(
            &(self.now)(),
            &SocketsReading {
                network: &network,
                unix: &unix,
                unnamed_unix: gathered.unnamed.len(),
                owners: &owners,
                users: &users,
                pairs: pairs.as_ref(),
            },
        ))
    }
}
