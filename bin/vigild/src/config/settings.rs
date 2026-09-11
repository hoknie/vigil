use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::Suppression;

const WHEN_NOTHING_SAYS_OTHERWISE: u32 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub state_dir: String,
    pub socket_path: String,
    pub retention_days: u32,
    pub interval_seconds: Option<u32>,
    pub schedule: BTreeMap<String, u32>,
    pub collectors: Option<Vec<String>>,
    pub reporters: Vec<Receiver>,
    pub suppressions: Vec<Suppression>,
    pub record_launch_arguments: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            state_dir: "/var/lib/vigil".into(),
            socket_path: "/run/vigil/vigil.sock".into(),
            retention_days: 90,
            interval_seconds: None,
            schedule: BTreeMap::new(),
            collectors: None,
            reporters: Vec::new(),
            suppressions: Vec::new(),
            record_launch_arguments: false,
        }
    }
}

impl Config {
    pub fn every_seconds(&self, collector: &str) -> u32 {
        if let Some(named) = self.schedule.get(collector) {
            return *named;
        }
        if let Some(one_for_all) = self.interval_seconds {
            return one_for_all;
        }
        vigil_collect::every_seconds_of_collector(collector).unwrap_or(WHEN_NOTHING_SAYS_OTHERWISE)
    }

    pub fn every_seconds_by_default(&self) -> u32 {
        self.interval_seconds.unwrap_or(WHEN_NOTHING_SAYS_OTHERWISE)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Receiver {
    Ndjson {
        path: String,
    },
    Syslog {
        facility: String,
    },
    Webhook {
        url: String,
        token_file: Option<String>,
    },
    HostFindings {
        url: String,
        token_file: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_names_one_interval_and_nothing_else_puts_every_collector_on_it() {
        let config = Config {
            interval_seconds: Some(10),
            ..Config::default()
        };

        for name in vigil_collect::collector_names() {
            assert_eq!(config.every_seconds(name), 10, "{name}");
        }
    }

    #[test]
    fn a_file_that_names_neither_gives_each_collector_the_period_its_own_crate_declares() {
        let config = Config::default();

        assert_eq!(config.every_seconds("launches"), 15);
        assert_eq!(config.every_seconds("ports"), 30);
        assert_eq!(config.every_seconds("persistence"), 300);
    }

    #[test]
    fn a_named_period_wins_over_the_interval_and_leaves_the_others_on_it() {
        let mut config = Config {
            interval_seconds: Some(10),
            ..Config::default()
        };
        config.schedule.insert("persistence".into(), 600);

        assert_eq!(config.every_seconds("persistence"), 600);
        assert_eq!(config.every_seconds("ports"), 10);
    }

    #[test]
    fn a_collector_this_build_has_never_heard_of_is_still_given_a_period() {
        let config = Config::default();

        assert_eq!(config.every_seconds("from-a-later-version"), 30);
    }
}
