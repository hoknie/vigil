use std::collections::BTreeMap;

use vigil_model::Snapshot;

use crate::parsers::{
    ProcessOwner, Protocol, SocketsReading, listening_snapshot, parse_net_table, parse_unix_table,
};

const TCP: &str = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 20480 1
   1: 0100007F:1538 00000000:0000 0A 00000000:00000000 00:00000000 00000000   106        0 23901 1
   2: 00000000:115C 00000000:0000 0A 00000000:00000000 00:00000000 00000000    33        0 23902 1
";

const TCP6: &str = "\
  sl  local_address                         remote_address                        st  uid inode
   0: 00000000000000000000000000000000:01BB 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 20500 1
   1: 00000000000000000000000000000000:1F90 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000   106        0 20501 1
";

const UDP: &str = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000:0035 00000000:0000 07 00000000:00000000 00:00000000 00000000   101        0 21000 2
   1: 00000000:0044 00000000:0000 07 00000000:00000000 00:00000000 00000000   106        0 21002 2
";

const UDP6: &str = "\
  sl  local_address                         remote_address                        st  uid inode
   0: 00000000000000000000000000000000:0222 00000000000000000000000000000000:0000 07 00000000:00000000 00:00000000 00000000     0        0 21001 2
   1: 00000000000000000000000000000000:14E9 00000000000000000000000000000000:0000 07 00000000:00000000 00:00000000 00000000   106        0 21003 2
";

const UNIX: &str = "\
Num       RefCount Protocol Flags    Type St Inode Path
0000: 00000002 00000000 00010000 0001 01 24875 /run/docker.sock
0000: 00000002 00000000 00010000 0001 01 24876 @/tmp/.X11-unix/X0
0000: 00000002 00000000 00010000 0001 01 24877
";

fn owner(executable: Option<&str>, command_line: Option<&str>, uid: u32) -> ProcessOwner {
    ProcessOwner {
        executable: executable.map(str::to_string),
        executable_deleted: executable.is_some_and(|path| path.starts_with("/tmp/")),
        command_line: command_line.map(str::to_string),
        command_line_redacted: command_line.is_some_and(|line| line.contains("[redacted]")),
        uid: Some(uid),
    }
}

fn owners() -> BTreeMap<u64, ProcessOwner> {
    BTreeMap::from([
        (
            20480,
            owner(
                Some("/usr/sbin/sshd"),
                Some("sshd: /usr/sbin/sshd -D [listener]"),
                0,
            ),
        ),
        (23902, owner(None, None, 33)),
        (
            20500,
            owner(
                Some("/usr/sbin/nginx"),
                Some("nginx: master process /usr/sbin/nginx"),
                0,
            ),
        ),
        (
            21000,
            owner(
                Some("/lib/systemd/systemd-resolved"),
                Some("/lib/systemd/systemd-resolved"),
                101,
            ),
        ),
        (
            21001,
            owner(
                Some("/usr/sbin/dhclient"),
                Some("dhclient -6 --password [redacted]"),
                0,
            ),
        ),
        (
            24875,
            owner(
                Some("/usr/bin/dockerd"),
                Some("dockerd --host unix:///run/docker.sock"),
                0,
            ),
        ),
    ])
}

pub fn ports() -> Snapshot {
    let mut network = parse_net_table(TCP, Protocol::Tcp).listening;
    network.extend(parse_net_table(TCP6, Protocol::Tcp6).listening);
    network.extend(parse_net_table(UDP, Protocol::Udp).listening);
    network.extend(parse_net_table(UDP6, Protocol::Udp6).listening);

    let unix = parse_unix_table(UNIX);
    let owners = owners();
    let users = BTreeMap::from([
        (0, "root".to_string()),
        (33, "www-data".to_string()),
        (101, "systemd-resolve".to_string()),
    ]);

    listening_snapshot(
        "2026-09-09T09:00:00.000Z",
        &SocketsReading {
            network: &network,
            unix: &unix.listening,
            unnamed_unix: unix.unnamed,
            owners: &owners,
            users: &users,
        },
    )
}
