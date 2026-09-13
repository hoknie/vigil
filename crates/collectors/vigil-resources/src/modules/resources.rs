use serde::Deserialize;
use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::{
    CLOCK_SKEW_SECONDS, DISK_FREE_PERCENT, INODE_FREE_PERCENT, ResourceLimits, resource_rules,
};
use crate::views::TheHostAndItsFiles;

const FAMILIES: &[&str] = &["resource"];

const BOOT_ROW: &str = "boot|current";

const FILESYSTEM_ROW: &str = "fs";

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Thresholds {
    pub clock_skew_seconds: u32,
    pub disk_free_percent: u32,
    pub inode_free_percent: u32,
}

impl Default for Thresholds {
    fn default() -> Thresholds {
        Thresholds {
            clock_skew_seconds: CLOCK_SKEW_SECONDS,
            disk_free_percent: DISK_FREE_PERCENT,
            inode_free_percent: INODE_FREE_PERCENT,
        }
    }
}

impl Thresholds {
    pub fn limits(&self) -> ResourceLimits {
        ResourceLimits {
            clock_skew_seconds: i64::from(self.clock_skew_seconds),
            disk_free_percent: u64::from(self.disk_free_percent),
            inode_free_percent: u64::from(self.inode_free_percent),
        }
    }
}

pub struct Resources;

impl Module for Resources {
    fn name(&self) -> &'static str {
        "resources"
    }

    fn subject(&self) -> &'static str {
        "the boot this host is running, the moment it started and the size of it"
    }

    fn every_seconds(&self) -> u32 {
        60
    }

    fn settings_key(&self) -> Option<&'static str> {
        Some("resources")
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, settings: &Settings) -> RuleSet {
        let thresholds: Thresholds = settings.read().unwrap_or_default();

        resource_rules(thresholds.limits())
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(TheHostAndItsFiles))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }

    fn row_of(&self, finding_key: &str) -> Option<String> {
        let (family, rest) = finding_key.split_once('|')?;
        if family != FAMILIES[0] {
            return None;
        }

        Some(match rest.split_once('|') {
            Some((_, mount)) => format!("{FILESYSTEM_ROW}|{mount}"),
            None => BOOT_ROW.to_string(),
        })
    }
}

#[cfg(target_os = "linux")]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let _: Thresholds = settings.read().map_err(|refusal| refusal.to_string())?;

    Ok(Box::new(crate::ResourcesCollector::new(settings.now())))
}

#[cfg(not(target_os = "linux"))]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let _: Thresholds = settings.read().map_err(|refusal| refusal.to_string())?;

    Err("what this host runs on is read from a Linux /proc".to_string())
}
