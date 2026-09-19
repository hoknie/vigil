use vigil_collect::{Collector, Health};
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;

use crate::Config;
use crate::adapters::Unported;
use crate::helpers::rfc3339;
use crate::loops::Watch;

pub struct Family {
    pub collector: Box<dyn Collector>,
    pub rules: RuleSet,
}

pub fn families(config: &Config) -> Result<Vec<Family>, String> {
    let mut families = Vec::new();
    for module in crate::modules::modules() {
        let settings = match module.settings_key() {
            Some(key) => Settings::of(rfc3339::now, key, config.of_the_module(key)),
            None => Settings::plain(rfc3339::now),
        };
        families.push(family(module.as_ref(), &settings)?);
    }

    Ok(families)
}

pub fn family(module: &dyn Module, settings: &Settings) -> Result<Family, String> {
    module.check(settings)?;
    let collector = match module.collector(settings) {
        Ok(collector) => collector,
        Err(why) => Box::new(Unported::new(module.name(), why)),
    };

    Ok(Family {
        collector,
        rules: module.rules(settings),
    })
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

    struct Elsewhere {
        refuses_its_settings: bool,
    }

    impl Module for Elsewhere {
        fn name(&self) -> &'static str {
            "elsewhere"
        }
        fn subject(&self) -> &'static str {
            "a subject read on another system"
        }
        fn every_seconds(&self) -> u32 {
            60
        }
        fn check(&self, _settings: &Settings) -> Result<(), String> {
            match self.refuses_its_settings {
                true => Err("schedule: yesterday is not a period".into()),
                false => Ok(()),
            }
        }
        fn collector(&self, _settings: &Settings) -> Result<Box<dyn Collector>, String> {
            Err("it is read from a file this system does not have".into())
        }
        fn rules(&self, _settings: &Settings) -> RuleSet {
            RuleSet::of(Vec::new())
        }
        fn families(&self) -> &[&'static str] {
            &[]
        }
    }

    fn noon() -> String {
        "2026-09-19T12:00:00.000Z".into()
    }

    #[test]
    fn a_module_that_cannot_read_this_system_starts_the_daemon_as_a_collector_that_says_why() {
        let module = Elsewhere {
            refuses_its_settings: false,
        };

        let family = super::family(&module, &Settings::plain(noon)).expect("the daemon starts");

        assert_eq!(family.collector.name(), "elsewhere");
        match family.collector.available() {
            Health::Unavailable(why) => assert!(
                why.contains("a file this system does not have"),
                "the reason is the module's own: {why}"
            ),
            other => panic!(
                "a subject nothing here can read is unavailable, and a daemon that stopped \
                 for it would watch none of the others: {other:?}"
            ),
        }
    }

    #[test]
    fn settings_the_module_refuses_still_stop_the_daemon_on_every_system() {
        let module = Elsewhere {
            refuses_its_settings: true,
        };

        let refused = super::family(&module, &Settings::plain(noon))
            .err()
            .expect("a file the operator got wrong is not a subject this system lacks");

        assert!(refused.contains("yesterday"), "{refused}");
    }

    #[test]
    fn every_module_this_build_has_is_asked_for_its_collector_on_this_system() {
        let families = families(&Config::default()).expect("the default starts on every system");

        assert_eq!(
            families
                .iter()
                .map(|family| family.collector.name())
                .collect::<Vec<_>>(),
            crate::modules::names()
        );
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
