use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use vigil_config::Suppression;

use super::apart::Apart;

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
    pub collectors_path: Option<String>,
    pub reporters: Vec<Receiver>,
    pub reporters_path: Option<String>,
    pub suppressions: Vec<Suppression>,
    pub suppressions_path: Option<String>,
    pub killing: Killing,
    pub accounts: Accounts,
    pub units: Units,
    #[serde(skip)]
    pub of_the_modules: BTreeMap<String, Value>,
    #[serde(skip)]
    pub apart: Apart,
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
            collectors_path: None,
            reporters: Vec::new(),
            reporters_path: None,
            suppressions: Vec::new(),
            suppressions_path: None,
            killing: Killing::default(),
            accounts: Accounts::default(),
            units: Units::default(),
            of_the_modules: BTreeMap::new(),
            apart: Apart::default(),
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
        crate::modules::every_seconds_of(collector).unwrap_or(WHEN_NOTHING_SAYS_OTHERWISE)
    }

    pub fn of_the_module(&self, key: &str) -> Value {
        self.of_the_modules.get(key).cloned().unwrap_or(Value::Null)
    }

    pub fn every_suppression(&self) -> Vec<Suppression> {
        let mut every = self.suppressions.clone();
        for source in &self.apart.suppressions {
            every.extend(source.suppressions.iter().cloned());
        }
        every
    }

    pub fn every_seconds_by_default(&self) -> u32 {
        self.interval_seconds.unwrap_or(WHEN_NOTHING_SAYS_OTHERWISE)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Killing {
    pub from_the_console: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Accounts {
    pub from_the_console: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Units {
    pub from_the_console: bool,
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
    fn a_configuration_nobody_edited_lets_nothing_on_this_host_be_killed() {
        assert!(
            !Config::default().killing.from_the_console,
            "the console can ask the agent to close a socket, and that is the one thing this \
             product does to a host it did not create. It is off on a host whose operator has \
             not written the word down, in the file they review and version, beside \
             suppressions. A default of true would mean every install is one keystroke from a \
             stopped service."
        );
    }

    #[test]
    fn a_configuration_nobody_edited_lets_no_account_on_this_host_be_changed() {
        assert!(
            !Config::default().accounts.from_the_console,
            "the console can ask the agent to delete an account, hand out sudo or put a key in \
             authorized_keys. That is off on a host whose operator has not written the word \
             down, and it is a key of its own: a host where a program may be stopped has not \
             thereby agreed that sudo may be granted"
        );
    }

    #[test]
    fn a_configuration_nobody_edited_stops_and_disables_nothing_this_host_starts_by_itself() {
        assert!(
            !Config::default().units.from_the_console,
            "the console can ask the agent to stop, disable or mask a unit and to comment a \
             line out of a crontab. That is off on a host whose operator has not written the \
             word down, and it is a key of its own: a host where a process may be signalled \
             has not thereby agreed that a service may be disabled until somebody notices"
        );
    }

    #[test]
    fn a_file_that_switches_killing_on_has_switched_neither_accounts_nor_units_on() {
        let config: Config =
            serde_yaml::from_str("killing:\n  from_the_console: true\n").expect("parses");

        assert!(config.killing.from_the_console);
        assert!(!config.accounts.from_the_console);
        assert!(
            !config.units.from_the_console,
            "three verbs, three keys: one word in this file must never switch on a second \
             thing the agent does to the host"
        );
    }

    #[test]
    fn a_file_that_names_one_interval_and_nothing_else_puts_every_collector_on_it() {
        let config = Config {
            interval_seconds: Some(10),
            ..Config::default()
        };

        for name in crate::modules::names() {
            assert_eq!(config.every_seconds(name), 10, "{name}");
        }
    }

    #[test]
    fn a_file_that_names_neither_gives_each_collector_the_period_its_own_crate_declares() {
        let config = Config::default();

        assert_eq!(config.every_seconds("launches"), 15);
        assert_eq!(config.every_seconds("network"), 30);
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
        assert_eq!(config.every_seconds("network"), 10);
    }

    #[test]
    fn a_collector_this_build_has_never_heard_of_is_still_given_a_period() {
        let config = Config::default();

        assert_eq!(config.every_seconds("from-a-later-version"), 30);
    }

    #[test]
    fn a_module_whose_key_the_file_never_names_is_handed_nothing_and_falls_back_on_its_own() {
        let config = Config::default();

        assert_eq!(
            config.of_the_module("files"),
            Value::Null,
            "the daemon holds no default of a module's own: a key nobody wrote is a module \
             left to its own values, not a module handed the daemon's idea of them"
        );
    }
}
