use std::fs;
use std::sync::MutexGuard;

use vigil_model::Snapshot;

use super::ContainersCollector;
use super::source::HOST_PATH_CEILING;
use super::walk::Found;
use crate::parsers::{
    Container, ContainersReading, MountedIn, containers_snapshot, parse_effective_capabilities,
    parse_mountinfo, paths_of_this_host,
};
use vigil_collect::CollectError;

const OWN_MOUNTS: &str = "self/mountinfo";

impl ContainersCollector {
    pub(super) fn reading(&self) -> Result<Snapshot, CollectError> {
        let found = self.containers_of_this_host()?;
        let mounts_of_this_host = self.mounts_of_this_host();
        let mut complaints: Vec<String> = Vec::new();

        if mounts_of_this_host.is_empty() {
            complaints.push(format!(
                "{} could not be read, so a path of this host bound into a container cannot be \
                 named by the path this host knows it by",
                self.path(OWN_MOUNTS).display()
            ));
        }

        let containers: Vec<Container> = found
            .iter()
            .map(|container| self.read(container, &mounts_of_this_host, &mut complaints))
            .collect();
        let sockets = self.runtime_sockets();

        *self.unread() = complaints;

        Ok(containers_snapshot(
            &(self.now)(),
            &ContainersReading {
                containers: &containers,
                sockets: &sockets,
            },
        ))
    }

    pub(super) fn unread(&self) -> MutexGuard<'_, Vec<String>> {
        self.unread
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn read(
        &self,
        found: &Found,
        mounts_of_this_host: &[MountedIn],
        complaints: &mut Vec<String>,
    ) -> Container {
        let pid = found.pid;
        let short = found.reference.short().to_string();

        let capabilities_effective = fs::read_to_string(self.path(&format!("{pid}/status")))
            .ok()
            .and_then(|status| parse_effective_capabilities(&status));
        if capabilities_effective.is_none() {
            complaints.push(format!(
                "the capabilities container {short} runs with could not be read from {}, so a \
                 privileged container among them would not be seen",
                self.path(&format!("{pid}/status")).display()
            ));
        }

        let inside = fs::read_to_string(self.path(&format!("{pid}/mountinfo")))
            .ok()
            .map(|text| parse_mountinfo(&text));
        if inside.is_none() {
            complaints.push(format!(
                "what container {short} has mounted could not be read from {}, so a path of \
                 this host bound into it would not be seen",
                self.path(&format!("{pid}/mountinfo")).display()
            ));
        }

        let mut host_paths = inside
            .as_deref()
            .map(|inside| paths_of_this_host(inside, mounts_of_this_host))
            .unwrap_or_default();
        let host_paths_truncated = host_paths.len() > HOST_PATH_CEILING;
        host_paths.truncate(HOST_PATH_CEILING);

        Container {
            id: found.reference.id.clone(),
            short,
            runtime: found.reference.runtime.clone(),
            executable: fs::read_link(self.path(&format!("{pid}/exe")))
                .ok()
                .map(|path| path.display().to_string()),
            capabilities_effective,
            host_paths,
            host_paths_truncated,
            mounts_readable: inside.is_some(),
        }
    }

    fn mounts_of_this_host(&self) -> Vec<MountedIn> {
        fs::read_to_string(self.path(OWN_MOUNTS))
            .ok()
            .map(|text| parse_mountinfo(&text))
            .unwrap_or_default()
    }
}
