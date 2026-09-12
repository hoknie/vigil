pub fn parse_uptime_seconds(text: &str) -> Option<u64> {
    let first = text.split_whitespace().next()?;
    let whole = match first.split_once('.') {
        Some((seconds, _)) => seconds,
        None => first,
    };

    whole.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seconds_since_boot_are_read_and_the_idle_seconds_beside_them_are_not() {
        assert_eq!(
            parse_uptime_seconds("128142.56 1005819.02\n"),
            Some(128_142)
        );
        assert_eq!(parse_uptime_seconds("0.42 0.11\n"), Some(0));
    }

    #[test]
    fn a_file_in_a_shape_we_do_not_know_is_not_a_host_that_booted_now() {
        assert_eq!(parse_uptime_seconds(""), None);
        assert_eq!(parse_uptime_seconds("\n"), None);
        assert_eq!(parse_uptime_seconds("up 3 days"), None);
        assert_eq!(
            parse_uptime_seconds("-12.00 0.00"),
            None,
            "a host that has been up for less than no time would put the moment it booted in \
             the future, and every reading after it would disagree with the one before"
        );
    }

    #[test]
    fn a_kernel_that_writes_whole_seconds_is_read_the_same_as_one_that_writes_hundredths() {
        assert_eq!(parse_uptime_seconds("41\n"), Some(41));
        assert_eq!(parse_uptime_seconds("41.00 0.00\n"), Some(41));
    }
}
