use super::docker::{AT, DEADLINE_SECONDS, answered};
use crate::types::{Dump, Engine, Subject};

pub const PROGRAM: &str = "/usr/bin/podman";

pub const REGISTRIES_CONF: &str = r#"
unqualified-search-registries = ["docker.io", "quay.io"]

[[registry]]
prefix = "registry.local:5000"
location = "registry.local:5000"
insecure = true

[[registry]]
location = "registry.internal"
insecure = false
"#;

const IMAGES: &str = r#"[
  {"Id":"7c8f3a1b5d2e4906f8a7b6c5d4e3f2a1908070605040302010fedcba98765432","RepoTags":["quay.io/podman/hello:latest"],"RepoDigests":["quay.io/podman/hello@sha256:1f0d2c3b4a59687756453423121f0e0d0c0b0a0908070605040302010fedcba9"],"Size":579328,"Created":1755000000,"CreatedAt":"2026-08-12T10:00:00Z","Labels":{"org.opencontainers.image.vendor":"podman"},"Containers":1},
  {"Id":"b2d6e4f80a1c3957e6d5c4b3a2918070f6e5d4c3b2a1908070605040302010fe","RepoTags":[],"RepoDigests":[],"Size":214958080,"Created":1757900000,"CreatedAt":"2026-09-15T04:11:00Z","Labels":{},"Containers":0}
]
"#;

const VOLUMES: &str = r#"[
  {"Name":"etc_backup","Driver":"local","Mountpoint":"/var/lib/containers/storage/volumes/etc_backup/_data","CreatedAt":"2026-09-02T06:30:00Z","Labels":{"role":"backup"},"Scope":"local","Options":{"device":"/etc","o":"bind","type":"none"},"MountCount":1,"NeedsCopyUp":false},
  {"Name":"tools_home","Driver":"local","Mountpoint":"/var/lib/containers/storage/volumes/tools_home/_data","CreatedAt":"2026-09-02T06:31:00Z","Labels":{},"Scope":"local","Options":{},"MountCount":0,"NeedsCopyUp":true}
]
"#;

const NETWORKS: &str = r#"[
  {"name":"podman","id":"2f259bab93aaaaa2542ba43ef33eb990d0999ee1b9924b557b7be53c0b7a1bb9","driver":"bridge","network_interface":"podman0","created":"2026-05-01T08:00:00Z","subnets":[{"subnet":"10.88.0.0/16","gateway":"10.88.0.1"}],"ipv6_enabled":false,"internal":false,"dns_enabled":false,"labels":{}},
  {"name":"podman1","id":"8e4d1c0b7a396584f3e2d1c0b9a8978675645342312f0e0d0c0b0a0908070605","driver":"bridge","network_interface":"podman1","created":"2026-09-02T06:29:00Z","subnets":[{"subnet":"10.89.0.0/24","gateway":"10.89.0.1"}],"ipv6_enabled":false,"internal":true,"dns_enabled":true,"labels":{"role":"tools"}}
]
"#;

const CONTAINERS: &str = r#"[
  {"Id":"6d1e9f4a0c73b852a1908070605040302010fedcba9876543210fedcba987654","Names":["tools-infra"],"Image":"localhost/podman-pause:5.1.1-1","ImageID":"aa11bb22cc33dd44ee55ff6677889900aabbccddeeff00112233445566778899","Labels":{},"Mounts":[],"Networks":["podman1"],"Pod":"3f2e1d0c9b8a7766554433221100ffeeddccbbaa99887766554433221100ffee","PodName":"tools","Ports":[],"State":"running","Status":"Up 6 days","Created":1757000000,"IsInfra":true,"Restarts":0,"AutoRemove":false},
  {"Id":"9a0b1c2d3e4f5061728394a5b6c7d8e9f0a1b2c3d4e5f60718293a4b5c6d7e8f","Names":["tools-shell"],"Image":"quay.io/podman/hello:latest","ImageID":"7c8f3a1b5d2e4906f8a7b6c5d4e3f2a1908070605040302010fedcba98765432","Labels":{"role":"shell","com.corp.vault-token":"s.0011223344556677"},"Mounts":["/etc","tools_home"],"Networks":["podman1"],"Pod":"3f2e1d0c9b8a7766554433221100ffeeddccbbaa99887766554433221100ffee","PodName":"tools","Ports":[{"host_ip":"127.0.0.1","container_port":2222,"host_port":2222,"range":1,"protocol":"tcp"}],"State":"running","Status":"Up 6 days","Created":1757000100,"IsInfra":false,"Restarts":3,"AutoRemove":false}
]
"#;

const PODS: &str = r#"[
  {"Cgroup":"machine.slice","Containers":[{"Id":"6d1e9f4a0c73","Names":"tools-infra","Status":"running"},{"Id":"9a0b1c2d3e4f","Names":"tools-shell","Status":"running"}],"Created":"2026-09-11T05:00:00Z","Id":"3f2e1d0c9b8a7766554433221100ffeeddccbbaa99887766554433221100ffee","InfraId":"6d1e9f4a0c73b852a1908070605040302010fedcba9876543210fedcba987654","Labels":{"role":"tools"},"Name":"tools","Namespace":"","Networks":["podman1"],"Status":"Running"}
]
"#;

const SECRETS: &str = r#"[
  {"ID":"4c1a9e7b3d05f286a190807060504030","CreatedAt":"2026-09-02T06:28:00Z","UpdatedAt":"2026-09-02T06:28:00Z","Spec":{"Name":"shop-database-password","Driver":{"Name":"file","Options":{"path":"/var/lib/containers/storage/secrets/filedriver"}},"Labels":{}}}
]
"#;

const INFORMATION: &str = r#"{"host":{"cgroupManager":"systemd","cgroupVersion":"v2","logDriver":"journald","security":{"rootless":true,"apparmorEnabled":false,"seccompEnabled":true,"selinuxEnabled":false},"arch":"amd64","kernel":"6.1.0-23-amd64","uptime":"146h 3m 11s"},"store":{"graphDriverName":"overlay","graphRoot":"/var/lib/containers/storage","containerStore":{"number":2,"running":2},"imageStore":{"number":2}},"version":{"APIVersion":"5.1.1","Version":"5.1.1","GoVersion":"go1.22.4"},"registries":{"search":["docker.io","quay.io"]}}"#;

pub fn dump() -> Dump {
    let mut dump = Dump::present(Engine::Podman.name(), AT, DEADLINE_SECONDS, PROGRAM);

    for (subject, printed, milliseconds) in [
        (Subject::Image, IMAGES, 88),
        (Subject::Volume, VOLUMES, 52),
        (Subject::Network, NETWORKS, 49),
        (Subject::Container, CONTAINERS, 95),
        (Subject::Engine, INFORMATION, 140),
        (Subject::Pod, PODS, 51),
        (Subject::Secret, SECRETS, 40),
    ] {
        dump.asked.insert(
            subject.as_str().to_string(),
            answered(Engine::Podman, subject, printed, milliseconds),
        );
    }

    dump
}
