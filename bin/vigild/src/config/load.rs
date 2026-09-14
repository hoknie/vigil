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
