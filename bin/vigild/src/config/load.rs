use std::fmt;
use std::path::Path;

use vigil_module::Settings;
use vigil_report::SyslogFacility;

use super::{Config, Receiver, apart, collectors, former, split};
use crate::helpers::rfc3339;

pub fn load(path: &str) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError {
        path: path.to_string(),
        cause: e.to_string(),
    })?;
    let mut read = split::read(&text, &keys()).map_err(|cause| ConfigError {
        path: path.to_string(),
        cause,
    })?;
    match collectors::pointed(&read) {
        Some(at) => collectors::forbidden(&read, &at),
        None => former::renamed_settings(&mut read.of_the_modules),
    }
    .map_err(|cause| ConfigError {
        path: path.to_string(),
        cause,
    })?;
    let mut config: Config =
        serde_yaml::from_value(read.of_the_daemon).map_err(|e| ConfigError {
            path: path.to_string(),
            cause: e.to_string(),
        })?;
    config.of_the_modules = read.of_the_modules;

    read_collectors(path, &mut config).map_err(|cause| ConfigError {
        path: path.to_string(),
        cause,
    })?;

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
                cause: match config.apart.block(module.name()) {
                    Some(block) => format!("{}: {key}: {cause}", block.file.display()),
                    None => format!("{key}: {cause}"),
                },
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

    read_apart(path, &mut config).map_err(|cause| ConfigError {
        path: path.to_string(),
        cause,
    })?;

    Ok(config)
}

fn read_collectors(path: &str, config: &mut Config) -> Result<(), String> {
    match apart::place(
        Path::new(path),
        vigil_config::COLLECTORS_PATH,
        config.collectors_path.as_deref(),
    )? {
        Some(at) => collectors::taken(config, at),
        None => former::renamed(config),
    }
}

fn read_apart(path: &str, config: &mut Config) -> Result<(), String> {
    let configuration = Path::new(path);

    if let Some(at) = apart::place(
        configuration,
        vigil_config::SUPPRESSIONS_PATH,
        config.suppressions_path.as_deref(),
    )? {
        config.apart.suppressions = apart::suppressions(&at)?;
        config.apart.suppressions_at = Some(at);
    }

    if let Some(at) = apart::place(
        configuration,
        REPORTERS_PATH,
        config.reporters_path.as_deref(),
    )? {
        config.apart.reporters = apart::reporters(&at)?;
        for file in &config.apart.reporters {
            config.reporters.extend(file.receivers.iter().cloned());
        }
        config.apart.reporters_at = Some(at);
    }

    Ok(())
}

const REPORTERS_PATH: &str = "reporters_path";

fn keys() -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = crate::modules::modules()
        .iter()
        .filter_map(|module| module.settings_key())
        .collect();
    keys.extend(former::settings_keys());
    keys
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
