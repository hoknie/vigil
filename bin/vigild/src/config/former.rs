use std::collections::BTreeMap;

use serde_json::Value;

use super::Config;

pub const FORMER_NAMES: &[(&str, &str)] = &[("ports", "network")];

pub const FORMER_SETTINGS_KEYS: &[(&str, &str)] = &[("containers", "containers-engines")];

pub fn current_name(name: &str) -> &str {
    FORMER_NAMES
        .iter()
        .find(|(was, now)| {
            *was == name && !crate::modules::is_known(was) && crate::modules::is_known(now)
        })
        .map(|(_, now)| *now)
        .unwrap_or(name)
}

pub fn former_name(name: &str) -> Option<&'static str> {
    FORMER_NAMES
        .iter()
        .find(|(was, now)| *now == name && current_name(was) == name)
        .map(|(was, _)| *was)
}

pub fn settings_keys() -> Vec<&'static str> {
    FORMER_SETTINGS_KEYS
        .iter()
        .filter(|(was, now)| is_a_settings_key(now) && !is_a_settings_key(was))
        .map(|(was, _)| *was)
        .collect()
}

pub fn current_settings_key(key: &str) -> &str {
    FORMER_SETTINGS_KEYS
        .iter()
        .find(|(was, _)| *was == key && settings_keys().contains(was))
        .map(|(_, now)| *now)
        .unwrap_or(key)
}

pub fn renamed(config: &mut Config) -> Result<(), String> {
    if let Some(named) = &mut config.collectors {
        for name in named.iter_mut() {
            *name = current_name(name).to_string();
        }
    }

    let mut schedule = BTreeMap::new();
    for (name, every_seconds) in std::mem::take(&mut config.schedule) {
        let now = current_name(&name).to_string();
        if schedule.insert(now.clone(), every_seconds).is_some() {
            return Err(format!(
                "schedule {name:?}: {name} is the former name of {now}, and both are written; \
                 keep {now}"
            ));
        }
    }
    config.schedule = schedule;

    Ok(())
}

pub fn renamed_settings(of_the_modules: &mut BTreeMap<String, Value>) -> Result<(), String> {
    for former in settings_keys() {
        let Some(block) = of_the_modules.remove(former) else {
            continue;
        };
        let now = current_settings_key(former);
        if of_the_modules.contains_key(now) {
            return Err(format!(
                "{former}: the block of the container engines is written twice, as {former} \
                 and as {now}; keep {now}"
            ));
        }
        of_the_modules.insert(now.to_string(), block);
    }
    Ok(())
}

fn is_a_settings_key(key: &str) -> bool {
    crate::modules::modules()
        .iter()
        .any(|module| module.settings_key() == Some(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_former_name_is_a_name_this_build_has_under_another_word() {
        for (was, now) in FORMER_NAMES {
            assert!(
                crate::modules::is_known(now),
                "{was} is read as {now}, and this build has no {now}"
            );
            assert!(
                !crate::modules::is_known(was),
                "{was} is a collector of its own again, and reading it as {now} would \
                 switch on the wrong one"
            );
            assert_eq!(current_name(was), *now);
            assert_eq!(former_name(now), Some(*was));
        }
    }

    #[test]
    fn a_name_that_was_never_changed_is_its_own_current_name() {
        assert_eq!(current_name("users"), "users");
        assert_eq!(former_name("users"), None);
        assert_eq!(current_name("from-a-later-version"), "from-a-later-version");
    }

    #[test]
    fn every_former_settings_key_is_read_as_a_key_a_module_of_this_build_declares() {
        assert_eq!(
            settings_keys().len(),
            FORMER_SETTINGS_KEYS.len(),
            "a former key that is no longer read as anything is a block an upgraded host \
             believes configures its engines"
        );
    }
}
