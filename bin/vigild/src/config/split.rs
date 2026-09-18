use std::collections::BTreeMap;

use serde_json::Value;
use serde_yaml::{Mapping, Value as Yaml};

pub struct Read {
    pub of_the_daemon: Yaml,
    pub of_the_modules: BTreeMap<String, Value>,
}

pub fn read(text: &str, keys: &[&'static str]) -> Result<Read, String> {
    let document: Yaml = serde_yaml::from_str(text).map_err(|error| error.to_string())?;

    let Yaml::Mapping(written) = document else {
        if document.is_null() {
            return Ok(Read {
                of_the_daemon: Yaml::Mapping(Mapping::new()),
                of_the_modules: BTreeMap::new(),
            });
        }
        return Err("the file is not a mapping of keys to values".to_string());
    };

    let mut of_the_daemon = Mapping::new();
    let mut of_the_modules = BTreeMap::new();

    for (key, value) in written {
        match key.as_str().filter(|named| keys.contains(named)) {
            Some(named) => {
                let said =
                    serde_json::to_value(&value).map_err(|error| format!("{named}: {error}"))?;
                of_the_modules.insert(named.to_string(), said);
            }
            None => {
                of_the_daemon.insert(key, value);
            }
        }
    }

    Ok(Read {
        of_the_daemon: Yaml::Mapping(of_the_daemon),
        of_the_modules,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: &[&str] = &["launches", "files"];

    fn read_of(text: &str) -> Read {
        read(text, KEYS).expect("the sample parses")
    }

    #[test]
    fn a_key_a_module_declares_is_taken_out_and_everything_else_is_the_daemons() {
        let read = read_of(
            "retention_days: 5\nlaunches:\n  record_arguments: true\nsocket_path: /run/x.sock\n",
        );

        assert_eq!(
            read.of_the_modules["launches"],
            serde_json::json!({"record_arguments": true})
        );
        let daemon = read.of_the_daemon.as_mapping().expect("a mapping");
        assert_eq!(daemon.len(), 2, "{daemon:?}");
        assert!(daemon.contains_key(Yaml::from("retention_days")));
        assert!(
            !daemon.contains_key(Yaml::from("launches")),
            "a module's key reaching the daemon's own struct is a field the daemon would have \
             to grow for every module there is"
        );
    }

    #[test]
    fn a_key_nobody_declares_stays_with_the_daemon_so_that_it_is_the_one_to_refuse_it() {
        let read = read_of("retention_dayz: 5\n");

        assert!(
            read.of_the_daemon
                .as_mapping()
                .expect("a mapping")
                .contains_key(Yaml::from("retention_dayz")),
            "a misspelled key handed to nobody would be a key silently ignored"
        );
        assert!(read.of_the_modules.is_empty());
    }

    #[test]
    fn a_file_with_nothing_in_it_reads_as_a_file_that_names_nothing() {
        for text in ["", "\n", "{}\n", "---\n"] {
            let read = read(text, KEYS).unwrap_or_else(|why| panic!("{text:?}: {why}"));

            assert!(read.of_the_modules.is_empty(), "{text:?}");
            assert_eq!(
                read.of_the_daemon.as_mapping().map(Mapping::len),
                Some(0),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_file_that_is_a_list_or_a_word_is_refused_rather_than_read_as_empty() {
        for text in ["- network\n", "network\n", "42\n"] {
            assert!(read(text, KEYS).is_err(), "{text:?}");
        }
    }

    #[test]
    fn a_module_key_reaches_the_module_as_the_shape_it_was_written_in() {
        let read = read_of("files:\n  paths:\n    - /etc/hosts\n  ceiling_bytes: 2048\n");

        assert_eq!(
            read.of_the_modules["files"],
            serde_json::json!({"paths": ["/etc/hosts"], "ceiling_bytes": 2048})
        );
    }
}
