use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::file_rules;
use crate::types::{Layout, Watching};
use crate::views::TheHostAndItsFiles;

const FAMILIES: &[&str] = &["file", "directory"];

pub struct Files;

impl Module for Files {
    fn name(&self) -> &'static str {
        "files"
    }

    fn subject(&self) -> &'static str {
        "the files this host is configured by, and whether any of them changed"
    }

    fn every_seconds(&self) -> u32 {
        300
    }

    fn settings_key(&self) -> Option<&'static str> {
        Some("files")
    }

    fn follows_the_file(&self) -> bool {
        true
    }

    fn check(&self, settings: &Settings) -> Result<(), String> {
        let watching: Watching = settings.read().map_err(|refusal| refusal.to_string())?;

        watching.check()
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        file_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(TheHostAndItsFiles))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }

    fn row_of(&self, finding_key: &str) -> Option<String> {
        let (family, _) = finding_key.split_once('|')?;
        match FAMILIES.contains(&family) {
            true => Some(finding_key.to_string()),
            false => None,
        }
    }
}

fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let watching: Watching = settings.read().map_err(|refusal| refusal.to_string())?;

    Ok(Box::new(match watching.layout() {
        Layout::Named(named) => crate::FilesCollector::new(settings.now(), &named.hashed()),
        Layout::Listed(listing) => crate::FilesCollector::listed(settings.now(), listing),
    }))
}
