use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::container_rules;
use crate::views::WhatRunsInContainers;

const FAMILIES: &[&str] = &["container"];

pub struct Containers;

impl Module for Containers {
    fn name(&self) -> &'static str {
        "containers"
    }

    fn subject(&self) -> &'static str {
        "the containers running on this host, what they may do and what of this host they hold"
    }

    fn every_seconds(&self) -> u32 {
        60
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        container_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(WhatRunsInContainers))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }

    fn row_of(&self, finding_key: &str) -> Option<String> {
        let (family, rest) = finding_key.split_once('|')?;
        match family == FAMILIES[0] {
            true => Some(
                rest.split_once('|')
                    .map(|(_, named)| named.to_string())
                    .unwrap_or_else(|| rest.to_string()),
            ),
            false => None,
        }
    }
}

fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Ok(Box::new(crate::ContainersCollector::new(settings.now())))
}
