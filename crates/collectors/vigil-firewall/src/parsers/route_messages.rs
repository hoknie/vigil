pub const ROUTE_MESSAGE_HEADER: usize = 92;

const LENGTH_AT: usize = 0;

const INDEX_AT: usize = 4;

const ADDRESSES_AT: usize = 12;

const DESTINATION: i32 = 0x1;

const FAMILY_AT: usize = 1;

const INET: u8 = 2;

const INET6: u8 = 30;

const INET_ADDRESS: std::ops::Range<usize> = 4..8;

const INET6_ADDRESS: std::ops::Range<usize> = 8..24;

pub fn parse_default_routes(dumped: &[u8]) -> Vec<u16> {
    let mut indices: Vec<u16> = Vec::new();
    let mut at = 0usize;

    while at + ROUTE_MESSAGE_HEADER <= dumped.len() {
        let length = usize::from(u16::from_ne_bytes([
            dumped[at + LENGTH_AT],
            dumped[at + LENGTH_AT + 1],
        ]));
        if length < ROUTE_MESSAGE_HEADER || at + length > dumped.len() {
            break;
        }
        let message = &dumped[at..at + length];
        at += length;

        let addresses = i32::from_ne_bytes([
            message[ADDRESSES_AT],
            message[ADDRESSES_AT + 1],
            message[ADDRESSES_AT + 2],
            message[ADDRESSES_AT + 3],
        ]);
        if addresses & DESTINATION == 0 {
            continue;
        }
        let destination = &message[ROUTE_MESSAGE_HEADER..];
        if !is_everywhere(destination) {
            continue;
        }

        let index = u16::from_ne_bytes([message[INDEX_AT], message[INDEX_AT + 1]]);
        if index != 0 && !indices.contains(&index) {
            indices.push(index);
        }
    }

    indices
}

fn is_everywhere(address: &[u8]) -> bool {
    let (Some(length), Some(family)) = (address.first(), address.get(FAMILY_AT)) else {
        return false;
    };
    let held = &address[..usize::from(*length).min(address.len())];
    let zero = |range: std::ops::Range<usize>| {
        held.get(range)
            .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0))
    };
    match *family {
        INET => zero(INET_ADDRESS),
        INET6 => zero(INET6_ADDRESS),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(index: u16, destination: &[u8]) -> Vec<u8> {
        let mut message = vec![0u8; ROUTE_MESSAGE_HEADER];
        let length = (ROUTE_MESSAGE_HEADER + destination.len()) as u16;
        message[..2].copy_from_slice(&length.to_ne_bytes());
        message[INDEX_AT..INDEX_AT + 2].copy_from_slice(&index.to_ne_bytes());
        message[ADDRESSES_AT..ADDRESSES_AT + 4].copy_from_slice(&(DESTINATION | 0x2).to_ne_bytes());
        message.extend_from_slice(destination);
        message
    }

    fn inet(address: [u8; 4]) -> Vec<u8> {
        let mut socket = vec![16, INET, 0, 0];
        socket.extend_from_slice(&address);
        socket.extend_from_slice(&[0; 8]);
        socket
    }

    #[test]
    fn the_interface_of_the_route_to_everywhere_is_the_way_out() {
        let mut dumped = message(4, &inet([10, 0, 0, 0]));
        dumped.extend(message(6, &inet([0, 0, 0, 0])));

        assert_eq!(parse_default_routes(&dumped), vec![6]);
    }

    #[test]
    fn a_route_to_everywhere_over_ipv6_is_one_too() {
        let mut socket = vec![28, INET6];
        socket.extend_from_slice(&[0; 26]);

        assert_eq!(parse_default_routes(&message(7, &socket)), vec![7]);
    }

    #[test]
    fn a_message_that_claims_more_than_the_dump_holds_ends_the_walk_rather_than_reading_past_it() {
        let mut dumped = message(6, &inet([0, 0, 0, 0]));
        dumped[..2].copy_from_slice(&4096u16.to_ne_bytes());

        assert!(parse_default_routes(&dumped).is_empty());
        assert!(parse_default_routes(&[1, 2, 3]).is_empty());
    }
}
