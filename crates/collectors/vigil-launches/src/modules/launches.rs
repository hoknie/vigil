use serde::Deserialize;
use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::launch_rules;
use crate::views::WhatHasRunHere;

const FAMILIES: &[&str] = &["run", "agent.buffer"];

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Watching {
    pub record_arguments: bool,
}

pub struct Launches;

impl Module for Launches {
    fn name(&self) -> &'static str {
        "launches"
    }

    fn subject(&self) -> &'static str {
        "what people run, from the kernel's audit records"
    }

    fn every_seconds(&self) -> u32 {
        15
    }

    fn settings_key(&self) -> Option<&'static str> {
        Some("launches")
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        launch_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(WhatHasRunHere))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }

    fn row_of(&self, finding_key: &str) -> Option<String> {
        let (family, rest) = finding_key.split_once('|')?;
        match family {
            "run" => Some(finding_key.to_string()),
            "agent.buffer" if rest == "launches" => Some("launches|dropping".to_string()),
            _ => None,
        }
    }
}

#[cfg(target_os = "linux")]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let watching: Watching = settings.read().map_err(|refusal| refusal.to_string())?;

    Ok(Box::new(crate::LaunchesCollector::new(
        settings.now(),
        watching.record_arguments,
    )))
}

#[cfg(not(target_os = "linux"))]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let _: Watching = settings.read().map_err(|refusal| refusal.to_string())?;

    Err("what people run is read from the records a Linux auditd writes".to_string())
}
