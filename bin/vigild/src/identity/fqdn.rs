const HOSTS: &str = "/etc/hosts";

pub fn read(hostname: &str) -> Option<String> {
    let hosts = std::fs::read_to_string(HOSTS).unwrap_or_default();
    fqdn(hostname, &hosts)
}

pub fn fqdn(hostname: &str, hosts: &str) -> Option<String> {
    if acceptable(hostname) {
        return Some(hostname.to_string());
    }

    hosts
        .lines()
        .map(|line| line.split('#').next().unwrap_or_default())
        .map(|line| line.split_whitespace().skip(1).collect::<Vec<&str>>())
        .find(|names| names.iter().any(|name| name.eq_ignore_ascii_case(hostname)))
        .and_then(|names| names.first().map(|first| first.to_string()))
        .filter(|first| acceptable(first))
}

fn acceptable(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    name.contains('.')
        && !name.starts_with('.')
        && !name.ends_with('.')
        && !lowered.ends_with(".local")
        && !lowered.starts_with("localhost")
        && !lowered.ends_with(".localhost")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hostname_with_a_dot_is_its_own_fqdn() {
        assert_eq!(
            fqdn("web-1.example.com", "").as_deref(),
            Some("web-1.example.com")
        );
    }

    #[test]
    fn the_fqdn_is_the_canonical_name_of_the_hosts_line_naming_this_host() {
        let hosts = "127.0.0.1 localhost\n127.0.1.1 web-1.example.com web-1\n";

        assert_eq!(fqdn("web-1", hosts).as_deref(), Some("web-1.example.com"));
    }

    #[test]
    fn a_name_ending_in_local_is_not_an_fqdn() {
        assert_eq!(fqdn("studio.local", ""), None);
        assert_eq!(
            fqdn("studio", "192.168.1.5 studio.local studio\n"),
            None,
            "a name only the link it sits on resolves is no name for a receiver elsewhere"
        );
    }

    #[test]
    fn localhost_is_never_an_fqdn() {
        let hosts = "127.0.0.1 localhost.localdomain localhost web-1\n";

        assert_eq!(fqdn("web-1", hosts), None);
        assert_eq!(fqdn("localhost.localdomain", ""), None);
    }

    #[test]
    fn a_commented_hosts_line_is_not_read() {
        let hosts = "# 127.0.1.1 web-1.example.com web-1\n127.0.1.1 web-1\n";

        assert_eq!(fqdn("web-1", hosts), None);
        assert_eq!(
            fqdn(
                "web-1",
                "127.0.1.1 web-1.example.com web-1 # the old name\n"
            )
            .as_deref(),
            Some("web-1.example.com"),
            "a comment after the names is not one of them"
        );
    }

    #[test]
    fn no_fqdn_is_sent_rather_than_a_guess() {
        assert_eq!(fqdn("web-1", ""), None);
        assert_eq!(fqdn("web-1", "10.0.0.2 db-1.example.com db-1\n"), None);
    }

    #[test]
    fn the_fqdn_is_decided_from_the_hostname_and_the_hosts_file_alone() {
        assert_eq!(
            fqdn("web-1", "127.0.1.1 web-1.example.com web-1\n"),
            fqdn("web-1", "127.0.1.1 web-1.example.com web-1\n"),
            "the same two texts give the same answer: no resolver and no network is asked"
        );
    }
}
