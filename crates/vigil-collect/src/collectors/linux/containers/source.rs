use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::Rfc3339;

use super::ContainersCollector;

pub(super) const NAME: &str = "containers";

pub(super) const PROC: &str = "/proc";

pub(super) const RUNTIME_SOCKETS: &[&str] = &[
    "/var/run/docker.sock",
    "/run/docker.sock",
    "/run/podman/podman.sock",
    "/run/containerd/containerd.sock",
    "/var/run/crio/crio.sock",
];

pub(super) const PROCESS_CEILING: usize = 8192;

pub(super) const CONTAINER_CEILING: usize = 128;

pub(super) const HOST_PATH_CEILING: usize = 32;

impl ContainersCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        ContainersCollector::with_sources(now, PROC, RUNTIME_SOCKETS)
    }

    pub fn with_sources(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        proc_directory: impl Into<PathBuf>,
        socket_paths: &[&str],
    ) -> Self {
        ContainersCollector {
            now: Box::new(now),
            proc_directory: proc_directory.into(),
            socket_paths: socket_paths.iter().map(PathBuf::from).collect(),
            unread: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn path(&self, under: &str) -> PathBuf {
        self.proc_directory.join(under)
    }
}
