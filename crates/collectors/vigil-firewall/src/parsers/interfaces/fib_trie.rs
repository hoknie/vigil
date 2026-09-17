pub const FIB_TRIE: &str = "/proc/net/fib_trie";

const LOCAL_TABLE: &str = "Local:";

const ANOTHER_TABLE: char = ':';

const LEAF: &str = "|--";

const NODE: &str = "+--";

const THE_HOST_ITSELF: &str = "/32 host LOCAL";

pub fn parse_fib_trie(text: &str) -> Vec<u32> {
    let mut found: Vec<u32> = Vec::new();
    let mut here: Option<u32> = None;
    let mut local = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.ends_with(ANOTHER_TABLE) {
            local = trimmed == LOCAL_TABLE;
            here = None;
            continue;
        }
        if !local {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(LEAF).or(trimmed.strip_prefix(NODE)) {
            here = address_of(rest.split('/').next().unwrap_or_default().trim());
            continue;
        }
        if trimmed.starts_with(THE_HOST_ITSELF)
            && let Some(address) = here.take()
        {
            found.push(address);
        }
    }

    found.sort_unstable();
    found.dedup();
    found
}

fn address_of(written: &str) -> Option<u32> {
    let mut octets = [0u8; 4];
    let mut counted = 0usize;

    for (at, part) in written.split('.').enumerate() {
        if at >= octets.len() {
            return None;
        }
        octets[at] = part.parse::<u8>().ok()?;
        counted += 1;
    }

    match counted == octets.len() {
        true => Some(u32::from_be_bytes(octets)),
        false => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::route::written_out;
    use super::*;

    const A_HOST_WITH_TWO_ADDRESSES: &str = "\
Main:
  +-- 0.0.0.0/0 3 0 5
     |-- 0.0.0.0
        /0 universe UNICAST
     +-- 192.168.1.0/24 2 0 2
        |-- 192.168.1.0
           /24 link UNICAST
Local:
  +-- 0.0.0.0/2 1 0 2
     +-- 127.0.0.0/8 2 0 2
        |-- 127.0.0.1
           /32 host LOCAL
     +-- 192.168.1.0/24 2 0 2
        |-- 192.168.1.23
           /32 host LOCAL
        |-- 192.168.1.255
           /32 link BROADCAST
     |-- 172.17.0.1
        /32 host LOCAL
";

    #[test]
    fn the_addresses_this_host_answers_on_are_the_ones_the_local_table_calls_its_own() {
        let found: Vec<String> = parse_fib_trie(A_HOST_WITH_TWO_ADDRESSES)
            .into_iter()
            .map(written_out)
            .collect();

        assert_eq!(found, vec!["127.0.0.1", "172.17.0.1", "192.168.1.23"]);
    }

    #[test]
    fn a_broadcast_address_is_not_an_address_this_host_answers_on() {
        let found = parse_fib_trie(A_HOST_WITH_TWO_ADDRESSES);

        assert!(
            !found.contains(&u32::from_be_bytes([192, 168, 1, 255])),
            "the line under an address is what says what the address is, and a reader shown \
             the broadcast of a network as this host's address looks for a host that is not \
             there"
        );
    }

    #[test]
    fn a_network_the_main_table_routes_is_not_an_address_of_this_host() {
        let found = parse_fib_trie(A_HOST_WITH_TWO_ADDRESSES);

        assert!(!found.contains(&u32::from_be_bytes([192, 168, 1, 0])));
        assert!(!found.contains(&0));
    }

    #[test]
    fn a_kernel_that_published_no_trie_at_all_leaves_this_host_with_no_address_and_no_refusal() {
        assert!(parse_fib_trie("").is_empty());
        assert!(parse_fib_trie("Local:\n  +-- nonsense\n").is_empty());
    }
}
