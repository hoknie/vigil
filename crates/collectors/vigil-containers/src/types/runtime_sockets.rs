pub const RUNTIME_SOCKETS: &[&str] = &[
    "/run/docker.sock",
    "/run/podman/podman.sock",
    "/run/containerd/containerd.sock",
    "/var/run/docker.sock",
    "/var/run/podman/podman.sock",
    "/var/run/crio/crio.sock",
];

pub fn is_a_runtime_socket(path: &str) -> bool {
    RUNTIME_SOCKETS.contains(&path)
}
