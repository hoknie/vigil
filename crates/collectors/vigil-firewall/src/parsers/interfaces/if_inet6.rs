pub const IF_INET6: &str = "/proc/net/if_inet6";

const HEXADECIMAL: u32 = 16;

const DIGITS: usize = 32;

const GROUPS: usize = 8;

const FIELDS: usize = 6;

const PREFIX_AT: usize = 2;

const NAME_AT: usize = 5;

const SHORTEST_RUN: usize = 2;

pub fn parse_if_inet6(text: &str) -> Vec<(String, String)> {
    text.lines().filter_map(address_of).collect()
}

fn address_of(line: &str) -> Option<(String, String)> {
    let fields: Vec<&str> = line.split_ascii_whitespace().collect();
    if fields.len() < FIELDS {
        return None;
    }

    let written = written_out(fields[0])?;
    let prefix = u32::from_str_radix(fields[PREFIX_AT], HEXADECIMAL).ok()?;

    Some((fields[NAME_AT].to_string(), format!("{written}/{prefix}")))
}

fn written_out(digits: &str) -> Option<String> {
    if digits.len() != DIGITS {
        return None;
    }

    let mut words: Vec<u16> = Vec::with_capacity(GROUPS);
    for group in 0..GROUPS {
        let at = group * 4;
        words.push(u16::from_str_radix(digits.get(at..at + 4)?, HEXADECIMAL).ok()?);
    }

    Some(shortened(&words))
}

fn shortened(words: &[u16]) -> String {
    let (from, length) = longest_run_of_zeroes(words);
    if length < SHORTEST_RUN {
        return joined(words);
    }

    format!(
        "{}::{}",
        joined(&words[..from]),
        joined(&words[from + length..])
    )
}

fn longest_run_of_zeroes(words: &[u16]) -> (usize, usize) {
    let (mut from, mut length, mut start, mut run) = (0usize, 0usize, 0usize, 0usize);

    for (at, word) in words.iter().enumerate() {
        match word {
            0 => {
                if run == 0 {
                    start = at;
                }
                run += 1;
                if run > length {
                    from = start;
                    length = run;
                }
            }
            _ => run = 0,
        }
    }

    (from, length)
}

fn joined(words: &[u16]) -> String {
    words
        .iter()
        .map(|word| format!("{word:x}"))
        .collect::<Vec<String>>()
        .join(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOOPBACK_AND_A_LINK: &str = "\
00000000000000000000000000000001 01 80 10 80       lo
fe800000000000000a00271ffe8a3b41 02 40 20 80     eth0
fe80000000000000a00271ffe8a3b41 02 40 20 80     eth0
20010db8000000000000000000000042 02 40 00 80     eth0
";

    #[test]
    fn an_address_the_kernel_wrote_as_thirty_two_digits_is_shown_the_way_a_person_writes_it() {
        let addresses = parse_if_inet6(LOOPBACK_AND_A_LINK);

        assert_eq!(addresses[0], ("lo".to_string(), "::1/128".into()));
        assert_eq!(
            addresses[1],
            ("eth0".to_string(), "fe80::a00:271f:fe8a:3b41/64".into()),
            "a reader matches what the console shows against what ip -6 addr shows, and a row \
             of hexadecimal digits matches neither"
        );
        assert_eq!(addresses[2], ("eth0".to_string(), "2001:db8::42/64".into()));
    }

    #[test]
    fn a_line_whose_address_is_not_thirty_two_digits_is_left_out_rather_than_half_read() {
        let addresses = parse_if_inet6(LOOPBACK_AND_A_LINK);

        assert_eq!(
            addresses.len(),
            3,
            "half of an address is a different address, and an interface shown with one is an \
             interface a reader will look for on the wire and not find"
        );
    }

    #[test]
    fn an_address_with_one_zero_group_in_it_keeps_that_group_written_out() {
        let single = "20010db80000ffff000100020003000f 02 40 00 80     eth0\n";

        assert_eq!(
            parse_if_inet6(single)[0].1,
            "2001:db8:0:ffff:1:2:3:f/64",
            "one group replaced by a pair of colons saves nothing and is not how an address \
             is written"
        );
    }

    #[test]
    fn a_host_with_no_sixth_family_configured_yields_no_address_and_no_refusal() {
        assert!(parse_if_inet6("").is_empty());
        assert!(parse_if_inet6("something else entirely\n").is_empty());
    }
}
