use serde_json::json;
use vigil_model::Snapshot;

pub const SOURCE: &str = "containers";

pub const CONTAINER: &str = "container";

pub const SOCKET: &str = "container-socket";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    pub short: String,
    pub runtime: String,
    pub executable: Option<String>,
    pub capabilities_effective: Option<String>,
    pub host_paths: Vec<String>,
    pub host_paths_truncated: bool,
    pub mounts_readable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSocket {
    pub path: String,
    pub mode: String,
    pub uid: u32,
    pub gid: u32,
}

pub struct ContainersReading<'a> {
    pub containers: &'a [Container],
    pub sockets: &'a [RuntimeSocket],
}

pub fn containers_snapshot(taken_at: &str, reading: &ContainersReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    for container in reading.containers {
        snapshot.items.insert(
            format!("{CONTAINER}|{}", container.short),
            json!({
                "id": container.id,
                "runtime": container.runtime,
                "exe": container.executable,
                "capabilities_effective": container.capabilities_effective,
                "host_paths": container.host_paths,
                "host_paths_truncated": container.host_paths_truncated,
                "mounts_readable": container.mounts_readable,
            }),
        );
    }

    for socket in reading.sockets {
        snapshot.items.insert(
            format!("{SOCKET}|{}", socket.path),
            json!({
                "path": socket.path,
                "mode": socket.mode,
                "uid": socket.uid,
                "gid": socket.gid,
            }),
        );
    }

    snapshot
}

#[cfg(test)]
mod tests {
    use vigil_model::class_of;

    use super::*;

    const AT: &str = "2026-09-11T12:00:00.000Z";

    const ID: &str = "3ab1c0f2d4e5a6b7c8d9e0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5";

    fn container() -> Container {
        Container {
            id: ID.to_string(),
            short: ID[..12].to_string(),
            runtime: "docker".into(),
            executable: Some("/usr/sbin/nginx".into()),
            capabilities_effective: Some("00000000a80425fb".into()),
            host_paths: vec!["/srv/www".into()],
            host_paths_truncated: false,
            mounts_readable: true,
        }
    }

    fn socket() -> RuntimeSocket {
        RuntimeSocket {
            path: "/run/docker.sock".into(),
            mode: "0660".into(),
            uid: 0,
            gid: 999,
        }
    }

    fn reading(containers: &[Container], sockets: &[RuntimeSocket]) -> Snapshot {
        containers_snapshot(
            AT,
            &ContainersReading {
                containers,
                sockets,
            },
        )
    }

    #[test]
    fn a_container_is_one_row_named_by_the_twelve_characters_a_person_reads_it_by() {
        let taken = reading(&[container()], &[socket()]);

        assert_eq!(taken.source, SOURCE);
        assert_eq!(taken.items["container|3ab1c0f2d4e5"]["id"], ID);
        assert_eq!(taken.items["container|3ab1c0f2d4e5"]["runtime"], "docker");
        assert_eq!(
            taken.items["container|3ab1c0f2d4e5"]["host_paths"][0],
            "/srv/www"
        );
        assert_eq!(
            taken.items["container-socket|/run/docker.sock"]["mode"],
            "0660"
        );
    }

    #[test]
    fn a_container_and_the_socket_of_its_runtime_are_two_classes_of_row() {
        let taken = reading(&[container()], &[socket()]);

        let mut classes: Vec<&str> = taken.items.keys().map(|key| class_of(key)).collect();
        classes.sort_unstable();

        assert_eq!(classes, vec!["container", "container-socket"]);
    }

    #[test]
    fn a_container_whose_mounts_could_not_be_read_says_so_rather_than_naming_no_paths() {
        let mut unread = container();
        unread.host_paths = Vec::new();
        unread.mounts_readable = false;

        let taken = reading(&[unread], &[]);
        let row = &taken.items["container|3ab1c0f2d4e5"];

        assert_eq!(row["mounts_readable"], false);
        assert_eq!(
            row["host_paths"].as_array().map(Vec::len),
            Some(0),
            "an empty list beside a mark saying it was never read is the one thing that is \
             not a container which mounted nothing of this host"
        );
    }

    #[test]
    fn how_many_processes_a_container_runs_is_nowhere_in_the_reading() {
        let taken = reading(&[container()], &[socket()]);
        let row = &taken.items["container|3ab1c0f2d4e5"];

        assert!(
            row.get("processes").is_none() && row.get("pid").is_none(),
            "the pid a container starts under changes whenever it is restarted and the number \
             of its processes changes between any two readings: a reading carrying either \
             differs from the one before it on a host that did not move"
        );
    }

    #[test]
    fn a_host_running_no_containers_is_a_reading_with_the_runtime_socket_still_in_it() {
        let taken = reading(&[], &[socket()]);

        assert_eq!(taken.items.len(), 1);
        assert!(
            taken
                .items
                .contains_key("container-socket|/run/docker.sock")
        );
    }
}
