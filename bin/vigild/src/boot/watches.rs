use vigil_collect::{Collector, Health};
use vigil_rules::RuleSet;

use crate::Config;
use crate::loops::Watch;

pub struct Family {
    pub collector: Box<dyn Collector>,
    pub rules: RuleSet,
}

#[cfg(target_os = "linux")]
pub fn families(config: &Config) -> Result<Vec<Family>, String> {
    use vigil_module::Settings;

    use crate::helpers::rfc3339;

    let mut families = Vec::new();
    for module in crate::modules::modules() {
        let settings = match module.settings_key() {
            Some(key) => Settings::of(rfc3339::now, key, config.of_the_module(key)),
            None => Settings::plain(rfc3339::now),
        };
        families.push(Family {
            collector: module.collector(&settings)?,
            rules: module.rules(&settings),
        });
    }

    Ok(families)
}

#[cfg(not(target_os = "linux"))]
pub fn families(config: &Config) -> Result<Vec<Family>, String> {
    let _ = config.of_the_module("files");
    Err(format!(
        "this build has no collector for {}: vigil reads a Linux /proc. Run it on Linux: `just docker` opens a container with the working copy mounted.",
        std::env::consts::OS
    ))
}

pub fn open(config: &Config) -> Result<(Vec<Watch>, Vec<String>), String> {
    let (enabled, switched_off) = split(families(config)?, config.collectors.as_deref());
    let watches = enabled
        .into_iter()
        .map(|family| Watch::new(family.collector, family.rules))
        .collect();

    Ok((watches, switched_off))
}

pub fn healths(watches: &[Watch]) -> Vec<(&'static str, Health)> {
    watches
        .iter()
        .map(|watch| (watch.name(), watch.health()))
        .collect()
}

pub fn split(families: Vec<Family>, wanted: Option<&[String]>) -> (Vec<Family>, Vec<String>) {
    let Some(wanted) = wanted else {
        return (families, Vec::new());
    };

    let mut on = Vec::new();
    let mut off = Vec::new();
    for family in families {
        match wanted.iter().any(|name| name == family.collector.name()) {
            true => on.push(family),
            false => off.push(family.collector.name().to_string()),
        }
    }
    (on, off)
}

#[cfg(test)]
mod tests {
    use vigil_collect::CollectError;
    use vigil_model::Snapshot;

    use super::*;

    struct Named(&'static str);

    impl Collector for Named {
        fn name(&self) -> &'static str {
            self.0
        }
        fn available(&self) -> Health {
            Health::Ok
        }
        fn collect(&self) -> Result<Snapshot, CollectError> {
            Err(CollectError::Absent("a test collector".into()))
        }
    }

    fn family(name: &'static str) -> Family {
        Family {
            collector: Box::new(Named(name)),
            rules: RuleSet::of(Vec::new()),
        }
    }

    fn all() -> Vec<Family> {
        vec![family("network"), family("users"), family("launches")]
    }

    fn names(families: &[Family]) -> Vec<&str> {
        families
            .iter()
            .map(|family| family.collector.name())
            .collect()
    }

    #[test]
    fn no_list_at_all_runs_everything_this_build_has() {
        let (on, off) = split(all(), None);

        assert_eq!(names(&on), vec!["network", "users", "launches"]);
        assert!(off.is_empty());
    }

    #[test]
    fn only_what_is_named_runs_and_the_rest_is_named_as_off() {
        let wanted = vec!["users".to_string()];

        let (on, off) = split(all(), Some(&wanted));

        assert_eq!(names(&on), vec!["users"]);
        assert_eq!(off, vec!["network".to_string(), "launches".to_string()]);
    }

    #[test]
    fn an_empty_list_is_a_host_nothing_is_watching_and_every_collector_is_named() {
        let wanted: Vec<String> = Vec::new();

        let (on, off) = split(all(), Some(&wanted));

        assert!(on.is_empty());
        assert_eq!(off.len(), 3);
    }

    #[test]
    fn the_order_is_the_products_and_not_the_files() {
        let wanted = vec!["launches".to_string(), "network".to_string()];

        let (on, _) = split(all(), Some(&wanted));

        assert_eq!(names(&on), vec!["network", "launches"]);
    }
}
