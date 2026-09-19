use std::net::{Ipv4Addr, Ipv6Addr};

use super::proc_net::{Protocol, SocketRow};
use super::proc_net_unix::{UnixKind, UnixSocketRow};

pub const SOCKET_INFORMATION_BYTES: usize = 792;

const OWNER: usize = 40;

const HANDLE: usize = 160;

const TYPE: usize = 176;

const PROTOCOL: usize = 180;

const FAMILY: usize = 184;

const OPTIONS: usize = 188;

const KIND: usize = 256;

const FOREIGN_PORT: usize = 264;

const LOCAL_PORT: usize = 268;

const VERSION: usize = 288;

const LOCAL_ADDRESS: usize = 312;

const TCP_STATE: usize = 344;

const UNIX_ADDRESS: usize = 280;

const UNIX_PATH: usize = 282;

const UNIX_PATH_BYTES: usize = 104;

const INTERNET: i32 = 1;

const TCP: i32 = 2;

const UNIX: i32 = 3;

const FAMILY_INTERNET: i32 = 2;

const FAMILY_INTERNET_6: i32 = 30;

const PROTOCOL_UDP: i32 = 17;

const ONLY_VERSION_4: u8 = 1;

const IPV6_PART: u8 = 2;

const LISTENING: i32 = 1;

const ACCEPTING: i16 = 0x0002;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Door {
    Network(SocketRow),
    Unix(UnixSocketRow),
    UnnamedUnix(u64),
    Ignored,
}

pub fn door_of(bytes: &[u8]) -> Option<Door> {
    if bytes.len() < SOCKET_INFORMATION_BYTES {
        return None;
    }

    let handle = u64::from_ne_bytes(bytes[HANDLE..HANDLE + 8].try_into().ok()?);
    Some(match signed(bytes, KIND)? {
        TCP => tcp(bytes, handle)?,
        INTERNET => udp(bytes, handle)?,
        UNIX => unix(bytes, handle)?,
        _ => Door::Ignored,
    })
}

fn tcp(bytes: &[u8], handle: u64) -> Option<Door> {
    if signed(bytes, TCP_STATE)? != LISTENING {
        return Some(Door::Ignored);
    }
    network(bytes, handle, false)
}

fn udp(bytes: &[u8], handle: u64) -> Option<Door> {
    if signed(bytes, PROTOCOL)? != PROTOCOL_UDP
        || port(bytes, FOREIGN_PORT)? != 0
        || port(bytes, LOCAL_PORT)? == 0
    {
        return Some(Door::Ignored);
    }
    network(bytes, handle, true)
}

fn network(bytes: &[u8], handle: u64, datagram: bool) -> Option<Door> {
    let version = *bytes.get(VERSION)?;
    let local = bytes.get(LOCAL_ADDRESS..LOCAL_ADDRESS + 16)?;

    let (protocol, address) = match signed(bytes, FAMILY)? {
        FAMILY_INTERNET => (
            (Protocol::Tcp, Protocol::Udp),
            Ipv4Addr::new(local[12], local[13], local[14], local[15]).to_string(),
        ),
        FAMILY_INTERNET_6 if version & IPV6_PART == 0 && version & ONLY_VERSION_4 != 0 => (
            (Protocol::Tcp6, Protocol::Udp6),
            Ipv4Addr::new(local[12], local[13], local[14], local[15])
                .to_ipv6_mapped()
                .to_string(),
        ),
        FAMILY_INTERNET_6 => (
            (Protocol::Tcp6, Protocol::Udp6),
            Ipv6Addr::from(<[u8; 16]>::try_from(local).ok()?).to_string(),
        ),
        _ => return Some(Door::Ignored),
    };

    Some(Door::Network(SocketRow {
        protocol: match datagram {
            true => protocol.1,
            false => protocol.0,
        },
        address,
        port: port(bytes, LOCAL_PORT)?,
        uid: u32::from_ne_bytes(bytes.get(OWNER..OWNER + 4)?.try_into().ok()?),
        inode: handle,
    }))
}

fn unix(bytes: &[u8], handle: u64) -> Option<Door> {
    let kind = match signed(bytes, TYPE)? {
        1 => UnixKind::Stream,
        2 => UnixKind::Datagram,
        5 => UnixKind::SeqPacket,
        _ => return Some(Door::Ignored),
    };
    let accepting =
        i16::from_ne_bytes(bytes.get(OPTIONS..OPTIONS + 2)?.try_into().ok()?) & ACCEPTING != 0;

    let length = usize::from(*bytes.get(UNIX_ADDRESS)?)
        .saturating_sub(UNIX_PATH - UNIX_ADDRESS)
        .min(UNIX_PATH_BYTES);
    let path = bytes.get(UNIX_PATH..UNIX_PATH + length)?;
    let end = path
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(path.len());
    let name = String::from_utf8_lossy(&path[..end]).into_owned();

    let door = match kind {
        UnixKind::Stream | UnixKind::SeqPacket => accepting,
        UnixKind::Datagram => name.starts_with('/'),
    };
    Some(match (door, name.is_empty()) {
        (false, _) => Door::Ignored,
        (true, true) => Door::UnnamedUnix(handle),
        (true, false) => Door::Unix(UnixSocketRow {
            kind,
            name,
            is_abstract: false,
            inode: handle,
        }),
    })
}

