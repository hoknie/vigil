use crate::types::{CONFIGURED, MIRROR, Registry, SEARCH, gathered};

const SEARCHED: &str = "unqualified-search-registries";

const REGISTRY: &str = "[[registry]]";

const MIRRORED: &str = "[[registry.mirror]]";

#[derive(Default)]
struct Block {
    location: Option<String>,
    insecure: bool,
    mirror: bool,
}

pub fn parse_registries_conf(text: &str, from: &str) -> Vec<Registry> {
    let mut found: Vec<Registry> = Vec::new();
    let mut block: Option<Block> = None;

    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') {
            close(&mut block, &mut found, from);
            block = match line {
                REGISTRY => Some(Block::default()),
                MIRRORED => Some(Block {
                    mirror: true,
                    ..Block::default()
                }),
                _ => None,
            };
            continue;
        }

        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        let (name, value) = (name.trim(), value.trim());

        match (name, block.as_mut()) {
            (SEARCHED, _) => {
                for written in listed(value) {
                    found.push(Registry {
                        host: Registry::host_of(&written),
                        insecure: false,
                        role: SEARCH,
                        from: from.to_string(),
                    });
                }
            }
            ("location", Some(block)) => block.location = Some(unquoted(value)),
            ("insecure", Some(block)) => block.insecure = value.starts_with("true"),
            _ => continue,
        }
    }

    close(&mut block, &mut found, from);
    gathered(found)
}

fn close(block: &mut Option<Block>, found: &mut Vec<Registry>, from: &str) {
    let Some(block) = block.take() else {
        return;
    };
    let Some(location) = block.location else {
        return;
    };

    found.push(Registry {
        host: Registry::host_of(&location),
        insecure: block.insecure || Registry::over_plain_http(&location),
        role: match block.mirror {
            true => MIRROR,
            false => CONFIGURED,
        },
        from: from.to_string(),
    });
}

fn listed(value: &str) -> Vec<String> {
    value
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(unquoted)
        .filter(|one| !one.is_empty())
        .collect()
}

fn unquoted(value: &str) -> String {
    value.trim().trim_matches(['"', '\'']).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT: &str = "/etc/containers/registries.conf";

    const WRITTEN: &str = r#"
unqualified-search-registries = ["docker.io", "quay.io"]

[[registry]]
prefix = "registry.local:5000"
location = "registry.local:5000"
insecure = true

[[registry]]
location = "registry.example.com"
insecure = false

[[registry.mirror]]
location = "mirror.example.com"
"#;

    #[test]
    fn a_registry_marked_insecure_in_the_file_is_read_as_a_registry_without_tls() {
        let read = parse_registries_conf(WRITTEN, AT);

        let one = read
            .iter()
            .find(|registry| registry.host == "registry.local:5000")
            .expect("the insecure one");
        assert!(one.insecure);
        assert_eq!(one.role, CONFIGURED);
        assert_eq!(one.from, AT);
    }

    #[test]
    fn the_registries_an_unqualified_name_is_looked_up_in_are_rows_of_their_own() {
        let read = parse_registries_conf(WRITTEN, AT);

        let searched: Vec<&str> = read
            .iter()
            .filter(|registry| registry.role == SEARCH)
            .map(|registry| registry.host.as_str())
            .collect();

        assert_eq!(
            searched,
            vec!["docker.io", "quay.io"],
            "an image named `nginx` is pulled from whichever of these answers first, so the \
             list changing is a change in where this host's images come from"
        );
    }

    #[test]
    fn a_mirror_declared_under_a_registry_is_a_row_beside_it_and_not_a_row_replacing_it() {
        let read = parse_registries_conf(WRITTEN, AT);

        assert!(read.iter().any(|one| one.host == "registry.example.com"));
        let mirror = read
            .iter()
            .find(|one| one.host == "mirror.example.com")
            .expect("the mirror");
        assert_eq!(mirror.role, MIRROR);
        assert!(!mirror.insecure);
    }

    #[test]
    fn a_line_commented_out_is_a_line_this_host_does_not_act_on() {
        let read = parse_registries_conf(
            "[[registry]]\nlocation = \"kept.example\"\n# insecure = true\n",
            AT,
        );

        assert_eq!(read.len(), 1);
        assert!(
            !read[0].insecure,
            "a commented insecure line reported as an insecure registry is a finding about \
             something nobody turned on"
        );
    }

    #[test]
    fn a_file_with_nothing_in_it_is_a_host_with_no_registry_written_down() {
        assert!(parse_registries_conf("", AT).is_empty());
        assert!(parse_registries_conf("# nothing but a comment\n", AT).is_empty());
    }
}
