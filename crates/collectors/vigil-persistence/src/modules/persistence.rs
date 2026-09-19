use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::persistence_rules;
use crate::views::WhatStartsByItself;

const FAMILIES: &[&str] = &["persistence"];

#[cfg(not(target_os = "macos"))]
const SUBJECT: &str = "what the host starts by itself: units, timers, cron, shell profiles";

#[cfg(target_os = "macos")]
const SUBJECT: &str = "what the host starts by itself: launchd jobs, cron, shell profiles";

pub struct Persistence;

impl Module for Persistence {
    fn name(&self) -> &'static str {
        "persistence"
    }

    fn subject(&self) -> &'static str {
        SUBJECT
    }

    fn every_seconds(&self) -> u32 {
        300
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        persistence_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(WhatStartsByItself))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }
}

fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Ok(Box::new(crate::PersistenceCollector::new(settings.now())))
}
