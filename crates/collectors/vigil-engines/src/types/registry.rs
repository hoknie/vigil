#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Registry {
    pub host: String,
    pub insecure: bool,
    pub role: &'static str,
    pub from: String,
}

pub const CONFIGURED: &str = "configured";

pub const MIRROR: &str = "mirror";

pub const SEARCH: &str = "search";

impl Registry {
    pub fn host_of(written: &str) -> String {
        let without_scheme = written
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://");

        without_scheme
            .split('/')
            .next()
            .unwrap_or(without_scheme)
            .trim_end_matches('/')
            .to_string()
    }

    pub fn over_plain_http(written: &str) -> bool {
        written.trim().starts_with("http://")
    }
}

pub fn gathered(mut found: Vec<Registry>) -> Vec<Registry> {
    found.sort();

    let mut kept: Vec<Registry> = Vec::new();
    for one in found {
        if one.host.is_empty() {
            continue;
        }
        match kept.iter_mut().find(|already| already.host == one.host) {
            Some(already) => already.insecure = already.insecure || one.insecure,
            None => kept.push(one),
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_registry_written_as_a_url_is_kept_by_the_host_it_names() {
        assert_eq!(
            Registry::host_of("https://mirror.example.com/"),
            "mirror.example.com"
        );
        assert_eq!(
            Registry::host_of("http://registry.local:5000"),
            "registry.local:5000"
        );
        assert_eq!(
            Registry::host_of("registry.local:5000"),
            "registry.local:5000"
        );
    }

    #[test]
    fn a_mirror_reached_over_plain_http_is_a_registry_without_tls_however_it_is_spelled() {
        assert!(Registry::over_plain_http("http://mirror.example.com"));
        assert!(!Registry::over_plain_http("https://mirror.example.com"));
    }

    #[test]
    fn a_host_named_twice_is_one_row_and_keeps_the_worse_of_the_two_answers() {
        let read = gathered(vec![
            Registry {
                host: "registry.local:5000".into(),
                insecure: false,
                role: MIRROR,
                from: "/etc/docker/daemon.json".into(),
            },
            Registry {
                host: "registry.local:5000".into(),
                insecure: true,
                role: CONFIGURED,
                from: "/etc/docker/daemon.json".into(),
            },
        ]);

        assert_eq!(read.len(), 1);
        assert!(
            read[0].insecure,
            "a registry named once as a mirror and once as insecure is an insecure registry, \
             and a row saying otherwise is a row that hides the thing worth reporting"
        );
    }
}
