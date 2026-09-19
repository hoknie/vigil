use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::process_rules;
use crate::views::WhatHasRunHere;

const FAMILIES: &[&str] = &["process"];

pub struct Processes;

impl Module for Processes {
    fn name(&self) -> &'static str {
        "processes"
    }

    fn subject(&self) -> &'static str {
        "the programs running on this host"
    }

    fn every_seconds(&self) -> u32 {
        30
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        Ok(Box::new(crate::ProcessesCollector::new(settings.now())))
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        process_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(WhatHasRunHere))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }
}
