#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnixKind {
    Stream,
    Datagram,
    SeqPacket,
}

impl UnixKind {
    pub fn as_str(self) -> &'static str {
        match self {
            UnixKind::Stream => "stream",
            UnixKind::Datagram => "dgram",
            UnixKind::SeqPacket => "seqpacket",
        }
    }

    fn from_hex(field: &str) -> Option<Self> {
        match u16::from_str_radix(field, 16).ok()? {
            1 => Some(UnixKind::Stream),
            2 => Some(UnixKind::Datagram),
            5 => Some(UnixKind::SeqPacket),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnixSocketRow {
    pub kind: UnixKind,
    pub name: String,
    pub is_abstract: bool,
    pub inode: u64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct UnixTable {
    pub listening: Vec<UnixSocketRow>,
    pub unnamed: usize,
    pub unparsed: usize,
}

const SO_ACCEPTCON: u32 = 0x0001_0000;

pub fn parse_unix_table(text: &str) -> UnixTable {
    let mut table = UnixTable::default();

    for line in text.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        match parse_row(line) {
            Some(Row::Listening(row)) => table.listening.push(row),
            Some(Row::Unnamed) => table.unnamed += 1,
            Some(Row::Ignored) => {}
            None => table.unparsed += 1,
        }
    }

    table
}

enum Row {
    Listening(UnixSocketRow),
    Unnamed,
    Ignored,
}

fn parse_row(line: &str) -> Option<Row> {
    let mut rest = line;
    let mut head: [&str; 7] = [""; 7];
    for field in head.iter_mut() {
        rest = rest.trim_start();
        if rest.is_empty() {
            return None;
        }
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        *field = &rest[..end];
        rest = &rest[end..];
    }

    let flags = u32::from_str_radix(head[3], 16).ok()?;
    let kind = UnixKind::from_hex(head[4])?;
    let inode: u64 = head[6].parse().ok()?;
    let name = rest.trim_start();

    let accepting = flags & SO_ACCEPTCON != 0;
    let is_abstract = name.starts_with('@');

    let door = match kind {
        UnixKind::Stream | UnixKind::SeqPacket => accepting,
        UnixKind::Datagram => name.starts_with('/'),
    };
    if !door {
        return Some(Row::Ignored);
    }
    if name.is_empty() {
        return Some(Row::Unnamed);
    }

    Some(Row::Listening(UnixSocketRow {
        kind,
        name: name.to_string(),
        is_abstract,
        inode,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNIX: &str = "\
Num       RefCount Protocol Flags    Type St Inode Path
00000000a1cd5adc: 00000003 00000000 00000000 0001 03 85517572 /var/run/docker.raw.sock
00000000398d9a3b: 00000003 00000000 00000000 0001 03 85517541
00000000b7090f1e: 00000002 00000000 00010000 0001 01    49 /run/init.sock
00000000c98fb272: 00000002 00000000 00010000 0001 01 40794315 /run/containerd/s/338de3703dd8
000000002e50fed6: 00000002 00000000 00010000 0001 01    68 /var/run/docker.raw.sock
0000000048077b7b: 00000002 00000000 00010000 0001 01    74 @/tmp/.X11-unix/X0
00000000e34be5e7: 00000002 00000000 00000000 0002 01    75 /run/systemd/journal/dev-log
0000000008a2ea51: 00000002 00000000 00000000 0002 01    76 @0000f
00000000960cde56: 00000002 00000000 00010000 0001 01    77
";

    #[test]
    fn keeps_the_sockets_something_is_serving_on_and_drops_the_conversations() {
        let table = parse_unix_table(UNIX);

        assert_eq!(table.unparsed, 0);
        let names: Vec<&str> = table
            .listening
            .iter()
            .map(|row| row.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "/run/init.sock",
                "/run/containerd/s/338de3703dd8",
                "/var/run/docker.raw.sock",
                "@/tmp/.X11-unix/X0",
                "/run/systemd/journal/dev-log",
            ]
        );
    }

    #[test]
    fn a_client_connected_to_a_socket_does_not_become_a_second_copy_of_it() {
        let table = parse_unix_table(UNIX);

        let docker = table
            .listening
            .iter()
            .filter(|row| row.name == "/var/run/docker.raw.sock")
            .count();
        assert_eq!(docker, 1, "one door, however many people walked through it");
    }

    #[test]
    fn a_datagram_socket_with_a_path_is_a_door_and_an_autobound_one_is_not() {
        let table = parse_unix_table(UNIX);

        assert!(
            table
                .listening
                .iter()
                .any(|row| row.name == "/run/systemd/journal/dev-log"
                    && row.kind == UnixKind::Datagram),
            "nothing calls listen() on the syslog socket, and it is still where logs go"
        );
        assert!(
            !table.listening.iter().any(|row| row.name == "@0000f"),
            "an autobound abstract name belongs to a client and changes on every start"
        );
    }

    #[test]
    fn an_abstract_listener_is_kept_and_marked_as_one() {
        let table = parse_unix_table(UNIX);

        let x11 = table
            .listening
            .iter()
            .find(|row| row.name == "@/tmp/.X11-unix/X0")
            .expect("kept");
        assert!(
            x11.is_abstract,
            "there is no file to go and look at, and a reader has to be told"
        );
    }

    #[test]
    fn a_listening_socket_whose_name_is_gone_is_counted_rather_than_given_an_unstable_key() {
        let table = parse_unix_table(UNIX);

        assert_eq!(table.unnamed, 1);
        assert!(
            table.listening.iter().all(|row| !row.name.is_empty()),
            "the only key left would be the inode, and that is new on every restart"
        );
    }

    #[test]
    fn a_name_with_a_space_in_it_survives_whole() {
        let text = "Num RefCount Protocol Flags Type St Inode Path\n\
             0000000000000000: 00000002 00000000 00010000 0001 01 4242 /run/my service.sock\n";

        let table = parse_unix_table(text);

        assert_eq!(table.listening[0].name, "/run/my service.sock");
    }

    #[test]
    fn a_line_we_do_not_understand_is_counted_rather_than_silently_skipped() {
        let text = "Num RefCount Protocol Flags Type St Inode Path\n\
             nonsense\n\
             0000000000000000: 00000002 00000000 00010000 0001 01 4242 /run/ok.sock\n";

        let table = parse_unix_table(text);

        assert_eq!(table.listening.len(), 1);
        assert_eq!(
            table.unparsed, 1,
            "a kernel whose columns we cannot read must degrade the collector, not shrink it"
        );
    }
}
