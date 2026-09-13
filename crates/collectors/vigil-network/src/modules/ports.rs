use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::listening_port_rules;
use crate::views::Listening;

const FAMILIES: &[&str] = &["port.listen"];

pub struct Ports;

impl Module for Ports {
    fn name(&self) -> &'static str {
        "ports"
    }

    fn subject(&self) -> &'static str {
        "the sockets this host listens on, and the process holding each one"
    }

    fn every_seconds(&self) -> u32 {
        30
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        elsewhere(settings)
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        listening_port_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(Listening))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }
}

#[cfg(target_os = "linux")]
fn elsewhere(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Ok(Box::new(crate::PortsCollector::new(settings.now())))
}

#[cfg(not(target_os = "linux"))]
fn elsewhere(_settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Err("the sockets of this host are read from a Linux /proc".to_string())
}
