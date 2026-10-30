pub fn matches(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let (mut at, mut from) = (0, 0);
    let mut star: Option<(usize, usize)> = None;

    while from < text.len() {
        match pattern.get(at) {
            Some(b'*') => {
                star = Some((at, from));
                at += 1;
            }
            Some(&literal) if literal == text[from] => {
                at += 1;
                from += 1;
            }
            _ => match star {
                Some((star_at, star_from)) => {
                    at = star_at + 1;
                    from = star_from + 1;
                    star = Some((star_at, star_from + 1));
                }
                None => return false,
            },
        }
    }

    pattern[at..].iter().all(|&byte| byte == b'*')
}

pub fn matches_everything(pattern: &str) -> bool {
    pattern.bytes().all(|byte| byte == b'*')
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn a_star_in_the_middle_stands_for_the_address_and_nothing_else_is_loosened() {
        assert!(matches(
            "port.listen|udp|*:51820",
            "port.listen|udp|0.0.0.0:51820"
        ));
        assert!(!matches(
            "port.listen|udp|*:51820",
            "port.listen|udp|0.0.0.0:51821"
        ));
        assert!(!matches(
            "port.listen|udp|*:51820",
            "port.listen|tcp|0.0.0.0:51820"
        ));
    }

    #[test]
    fn a_pattern_covers_the_whole_text_rather_than_a_piece_of_it() {
        assert!(matches(
            "port.listen|udp|*",
            "port.listen|udp|0.0.0.0:41234"
        ));
        assert!(!matches("udp", "port.listen|udp|0.0.0.0:41234"));
        assert!(matches("*|udp|*", "port.listen|udp|0.0.0.0:41234"));
    }

    #[test]
    fn a_star_crosses_slashes_and_bars_because_a_key_is_not_a_path() {
        assert!(matches("/opt/*/netbird", "/opt/netbird/bin/netbird"));
        assert!(!matches("/opt/*/netbird", "/usr/bin/netbird"));
        assert!(matches("*", ""));
        assert!(matches("a*", "a"));
    }

    #[test]
    fn a_pattern_without_a_star_is_the_text_itself_and_case_counts() {
        assert!(matches("netbird", "netbird"));
        assert!(!matches("NetBird", "netbird"));
        assert!(!matches("netbird", "netbird-ui"));
        assert!(!matches("", "netbird"));
    }

    #[test]
    fn a_pattern_of_stars_alone_matches_everything_and_is_named_as_such() {
        assert!(matches_everything("*"));
        assert!(matches_everything("***"));
        assert!(matches_everything(""));
        assert!(!matches_everything("*netbird*"));
    }

    #[test]
    fn the_worst_pattern_an_operator_can_write_is_still_cheap_on_a_long_key() {
        let key = "a".repeat(4096);
        let started = Instant::now();

        assert!(!matches("*a*a*a*b", &key));
        assert!(started.elapsed() < Duration::from_millis(5));
    }
}
