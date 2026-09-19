use std::collections::{BTreeMap, BTreeSet};

use vigil_collect::{CollectError, processes_running};

use super::descriptors::{open_descriptors, socket_information};
use crate::parsers::{Door, SocketRow, UnixSocketRow, door_of, socket_descriptors};

#[derive(Default)]
pub(super) struct Gathered {
    pub(super) network: BTreeMap<u64, SocketRow>,
    pub(super) unix: BTreeMap<u64, UnixSocketRow>,
    pub(super) unnamed: BTreeSet<u64>,
    pub(super) holders: BTreeMap<u64, u32>,
    pub(super) uids: BTreeMap<u32, u32>,
    pub(super) looked_at: usize,
    pub(super) refused: usize,
    pub(super) unparsed: usize,
}

pub(super) fn gathered() -> Result<Gathered, CollectError> {
    let mut gathered = Gathered::default();

    for entry in processes_running()? {
        if entry.pid == 0 || entry.zombie {
            continue;
        }
        let listed = match open_descriptors(entry.pid) {
            Ok(listed) => listed,
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => continue,
            Err(_) => {
                gathered.refused += 1;
                continue;
            }
        };
        gathered.looked_at += 1;
        gathered.uids.insert(entry.pid, entry.effective_uid);

        let Some(sockets) = socket_descriptors(&listed) else {
            gathered.unparsed += 1;
            continue;
        };
        for descriptor in sockets {
            let Ok(information) = socket_information(entry.pid, descriptor) else {
                continue;
            };
            let Some(door) = door_of(&information) else {
                gathered.unparsed += 1;
                continue;
            };
            gathered.held(door, entry.pid);
        }
    }

    Ok(gathered)
}

impl Gathered {
    fn held(&mut self, door: Door, pid: u32) {
        let handle = match door {
            Door::Ignored => return,
            Door::Network(row) => {
                let handle = row.inode;
                self.network.entry(handle).or_insert(row);
                handle
            }
            Door::Unix(row) => {
                let handle = row.inode;
                self.unix.entry(handle).or_insert(row);
                handle
            }
            Door::UnnamedUnix(handle) => {
                self.unnamed.insert(handle);
                return;
            }
        };

        self.holders
            .entry(handle)
            .and_modify(|lowest| *lowest = (*lowest).min(pid))
            .or_insert(pid);
    }
}
