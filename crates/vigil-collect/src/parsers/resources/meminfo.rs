const KIBIBYTE: u64 = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryFacts {
    pub total_bytes: u64,
    pub swap_total_bytes: Option<u64>,
}

pub fn parse_meminfo(text: &str) -> Option<MemoryFacts> {
    let mut total_bytes = None;
    let mut swap_total_bytes = None;

    for line in text.lines() {
        let Some((name, rest)) = line.split_once(':') else {
            continue;
        };
        match name {
            "MemTotal" => total_bytes = bytes(rest),
            "SwapTotal" => swap_total_bytes = bytes(rest),
            _ => {}
        }
    }

    Some(MemoryFacts {
        total_bytes: total_bytes?,
        swap_total_bytes,
    })
}

fn bytes(rest: &str) -> Option<u64> {
    let mut fields = rest.split_whitespace();
    let amount: u64 = fields.next()?.parse().ok()?;

    match fields.next() {
        Some("kB") => amount.checked_mul(KIBIBYTE),
        None => Some(amount),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM_A_HOST_WITH_SWAP: &str = "MemTotal:        8039152 kB\n\
         MemFree:          311104 kB\n\
         MemAvailable:    5120884 kB\n\
         Buffers:          182300 kB\n\
         SwapTotal:       1048572 kB\n\
         SwapFree:        1048572 kB\n";

    #[test]
    fn the_size_of_this_host_and_of_its_swap_are_read_in_bytes() {
        let facts = parse_meminfo(FROM_A_HOST_WITH_SWAP).expect("reads");

        assert_eq!(facts.total_bytes, 8_039_152 * 1024);
        assert_eq!(facts.swap_total_bytes, Some(1_048_572 * 1024));
    }

    #[test]
    fn the_numbers_that_move_between_two_readings_are_read_past_rather_than_recorded() {
        let later = FROM_A_HOST_WITH_SWAP
            .replace("311104", "294512")
            .replace("5120884", "4901220")
            .replace("182300", "179944");

        assert_eq!(
            parse_meminfo(FROM_A_HOST_WITH_SWAP),
            parse_meminfo(&later),
            "free memory moves between any two readings, so a reading carrying it differs from \
             the one before it every time and the differ has something to say on every tick"
        );
    }

    #[test]
    fn a_kernel_that_says_nothing_about_swap_is_not_a_host_with_no_swap() {
        let facts = parse_meminfo("MemTotal:        8039152 kB\n").expect("reads");

        assert_eq!(facts.swap_total_bytes, None);

        let none = parse_meminfo("MemTotal:        8039152 kB\nSwapTotal:             0 kB\n")
            .expect("reads");

        assert_eq!(none.swap_total_bytes, Some(0));
    }

    #[test]
    fn a_file_in_a_shape_we_do_not_know_is_not_a_host_with_no_memory() {
        assert_eq!(parse_meminfo(""), None);
        assert_eq!(parse_meminfo("MemTotal: plenty\n"), None);
        assert_eq!(parse_meminfo("MemTotal:        8039152 MB\n"), None);
        assert_eq!(parse_meminfo("MemFree:          311104 kB\n"), None);
    }
}
