use vigil_model::Snapshot;

use crate::parsers::{
    Container, ContainersReading, RuntimeSocket, containers_snapshot, parse_container_reference,
    parse_effective_capabilities, parse_mountinfo, paths_of_this_host,
};

const THIS_HOST: &str = "\
25 30 254:1 / / rw,relatime shared:1 - ext4 /dev/vda1 rw\n\
27 25 0:26 / /run rw,nosuid,nodev shared:5 - tmpfs tmpfs rw,size=802016k,mode=755\n\
99 25 0:98 / /var/lib/docker/overlay2/ABC/merged rw,relatime shared:180 - overlay overlay rw\n";

const OF_THE_WEB_CONTAINER: &str = "\
447 446 0:98 / / rw,relatime master:180 - overlay overlay rw\n\
448 447 0:140 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw\n\
451 447 254:1 /var/lib/docker/containers/3ab1/resolv.conf /etc/resolv.conf rw,relatime - ext4 /dev/vda1 rw\n\
452 447 254:1 /srv/www /usr/share/nginx/html ro,relatime - ext4 /dev/vda1 ro\n";

const OF_THE_AGENT_CONTAINER: &str = "\
480 479 0:98 / / rw,relatime master:181 - overlay overlay rw\n\
481 480 0:26 /docker.sock /var/run/docker.sock rw,relatime - tmpfs tmpfs rw\n";

const WEB_CGROUP: &str = "0::/system.slice/docker-3ab1c0f2d4e5a6b7c8d9e0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5.scope\n";

const AGENT_CGROUP: &str = "0::/system.slice/docker-9f2e8d7c6b5a4039281706f5e4d3c2b1a0998877665544332211ffeeddccbbaa.scope\n";

const UNPRIVILEGED: &str = "Name:\tnginx\nCapEff:\t00000000a80425fb\n";

const PRIVILEGED: &str = "Name:\tagent\nCapEff:\t000001ffffffffff\n";

pub fn containers() -> Snapshot {
    let host = parse_mountinfo(THIS_HOST);
    let containers = vec![
        container(
            WEB_CGROUP,
            "/usr/sbin/nginx",
            UNPRIVILEGED,
            OF_THE_WEB_CONTAINER,
            &host,
        ),
        container(
            AGENT_CGROUP,
            "/usr/local/bin/agent",
            PRIVILEGED,
            OF_THE_AGENT_CONTAINER,
            &host,
        ),
    ];

    containers_snapshot(
        "2026-09-09T09:00:00.000Z",
        &ContainersReading {
            containers: &containers,
            sockets: &[RuntimeSocket {
                path: "/run/docker.sock".into(),
                mode: "0660".into(),
                uid: 0,
                gid: 999,
            }],
        },
    )
}

fn container(
    cgroup: &str,
    executable: &str,
    status: &str,
    mountinfo: &str,
    host: &[crate::parsers::MountedIn],
) -> Container {
    let reference = parse_container_reference(cgroup).expect("the sample cgroup names one");
    let inside = parse_mountinfo(mountinfo);

    Container {
        id: reference.id.clone(),
        short: reference.short().to_string(),
        runtime: reference.runtime.clone(),
        executable: Some(executable.to_string()),
        capabilities_effective: parse_effective_capabilities(status),
        host_paths: paths_of_this_host(&inside, host),
        host_paths_truncated: false,
        mounts_readable: true,
    }
}
