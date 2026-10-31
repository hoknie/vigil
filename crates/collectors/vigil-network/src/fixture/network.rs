use std::collections::BTreeMap;

use vigil_model::Snapshot;

use crate::parsers::{
    PairsReading, ProcessOwner, Protocol, SocketsReading, connection_pairs, listening_snapshot,
    parse_net_table, parse_unix_table,
};

const TCP: &str = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 20480 1
   1: 0100007F:1538 00000000:0000 0A 00000000:00000000 00:00000000 00000000   106        0 23901 1
   2: 00000000:115C 00000000:0000 0A 00000000:00000000 00:00000000 00000000    33        0 23902 1
   3: 1104000A:C350 0A7100CB:01BB 01 00000000:00000000 00:00000000 00000000     0        0 30001 1
   4: 1104000A:C351 077100CB:115C 01 00000000:00000000 00:00000000 00000000    33        0 30002 1
   5: 1104000A:0016 056433C6:D431 01 00000000:00000000 00:00000000 00000000     0        0 30003 1
   6: 0100007F:C352 0100007F:1538 01 00000000:00000000 00:00000000 00000000  1000        0 30004 1
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

fn owner(pid: u32, executable: Option<&str>, command_line: Option<&str>, uid: u32) -> ProcessOwner {
    ProcessOwner {
        pid: Some(pid),
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
                812,
                Some("/usr/sbin/sshd"),
                Some("sshd: /usr/sbin/sshd -D [listener]"),
                0,
            ),
        ),
        (23902, owner(2291, None, None, 33)),
        (
            30001,
            owner(
                1500,
                Some("/usr/bin/curl"),
                Some("curl https://203.0.113.10/health"),
                0,
            ),
        ),
        (
            30002,
            owner(2600, Some("/tmp/.x/nc"), Some("nc 203.0.113.7 4444"), 33),
        ),
        (
            30003,
            owner(3100, Some("/usr/sbin/sshd"), Some("sshd: alice [priv]"), 0),
        ),
        (
            20500,
            owner(
                1042,
                Some("/usr/sbin/nginx"),
                Some("nginx: master process /usr/sbin/nginx"),
                0,
            ),
        ),
        (
            21000,
            owner(
                640,
                Some("/lib/systemd/systemd-resolved"),
                Some("/lib/systemd/systemd-resolved"),
                101,
            ),
        ),
        (
            21001,
            owner(
                701,
                Some("/usr/sbin/dhclient"),
                Some("dhclient -6 --password [redacted]"),
                0,
            ),
        ),
        (
            24875,
            owner(
                980,
                Some("/usr/bin/dockerd"),
                Some("dockerd --host unix:///run/docker.sock"),
                0,
            ),
        ),
    ])
}

pub fn network() -> Snapshot {
    let tcp = parse_net_table(TCP, Protocol::Tcp);
    let established = tcp.established;
    let mut network = tcp.listening;
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

    let pairs = connection_pairs(&PairsReading {
        listening: &network,
        established: &established,
        owners: &owners,
        users: &users,
    });

    listening_snapshot(
        "2026-09-09T09:00:00.000Z",
        &SocketsReading {
            network: &network,
            unix: &unix.listening,
            unnamed_unix: unix.unnamed,
            owners: &owners,
            users: &users,
            pairs: Some(&pairs),
        },
    )
}
