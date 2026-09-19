use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::helpers::FAMILY;
use crate::rules::engine_rules;
use crate::types::{DUMP_SECONDS, WRITTEN_BY, Watching};
use crate::views::WhatTheEnginesHold;

const FAMILIES: &[&str] = &[FAMILY];

pub struct Engines;

impl Module for Engines {
    fn name(&self) -> &'static str {
        crate::parsers::SOURCE
    }

    fn subject(&self) -> &'static str {
        "the images, volumes, networks, projects and secrets the container engines of this host hold"
    }

    fn every_seconds(&self) -> u32 {
        DUMP_SECONDS
    }

    fn unit(&self) -> Option<&'static str> {
        Some(WRITTEN_BY)
    }

    fn settings_key(&self) -> Option<&'static str> {
        Some("containers-engines")
    }

    fn check(&self, settings: &Settings) -> Result<(), String> {
        watching(settings)?.check()
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, settings: &Settings) -> RuleSet {
        engine_rules(&watching(settings).unwrap_or_default().report)
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(WhatTheEnginesHold))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }
}

fn watching(settings: &Settings) -> Result<Watching, String> {
    settings.read().map_err(|refusal| refusal.to_string())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let watching = watching(settings)?;
    watching.check()?;

    Ok(Box::new(crate::EnginesCollector::new(
        settings.now(),
        watching,
    )))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let watching = watching(settings)?;

    Ok(Box::new(crate::EnginesCollector::new(
        settings.now(),
        watching,
    )))
}