fn signed(bytes: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_ne_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn port(bytes: &[u8], at: usize) -> Option<u16> {
    let held = signed(bytes, at)?;
    Some(u16::from_be(u16::try_from(held & 0xffff).ok()?))
}

#[cfg(test)]
struct Sample {
    bytes: Vec<u8>,
}

#[cfg(test)]
impl Sample {
    fn socket(kind: i32, family: i32, socket_type: i32, protocol: i32) -> Sample {
        let mut bytes = vec![0u8; SOCKET_INFORMATION_BYTES];
        bytes[KIND..KIND + 4].copy_from_slice(&kind.to_ne_bytes());
        bytes[FAMILY..FAMILY + 4].copy_from_slice(&family.to_ne_bytes());
        bytes[TYPE..TYPE + 4].copy_from_slice(&socket_type.to_ne_bytes());
        bytes[PROTOCOL..PROTOCOL + 4].copy_from_slice(&protocol.to_ne_bytes());
        Sample { bytes }
    }

    fn tcp(family: i32, state: i32) -> Sample {
        let mut sample = Sample::socket(TCP, family, 1, 6);
        sample.bytes[TCP_STATE..TCP_STATE + 4].copy_from_slice(&state.to_ne_bytes());
        sample
    }

    fn udp(family: i32) -> Sample {
        Sample::socket(INTERNET, family, 2, PROTOCOL_UDP)
    }

    fn unix(socket_type: i32, accepting: bool, path: &str) -> Sample {
        let mut sample = Sample::socket(UNIX, 1, socket_type, 0);
        if accepting {
            sample.bytes[OPTIONS..OPTIONS + 2].copy_from_slice(&ACCEPTING.to_ne_bytes());
        }
        sample.bytes[UNIX_ADDRESS] = u8::try_from(path.len() + 2).expect("short");
        sample.bytes[UNIX_ADDRESS + 1] = 1;
        sample.bytes[UNIX_PATH..UNIX_PATH + path.len()].copy_from_slice(path.as_bytes());
        sample
    }

    fn port(mut self, local: u16, foreign: u16) -> Sample {
        for (at, port) in [(LOCAL_PORT, local), (FOREIGN_PORT, foreign)] {
            let held = i32::from(u16::from_ne_bytes(port.to_be_bytes()));
            self.bytes[at..at + 4].copy_from_slice(&held.to_ne_bytes());
        }
        self
    }

    fn address(mut self, version: u8, local: [u8; 16]) -> Sample {
        self.bytes[VERSION] = version;
        self.bytes[LOCAL_ADDRESS..LOCAL_ADDRESS + 16].copy_from_slice(&local);
        self
    }

    fn owned(mut self, uid: u32, handle: u64) -> Sample {
        self.bytes[OWNER..OWNER + 4].copy_from_slice(&uid.to_ne_bytes());
        self.bytes[HANDLE..HANDLE + 8].copy_from_slice(&handle.to_ne_bytes());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(address: [u8; 4]) -> [u8; 16] {
        let mut held = [0u8; 16];
        held[12..].copy_from_slice(&address);
        held
    }

    #[test]
    fn a_tcp_socket_listening_on_every_address_is_a_door_with_its_port_and_its_owner() {
        let sample = Sample::tcp(FAMILY_INTERNET, LISTENING)
            .port(22, 0)
            .address(ONLY_VERSION_4, v4([0, 0, 0, 0]))
            .owned(0, 0xfeed);

        assert_eq!(
            door_of(&sample.bytes),
            Some(Door::Network(SocketRow {
                protocol: Protocol::Tcp,
                address: "0.0.0.0".into(),
                port: 22,
                uid: 0,
                inode: 0xfeed,
            }))
        );
    }

    #[test]
    fn the_port_is_read_in_the_order_the_network_writes_it_and_not_the_hosts() {
        let sample = Sample::tcp(FAMILY_INTERNET, LISTENING)
            .port(5432, 0)
            .address(ONLY_VERSION_4, v4([127, 0, 0, 1]));

        let Some(Door::Network(row)) = door_of(&sample.bytes) else {
            panic!("a door")
        };
        assert_eq!(row.port, 5432, "0x1538 read the other way round is 14357");
        assert_eq!(row.address, "127.0.0.1");
    }

    #[test]
    fn a_tcp_conversation_is_not_a_door() {
        let established = Sample::tcp(FAMILY_INTERNET, 4).port(50123, 443);

        assert_eq!(door_of(&established.bytes), Some(Door::Ignored));
    }

    #[test]
    fn a_v6_socket_is_read_in_the_v6_table_with_its_own_address_as_linux_prints_it() {
        let mut loopback = [0u8; 16];
        loopback[15] = 1;
        let any = Sample::tcp(FAMILY_INTERNET_6, LISTENING)
            .port(80, 0)
            .address(ONLY_VERSION_4 | IPV6_PART, [0u8; 16]);
        let local = Sample::tcp(FAMILY_INTERNET_6, LISTENING)
            .port(8080, 0)
            .address(IPV6_PART, loopback);

        let Some(Door::Network(any)) = door_of(&any.bytes) else {
            panic!("a door")
        };
        let Some(Door::Network(local)) = door_of(&local.bytes) else {
            panic!("a door")
        };
        assert_eq!(
            (any.protocol, any.address.as_str()),
            (Protocol::Tcp6, "::"),
            "a socket on :: that also takes v4 is one socket in the v6 table, as on Linux"
        );
        assert_eq!(local.address, "::1");
    }

    #[test]
    fn a_v6_socket_bound_to_a_v4_address_is_written_the_way_linux_writes_a_mapped_one() {
        let sample = Sample::tcp(FAMILY_INTERNET_6, LISTENING)
            .port(3000, 0)
            .address(ONLY_VERSION_4, v4([127, 0, 0, 1]));

        let Some(Door::Network(row)) = door_of(&sample.bytes) else {
            panic!("a door")
        };
        assert_eq!(row.address, "::ffff:127.0.0.1");
        assert_eq!(row.protocol, Protocol::Tcp6);
    }

    #[test]
    fn a_udp_socket_with_no_peer_is_a_door_and_one_with_a_peer_is_a_conversation() {
        let door = Sample::udp(FAMILY_INTERNET)
            .port(5353, 0)
            .address(ONLY_VERSION_4, v4([0, 0, 0, 0]));
        let conversation = Sample::udp(FAMILY_INTERNET).port(50000, 53);

        let Some(Door::Network(row)) = door_of(&door.bytes) else {
            panic!("a door")
        };
        assert_eq!((row.protocol, row.port), (Protocol::Udp, 5353));
        assert_eq!(door_of(&conversation.bytes), Some(Door::Ignored));
    }

    #[test]
    fn a_udp_socket_bound_to_no_port_yet_is_not_a_door() {
        let unbound = Sample::udp(FAMILY_INTERNET)
            .port(0, 0)
            .address(ONLY_VERSION_4, v4([0, 0, 0, 0]));

        assert_eq!(
            door_of(&unbound.bytes),
            Some(Door::Ignored),
            "a socket that was opened and never bound receives nothing, and Linux does not list \
             it either"
        );
    }

    #[test]
    fn a_unix_socket_accepting_connections_is_a_door_under_its_path() {
        let sample = Sample::unix(1, true, "/var/run/mDNSResponder").owned(0, 7);

        assert_eq!(
            door_of(&sample.bytes),
            Some(Door::Unix(UnixSocketRow {
                kind: UnixKind::Stream,
                name: "/var/run/mDNSResponder".into(),
                is_abstract: false,
                inode: 7,
            }))
        );
    }

    #[test]
    fn a_unix_socket_that_connected_somewhere_is_not_a_second_copy_of_the_door() {
        let client = Sample::unix(1, false, "");

        assert_eq!(door_of(&client.bytes), Some(Door::Ignored));
    }

    #[test]
    fn an_accepting_unix_socket_with_no_name_is_counted_and_not_named() {
        let sample = Sample::unix(1, true, "").owned(0, 9);

        assert_eq!(door_of(&sample.bytes), Some(Door::UnnamedUnix(9)));
    }

    #[test]
    fn a_datagram_socket_bound_to_a_path_is_a_door_like_on_linux() {
        let sample = Sample::unix(2, false, "/var/run/syslog");

        let Some(Door::Unix(row)) = door_of(&sample.bytes) else {
            panic!("a door")
        };
        assert_eq!(row.kind, UnixKind::Datagram);
    }

    #[test]
    fn a_path_that_fills_its_whole_field_is_read_to_the_end_of_the_field() {
        let long = format!("/{}", "a".repeat(103));
        let sample = Sample::unix(1, true, &long);

        let Some(Door::Unix(row)) = door_of(&sample.bytes) else {
            panic!("a door")
        };
        assert_eq!(row.name, long);
    }

    #[test]
    fn a_record_shorter_than_the_kernel_writes_is_refused() {
        assert_eq!(door_of(&[0u8; 100]), None);
    }

    #[test]
    fn a_socket_of_another_kind_is_not_a_door() {
        let sample = Sample::socket(4, 27, 3, 0);

        assert_eq!(door_of(&sample.bytes), Some(Door::Ignored));
    }
}
