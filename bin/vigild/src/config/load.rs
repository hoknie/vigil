use std::fmt;

use vigil_module::Settings;
use vigil_report::SyslogFacility;

use super::{Config, Receiver, split};
use crate::helpers::rfc3339;

pub fn load(path: &str) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError {
        path: path.to_string(),
        cause: e.to_string(),
    })?;
    let read = split::read(&text, &keys()).map_err(|cause| ConfigError {
        path: path.to_string(),
        cause,
    })?;
    let mut config: Config =
        serde_yaml::from_value(read.of_the_daemon).map_err(|e| ConfigError {
            path: path.to_string(),
            cause: e.to_string(),
        })?;
    config.of_the_modules = read.of_the_modules;

    for (index, suppression) in config.suppressions.iter().enumerate() {
        suppression.validate().map_err(|cause| ConfigError {
            path: path.to_string(),
            cause: format!("suppression #{}: {cause}", index + 1),
        })?;
    }

    if let Some(enabled) = &config.collectors {
        let mut already: Vec<&str> = Vec::new();
        for (index, name) in enabled.iter().enumerate() {
            if !crate::modules::is_known(name) {
                return Err(ConfigError {
                    path: path.to_string(),
                    cause: format!(
                        "collector #{}: unknown collector {name:?}; known: {}",
                        index + 1,
                        crate::modules::names().join(", ")
                    ),
                });
            }
            if already.contains(&name.as_str()) {
                return Err(ConfigError {
                    path: path.to_string(),
                    cause: format!("collector #{}: {name:?} is named twice", index + 1),
                });
            }
            already.push(name);
        }
    }

    super::schedule::check(&config).map_err(|cause| ConfigError {
        path: path.to_string(),
        cause,
    })?;

    for module in crate::modules::modules() {
        let Some(key) = module.settings_key() else {
            continue;
        };
        module
            .check(&Settings::of(rfc3339::now, key, config.of_the_module(key)))
            .map_err(|cause| ConfigError {
                path: path.to_string(),
                cause: format!("{key}: {cause}"),
            })?;
    }

    for (index, receiver) in config.reporters.iter().enumerate() {
        if let Receiver::Syslog { facility } = receiver {
            SyslogFacility::parse(facility).map_err(|cause| ConfigError {
                path: path.to_string(),
                cause: format!("reporter #{}: {cause}", index + 1),
            })?;
        }
    }

    Ok(config)
}

fn keys() -> Vec<&'static str> {
    crate::modules::modules()
        .iter()
        .filter_map(|module| module.settings_key())
        .collect()
}

#[derive(Debug)]
pub struct ConfigError {
    pub path: String,
    pub cause: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "configuration {}: {}", self.path, self.cause)
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_is_a_working_configuration() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("empty.yaml");
        std::fs::write(&path, "{}\n").expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.state_dir, "/var/lib/vigil");
        assert!(
            config.reporters.is_empty(),
            "no receiver configured is a supported configuration"
        );
    }

    #[test]
    fn a_suppression_that_covers_nothing_is_refused_at_start_up() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("empty-suppression.yaml");
        std::fs::write(&path, "suppressions:\n  - reason: the staging api\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("suppression #1"), "{error}");
        assert!(error.cause.contains("silences nothing"), "{error}");
    }

    #[test]
    fn a_suppression_that_names_an_object_and_a_reason_is_accepted() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("suppression.yaml");
        std::fs::write(
            &path,
            "suppressions:\n  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"\n    reason: the staging api, expected\n",
        )
        .expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.suppressions.len(), 1);
        assert_eq!(config.suppressions[0].reason, "the staging api, expected");
    }

    #[test]
    fn a_misspelled_syslog_facility_is_refused_at_start_up_rather_than_sent_to_nowhere() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("facility.yaml");
        std::fs::write(
            &path,
            "reporters:\n  - kind: syslog\n    facility: lokal0\n",
        )
        .expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("reporter #1"), "{error}");
        assert!(error.cause.contains("lokal0"), "{error}");
        assert!(
            error.cause.contains("local0"),
            "the message says what was allowed: {error}"
        );
    }

    #[test]
    fn a_facility_every_syslog_daemon_knows_is_accepted() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("facility-ok.yaml");
        std::fs::write(
            &path,
            "reporters:\n  - kind: syslog\n    facility: local4\n",
        )
        .expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.reporters.len(), 1);
    }

    #[test]
    fn the_file_this_product_ships_is_a_file_this_product_reads() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config/vigil.example.yaml"
        );

        let config = load(path).expect("the shipped example must load");

        assert_eq!(
            config.collectors,
            Some(
                crate::modules::names()
                    .iter()
                    .map(|name| name.to_string())
                    .collect()
            ),
            "the shipped example names every collector this product has, in the product's order"
        );
    }

    #[test]
    fn a_configuration_with_no_collectors_key_watches_everything() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("no-collectors.yaml");
        std::fs::write(&path, "interval_seconds: 30\n").expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.collectors, None);
    }

    #[test]
    fn an_empty_collector_list_is_a_deliberate_nothing_and_not_the_same_as_no_key() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("no-collector.yaml");
        std::fs::write(&path, "collectors: []\n").expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.collectors, Some(Vec::new()));
    }

    #[test]
    fn a_misspelled_collector_is_refused_by_name_and_told_which_ones_exist() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("collector-typo.yaml");
        std::fs::write(&path, "collectors:\n  - ports\n  - proccesses\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("proccesses"), "{error}");
        assert!(error.cause.contains("processes"), "{error}");
        assert!(error.cause.contains("collector #2"), "{error}");
    }

    #[test]
    fn a_collector_named_twice_is_refused_rather_than_watched_twice() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("collector-twice.yaml");
        std::fs::write(&path, "collectors: [ports, users, ports]\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("named twice"), "{error}");
    }

    #[test]
    fn a_value_a_module_refuses_stops_the_daemon_at_the_door_and_not_at_the_first_reading() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("module-value.yaml");
        std::fs::write(&path, "resources:\n  disk_free_percent: 101\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("resources"), "{error}");
        assert!(error.cause.contains("disk_free_percent"), "{error}");
    }

    #[test]
    fn a_misspelled_key_inside_a_module_block_is_refused_by_the_module_that_owns_it() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("module-typo.yaml");
        std::fs::write(&path, "files:\n  celing_bytes: 2048\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be ignored");

        assert!(error.cause.contains("celing_bytes"), "{error}");
        assert!(
            error.cause.contains("files"),
            "the message says whose key it was: {error}"
        );
    }

    #[test]
    fn a_module_block_the_file_names_reaches_the_module_and_not_the_daemons_own_fields() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("module-block.yaml");
        std::fs::write(
            &path,
            "retention_days: 5\nlaunches:\n  record_arguments: true\n",
        )
        .expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.retention_days, 5);
        assert_eq!(
            config.of_the_module("launches"),
            serde_json::json!({"record_arguments": true}),
            "the daemon carries the block whole and hands it over: what is in it is between \
             the module and the operator"
        );
    }

    #[test]
    fn a_misspelled_field_is_refused_by_name_instead_of_ignored() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("typo.yaml");
        std::fs::write(&path, "retention_dayz: 5\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be ignored");

        assert!(error.cause.contains("retention_dayz"), "{error}");
        assert!(error.to_string().contains("typo.yaml"), "{error}");
    }
}
