pub const NET_DEV: &str = "/proc/net/dev";

const HEADER_LINES: usize = 2;

const RECEIVED_PACKETS: usize = 1;

const RECEIVED_DROPPED: usize = 3;

const SENT_BYTES: usize = 8;

const SENT_PACKETS: usize = 9;

const SENT_DROPPED: usize = 11;

const COLUMNS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Traffic {
    pub packets_in: u64,
    pub bytes_in: u64,
    pub dropped_in: u64,
    pub packets_out: u64,
    pub bytes_out: u64,
    pub dropped_out: u64,
}

pub fn parse_net_dev(text: &str) -> Vec<(String, Traffic)> {
    let mut counted: Vec<(String, Traffic)> = text
        .lines()
        .skip(HEADER_LINES)
        .filter_map(counters)
        .collect();

    counted.sort_by(|left, right| left.0.cmp(&right.0));
    counted.dedup_by(|left, right| left.0 == right.0);
    counted
}

fn counters(line: &str) -> Option<(String, Traffic)> {
    let (name, rest) = line.split_once(':')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let numbers: Vec<u64> = rest
        .split_ascii_whitespace()
        .map(|word| word.parse::<u64>().unwrap_or(0))
        .collect();
    if numbers.len() < COLUMNS {
        return None;
    }

    Some((
        name.to_string(),
        Traffic {
            packets_in: numbers[RECEIVED_PACKETS],
            bytes_in: numbers[0],
            dropped_in: numbers[RECEIVED_DROPPED],
            packets_out: numbers[SENT_PACKETS],
            bytes_out: numbers[SENT_BYTES],
            dropped_out: numbers[SENT_DROPPED],
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_INTERFACES: &str = "Inter-|   Receive                                                |  Transmit\n \
         face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n    \
         lo: 5140552   41220    0    0    0     0          0         0  5140552   41220    0    0    0     0       0          0\n  \
         eth0: 91200311  318841    0    7    0     0          0     51204 12084411  201773    0    2    0     0       0          0\n";

    #[test]
    fn an_interface_is_read_with_what_came_in_and_what_went_out_of_it() {
        let counted = parse_net_dev(TWO_INTERFACES);

        assert_eq!(counted.len(), 2);
        assert_eq!(counted[0].0, "eth0");
        assert_eq!(counted[0].1.packets_in, 318_841);
        assert_eq!(counted[0].1.packets_out, 201_773);
        assert_eq!(counted[0].1.bytes_in, 91_200_311);
        assert_eq!(counted[0].1.bytes_out, 12_084_411);
        assert_eq!(counted[0].1.dropped_in, 7);
        assert_eq!(counted[0].1.dropped_out, 2);
    }

    #[test]
    fn a_kernel_writing_a_shape_this_build_does_not_know_yields_no_interface_of_its_own() {
        assert!(parse_net_dev("").is_empty());
        assert!(
            parse_net_dev("Inter-|\n face |\n  eth0: 1 2 3\n").is_empty(),
            "a line with fewer numbers than the kernel writes is a line this build cannot \
             place, and placing it anyway attributes one interface's traffic to another"
        );
    }

    #[test]
    fn a_counter_the_kernel_wrote_in_a_shape_that_is_not_a_number_is_read_as_none_of_it() {
        let odd = "a\nb\n  eth0: bytes packets 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n";

        let counted = parse_net_dev(odd);

        assert_eq!(counted.len(), 1);
        assert_eq!(counted[0].1.packets_in, 0);
        assert_eq!(counted[0].1.bytes_in, 0);
    }

    #[test]
    fn an_interface_named_twice_is_counted_once() {
        let twice = format!("a\nb\n{}{}", line("eth0", 5), line("eth0", 9));

        let counted = parse_net_dev(&twice);

        assert_eq!(counted.len(), 1);
    }

    fn line(name: &str, packets: u64) -> String {
        format!("  {name}: 1 {packets} 0 0 0 0 0 0 1 {packets} 0 0 0 0 0 0\n")
    }
}
