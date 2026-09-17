use vigil_collect::Collector;
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::helpers::family_of;
use crate::types::Settings;

pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;

    fn subject(&self) -> &'static str;

    fn every_seconds(&self) -> u32;

    fn unit(&self) -> Option<&'static str> {
        None
    }

    fn settings_key(&self) -> Option<&'static str> {
        None
    }

    fn follows_the_file(&self) -> bool {
        false
    }

    fn check(&self, _settings: &Settings) -> Result<(), String> {
        Ok(())
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String>;

    fn rules(&self, settings: &Settings) -> RuleSet;

    fn section(&self) -> Option<Box<dyn Section>> {
        None
    }

    fn families(&self) -> &[&'static str];

    fn row_of(&self, finding_key: &str) -> Option<String> {
        let (family, rest) = finding_key.split_once('|')?;
        match self.families().contains(&family) {
            true => Some(rest.to_string()),
            false => None,
        }
    }

    fn raised(&self, finding_key: &str) -> bool {
        family_of(finding_key).is_some_and(|family| self.families().contains(&family))
    }
}
