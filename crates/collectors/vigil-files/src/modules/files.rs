use serde::Deserialize;
use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::file_rules;
use crate::views::TheHostAndItsFiles;

pub const CEILING_BYTES: u64 = 1024 * 1024;

pub const WATCHED_BY_DEFAULT: &[&str] = &[
    "/etc/ssh/sshd_config",
    "/etc/pam.d/sshd",
    "/etc/pam.d/su",
    "/etc/nsswitch.conf",
    "/etc/login.defs",
    "/etc/hosts",
];

const FAMILIES: &[&str] = &["file", "directory"];

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Watching {
    pub paths: Vec<String>,
    pub ceiling_bytes: u64,
}

impl Default for Watching {
    fn default() -> Watching {
        Watching {
            paths: WATCHED_BY_DEFAULT
                .iter()
                .map(|path| (*path).to_string())
                .collect(),
            ceiling_bytes: CEILING_BYTES,
        }
    }
}

impl Watching {
    pub fn check(&self) -> Result<(), String> {
        if self.ceiling_bytes == 0 {
            return Err(
                "ceiling_bytes: 0 hashes nothing, and a file nobody hashes is a file nobody \
                 watches"
                    .to_string(),
            );
        }
        for path in &self.paths {
            if !path.starts_with('/') {
                return Err(format!(
                    "paths: {path:?} is not an absolute path, and this agent reads no working \
                     directory of its own"
                ));
            }
        }

        Ok(())
    }
}

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

#[cfg(target_os = "linux")]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let watching: Watching = settings.read().map_err(|refusal| refusal.to_string())?;

    Ok(Box::new(crate::FilesCollector::new(
        settings.now(),
        &watching.paths,
        watching.ceiling_bytes,
    )))
}

#[cfg(not(target_os = "linux"))]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    let _: Watching = settings.read().map_err(|refusal| refusal.to_string())?;

    Err("the files this host is configured by are read with a Linux stat and a hash".to_string())
}
