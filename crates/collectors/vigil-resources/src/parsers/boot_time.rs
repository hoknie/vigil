const SECONDS_BYTES: usize = 8;

pub fn parse_boot_time(bytes: &[u8]) -> Option<i64> {
    let seconds = i64::from_ne_bytes(bytes.get(..SECONDS_BYTES)?.try_into().ok()?);

    (seconds > 0).then_some(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moment(seconds: i64, microseconds: i32) -> Vec<u8> {
        let mut bytes = seconds.to_ne_bytes().to_vec();
        bytes.extend(microseconds.to_ne_bytes());
        bytes.extend([0u8; 4]);
        bytes
    }

    #[test]
    fn the_moment_a_mac_booted_is_read_to_the_second_it_began_in() {
        assert_eq!(
            parse_boot_time(&moment(1_789_538_653, 153_768)),
            Some(1_789_538_653),
            "the reading carries whole seconds, as the Linux one does, so that the two compare \
             under one tolerance"
        );
    }

    #[test]
    fn a_moment_cut_short_or_at_the_start_of_time_is_no_moment() {
        assert_eq!(parse_boot_time(&[1, 2, 3]), None);
        assert_eq!(
            parse_boot_time(&moment(0, 0)),
            None,
            "a kernel that has not set its boot time did not boot in 1970"
        );
    }
}
