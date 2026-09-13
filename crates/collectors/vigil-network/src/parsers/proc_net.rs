use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Tcp6,
    Udp,
    Udp6,
}

impl Protocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Protocol::Tcp => "tcp",
            Protocol::Tcp6 => "tcp6",
            Protocol::Udp => "udp",
            Protocol::Udp6 => "udp6",
        }
    }

    fn is_tcp(self) -> bool {
        matches!(self, Protocol::Tcp | Protocol::Tcp6)
    }

    fn is_v6(self) -> bool {
        matches!(self, Protocol::Tcp6 | Protocol::Udp6)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketRow {
    pub protocol: Protocol,
    pub address: String,
    pub port: u16,
    pub uid: u32,
    pub inode: u64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct NetTable {
    pub listening: Vec<SocketRow>,
    pub unparsed: usize,
}

const TCP_LISTEN: &str = "0A";

pub fn parse_net_table(text: &str, protocol: Protocol) -> NetTable {
    let mut table = NetTable::default();

    for line in text.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        match parse_row(line, protocol) {
            Some(Some(row)) => table.listening.push(row),
            Some(None) => {}
            None => table.unparsed += 1,
        }
    }

    table
}

fn parse_row(line: &str, protocol: Protocol) -> Option<Option<SocketRow>> {
    let mut fields = line.split_whitespace();
    let _slot = fields.next()?;
    let local = fields.next()?;
    let remote = fields.next()?;
    let state = fields.next()?;
    let _queues = fields.next()?;
    let _timer = fields.next()?;
    let _retransmits = fields.next()?;
    let uid = fields.next()?;
    let _timeout = fields.next()?;
    let inode = fields.next()?;

    let (address, port) = parse_endpoint(local, protocol.is_v6())?;
    let (_, remote_port) = parse_endpoint(remote, protocol.is_v6())?;

    let listening = if protocol.is_tcp() {
        state.eq_ignore_ascii_case(TCP_LISTEN)
    } else {
        remote_port == 0
    };
    if !listening {
        return Some(None);
    }

    Some(Some(SocketRow {
        protocol,
        address,
        port,
        uid: uid.parse().ok()?,
        inode: inode.parse().ok()?,
    }))
}

fn parse_endpoint(field: &str, v6: bool) -> Option<(String, u16)> {
    let (address, port) = field.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;

    let address = if v6 {
        if address.len() != 32 {
            return None;
        }
        let mut octets = [0u8; 16];
        for (index, word) in address.as_bytes().chunks(8).enumerate() {
            let word = std::str::from_utf8(word).ok()?;
            let word = u32::from_str_radix(word, 16).ok()?;
            octets[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Ipv6Addr::from(octets).to_string()
    } else {
        if address.len() != 8 {
            return None;
        }
        let word = u32::from_str_radix(address, 16).ok()?;
        Ipv4Addr::from(word.to_le_bytes()).to_string()
    };

    Some((address, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TCP: &str = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 20481 1 0000000000000000 100 0 0 10 0
   1: 0100007F:1538 00000000:0000 0A 00000000:00000000 00:00000000 00000000   106        0 23901 1 0000000000000000 100 0 0 10 0
   2: 0100007F:8AE2 0100007F:CFDA 01 00000000:00000000 00:00000000 00000000  1000        0 55120 1 0000000000000000 20 4 30 10 -1
";

    const TCP6: &str = "\
  sl  local_address                         remote_address                        st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000000000000000000000000000:0050 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 24011 1 0000000000000000 100 0 0 10 0
   1: 0000000000000000FFFF00000100007F:0BB8 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000    33        0 24012 1 0000000000000000 100 0 0 10 0
";

    const UDP: &str = "\
   sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops
  1234: 00000000:0044 00000000:0000 07 00000000:00000000 00:00000000 00000000     0        0 19334 2 0000000000000000 0
  2345: 0100007F:0035 0200007F:0035 01 00000000:00000000 00:00000000 00000000     0        0 19335 2 0000000000000000 0
";

    #[test]
    fn keeps_the_listening_sockets_and_drops_the_conversations() {
        let table = parse_net_table(TCP, Protocol::Tcp);

        assert_eq!(table.unparsed, 0);
        assert_eq!(
            table.listening.len(),
            2,
            "the established socket is not a door"
        );
        assert_eq!(
            table.listening[0],
            SocketRow {
                protocol: Protocol::Tcp,
                address: "0.0.0.0".into(),
                port: 22,
                uid: 0,
                inode: 20481,
            }
        );
        assert_eq!(table.listening[1].address, "127.0.0.1");
        assert_eq!(table.listening[1].port, 5432);
        assert_eq!(table.listening[1].uid, 106);
    }

    #[test]
    fn reads_the_address_word_in_the_hosts_byte_order_not_left_to_right() {
        let table = parse_net_table(TCP, Protocol::Tcp);
        assert_eq!(table.listening[1].address, "127.0.0.1");
    }

    #[test]
    fn understands_the_v6_table_including_its_ipv4_mapped_half() {
        let table = parse_net_table(TCP6, Protocol::Tcp6);

        assert_eq!(table.unparsed, 0);
        assert_eq!(table.listening.len(), 2);
        assert_eq!(table.listening[0].address, "::");
        assert_eq!(table.listening[0].port, 80);
        assert_eq!(table.listening[1].address, "::ffff:127.0.0.1");
        assert_eq!(table.listening[1].port, 3000);
        assert_eq!(table.listening[1].uid, 33);
    }

    #[test]
    fn a_udp_socket_with_a_peer_is_a_conversation_not_a_door() {
        let table = parse_net_table(UDP, Protocol::Udp);

        assert_eq!(table.listening.len(), 1);
        assert_eq!(table.listening[0].port, 68);
        assert_eq!(table.listening[0].inode, 19334);
    }

    #[test]
    fn a_line_we_do_not_understand_is_counted_rather_than_silently_skipped() {
        let text = "  sl  local_address\n   0: nonsense\n   1: 00000000:0016 00000000:0000 0A 0 0 0 0 0 0 77 1\n";

        let table = parse_net_table(text, Protocol::Tcp);

        assert_eq!(table.listening.len(), 1);
        assert_eq!(
            table.unparsed, 1,
            "a kernel whose columns we cannot read must degrade the collector, not shrink it"
        );
    }
}
