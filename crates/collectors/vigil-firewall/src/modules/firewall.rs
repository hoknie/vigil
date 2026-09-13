use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::firewall_rules;
use crate::views::WhatTheHostLetsIn;

const FAMILIES: &[&str] = &["firewall"];

pub struct Firewall;

impl Module for Firewall {
    fn name(&self) -> &'static str {
        "firewall"
    }

    fn subject(&self) -> &'static str {
        "the rules by which this host lets network in and drops it"
    }

    fn every_seconds(&self) -> u32 {
        60
    }

    fn unit(&self) -> Option<&'static str> {
        Some("vigil-firewall.timer")
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        firewall_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(WhatTheHostLetsIn))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }

    fn row_of(&self, finding_key: &str) -> Option<String> {
        let (family, rest) = finding_key.split_once('|')?;
        match family == FAMILIES[0] {
            true => Some(format!("fw-{rest}")),
            false => None,
        }
    }
}

#[cfg(target_os = "linux")]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Ok(Box::new(crate::FirewallCollector::new(settings.now())))
}

#[cfg(not(target_os = "linux"))]
fn reading(_settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Err("the ruleset of this host is read from what a Linux nft writes down".to_string())
}
