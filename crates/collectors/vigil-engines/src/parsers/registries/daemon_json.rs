use serde_json::Value;

use crate::types::{CONFIGURED, MIRROR, Registry, gathered};

const INSECURE: &str = "insecure-registries";

const MIRRORS: &str = "registry-mirrors";

pub fn parse_daemon_json(text: &str, from: &str) -> Result<Vec<Registry>, String> {
    let said = match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(said)) => said,
        Ok(_) => return Err(format!("{from} is not a mapping of keys to values")),
        Err(error) => return Err(format!("{from}: {error}")),
    };

    let mut found = Vec::new();
    for written in listed(said.get(INSECURE)) {
        found.push(Registry {
            host: Registry::host_of(&written),
            insecure: true,
            role: CONFIGURED,
            from: from.to_string(),
        });
    }
    for written in listed(said.get(MIRRORS)) {
        found.push(Registry {
            host: Registry::host_of(&written),
            insecure: Registry::over_plain_http(&written),
            role: MIRROR,
            from: from.to_string(),
        });
    }

    Ok(gathered(found))
}

fn listed(said: Option<&Value>) -> Vec<String> {
    match said {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|one| one.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT: &str = "/etc/docker/daemon.json";

    #[test]
    fn a_registry_this_host_is_allowed_to_reach_without_tls_is_read_out_of_the_daemon_file() {
        let read = parse_daemon_json(
            r#"{"insecure-registries": ["registry.local:5000"], "log-driver": "json-file"}"#,
            AT,
        )
        .expect("reads");

        assert_eq!(read.len(), 1);
        assert_eq!(read[0].host, "registry.local:5000");
        assert!(read[0].insecure);
        assert_eq!(read[0].from, AT);
    }

    #[test]
    fn a_mirror_over_https_is_a_registry_this_host_uses_and_not_one_without_tls() {
        let read = parse_daemon_json(
            r#"{"registry-mirrors": ["https://mirror.example.com"]}"#,
            AT,
        )
        .expect("reads");

        assert_eq!(read[0].host, "mirror.example.com");
        assert!(!read[0].insecure);
        assert_eq!(read[0].role, MIRROR);
    }

    #[test]
    fn a_mirror_named_over_plain_http_is_a_registry_without_tls_even_where_nobody_said_insecure() {
        let read = parse_daemon_json(r#"{"registry-mirrors": ["http://mirror.local"]}"#, AT)
            .expect("reads");

        assert!(
            read[0].insecure,
            "a mirror reached over http is an image this host will pull from anybody who \
             answers first, and the key it is written under does not change that"
        );
    }

    #[test]
    fn a_daemon_file_that_is_not_json_is_a_named_refusal_and_not_a_host_with_no_registry() {
        let refusal = parse_daemon_json("{ this is not json", AT).expect_err("must not be read");

        assert!(refusal.contains(AT), "{refusal}");
        assert!(
            parse_daemon_json("[]", AT).is_err(),
            "a file this collector cannot read must reach the health line: a host whose \
             daemon.json nobody could parse is not a host that allows no insecure registry"
        );
    }
}
