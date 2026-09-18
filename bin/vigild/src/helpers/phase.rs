const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0100_0000_01b3;
const SEPARATOR: u8 = b'|';

pub fn seconds(host_id: &str, collector: &str, every_seconds: u32) -> u32 {
    if every_seconds == 0 {
        return 0;
    }
    (fnv1a(host_id, collector) % u64::from(every_seconds)) as u32
}

fn fnv1a(host_id: &str, collector: &str) -> u64 {
    let mut hash = OFFSET_BASIS;
    for byte in host_id
        .as_bytes()
        .iter()
        .chain(std::iter::once(&SEPARATOR))
        .chain(collector.as_bytes())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    const A_HOST: &str = "1c9d8e7b4a5c6d0e";

    #[test]
    fn the_phase_of_a_host_is_the_same_after_a_restart() {
        assert_eq!(seconds(A_HOST, "network", 30), 12);
        assert_eq!(seconds(A_HOST, "processes", 30), 7);
        assert_eq!(seconds(A_HOST, "launches", 15), 9);
        assert_eq!(seconds(A_HOST, "users", 300), 108);
        assert_eq!(seconds(A_HOST, "persistence", 300), 175);
    }

    #[test]
    fn two_collectors_of_the_same_period_do_not_wake_up_in_the_same_second() {
        assert_ne!(
            seconds(A_HOST, "network", 30),
            seconds(A_HOST, "processes", 30)
        );
    }

    #[test]
    fn five_hundred_hosts_of_one_park_do_not_all_report_in_the_same_second() {
        let park: Vec<u32> = (0..500)
            .map(|number| seconds(&format!("host-{number:04}"), "network", 30))
            .collect();

        let mut spread = park.clone();
        spread.sort_unstable();
        spread.dedup();

        assert!(
            spread.len() >= 25,
            "500 identically configured hosts landed on {} second(s)",
            spread.len()
        );
        let busiest = spread
            .iter()
            .map(|second| park.iter().filter(|it| *it == second).count())
            .max()
            .unwrap_or(0);
        assert!(busiest < 60, "{busiest} of 500 hosts share one second");
    }

    #[test]
    fn a_phase_never_lands_outside_the_period_it_shifts() {
        for every_seconds in [1u32, 15, 30, 300, 3_600] {
            for collector in crate::modules::names() {
                assert!(seconds(A_HOST, collector, every_seconds) < every_seconds);
            }
        }
        assert_eq!(seconds(A_HOST, "network", 0), 0);
    }
}
