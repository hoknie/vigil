use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde_yaml::Value as Yaml;
use vigil_config::{Block, COLLECTORS_PATH, blocks};

use super::settings::{Accounts, Killing, Units};
use super::split::Read;
use super::{Config, former};

pub const VERBS: &[(&str, &str)] = &[
    ("killing", "processes"),
    ("accounts", "users"),
    ("units", "persistence"),
];

const OF_THE_BLOCKS: &[&str] = &["collectors", "schedule", "interval_seconds"];

pub fn pointed(read: &Read) -> Option<String> {
    match read.of_the_daemon.get(COLLECTORS_PATH)? {
        Yaml::Null => None,
        Yaml::String(written) => Some(written.clone()),
        other => Some(format!("{other:?}")),
    }
}

pub fn forbidden(read: &Read, at: &str) -> Result<(), String> {
    let now_there = |key: &str, what: String| {
        Err(format!(
            "{key}: with {COLLECTORS_PATH} set, this key is not read from this file — it now \
             lives in {COLLECTORS_PATH} ({at}): {what}"
        ))
    };

    if let Some(key) = read.of_the_modules.keys().next() {
        let block = former::current_settings_key(key);
        return now_there(key, format!("move it into the block of {block} there"));
    }
    let Some(daemon) = read.of_the_daemon.as_mapping() else {
        return Ok(());
    };
    for key in daemon.keys().filter_map(Yaml::as_str) {
        if OF_THE_BLOCKS.contains(&key) {
            return now_there(
                key,
                "a collector runs when its block is there and does not say `enabled: false`, \
                 and reads at the `schedule:` of its block"
                    .to_string(),
            );
        }
        if let Some((verb, owner)) = VERBS.iter().find(|(verb, _)| *verb == key) {
            return now_there(verb, format!("it is written in the block of {owner} there"));
        }
        if crate::modules::is_known(key) {
            return now_there(key, format!("move it into the block of {key} there"));
        }
    }
    Ok(())
}

pub fn taken(config: &mut Config, at: PathBuf) -> Result<(), String> {
    let modules = crate::modules::modules();
    let mut on: Vec<String> = Vec::new();
    let mut kept = Vec::new();

    for mut block in blocks(&at)? {
        let file = block.file.display().to_string();
        let Some(module) = modules.iter().find(|module| module.name() == block.name) else {
            return Err(format!(
                "{file}: unknown collector {:?}; known: {}",
                block.name,
                crate::modules::names().join(", ")
            ));
        };

        for (verb, owner) in VERBS {
            let Some(value) = block.settings.remove(*verb) else {
                continue;
            };
            if block.name != *owner {
                return Err(format!(
                    "{file}: {}: `{verb}` is written in the block of {owner}, the collector whose \
                     readings it acts on, and nowhere else",
                    block.name
                ));
            }
            said(config, verb, value).map_err(|cause| format!("{file}: {owner}: {cause}"))?;
        }

        if !block.settings.is_empty() {
            let Some(key) = module.settings_key() else {
                return Err(format!(
                    "{file}: {} takes no settings but `enabled` and `schedule`, and its block \
                     holds {}",
                    block.name,
                    written_keys(&block)
                ));
            };
            let settings = serde_json::to_value(&block.settings)
                .map_err(|error| format!("{file}: {}: {error}", block.name))?;
            config.of_the_modules.insert(key.to_string(), settings);
        }

        if block.enabled {
            if let Some(every_seconds) = block.schedule {
                config.schedule.insert(block.name.clone(), every_seconds);
            }
            on.push(block.name.clone());
        }
        kept.push(block);
    }

    config.collectors = Some(
        crate::modules::names()
            .into_iter()
            .filter(|name| on.iter().any(|named| named == name))
            .map(str::to_string)
            .collect(),
    );
    config.apart.collectors = kept;
    config.apart.collectors_at = Some(at);
    Ok(())
}

fn said(config: &mut Config, verb: &str, value: Yaml) -> Result<(), String> {
    match verb {
        "killing" => config.killing = verb_of::<Killing>(verb, value)?,
        "accounts" => config.accounts = verb_of::<Accounts>(verb, value)?,
        "units" => config.units = verb_of::<Units>(verb, value)?,
        _ => return Err(format!("{verb} is not a verb of the console")),
    }
    Ok(())
}

fn verb_of<T: DeserializeOwned>(verb: &str, value: Yaml) -> Result<T, String> {
    serde_yaml::from_value(value).map_err(|error| format!("{verb}: {error}"))
}

fn written_keys(block: &Block) -> String {
    block
        .settings
        .keys()
        .map(|key| match key.as_str() {
            Some(named) => format!("`{named}`"),
            None => format!("{key:?}"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
