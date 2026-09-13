use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::account_rules;
use crate::views::WhoCanLogIn;

const FAMILIES: &[&str] = &["user"];

pub struct Users;

impl Module for Users {
    fn name(&self) -> &'static str {
        "users"
    }

    fn subject(&self) -> &'static str {
        "who may log in to this host, as whom, and with what"
    }

    fn every_seconds(&self) -> u32 {
        300
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        reading(settings)
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        account_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(WhoCanLogIn))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }
}

#[cfg(target_os = "linux")]
fn reading(settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Ok(Box::new(crate::UsersCollector::new(settings.now())))
}

#[cfg(not(target_os = "linux"))]
fn reading(_settings: &Settings) -> Result<Box<dyn Collector>, String> {
    Err("who may log in is read from a Linux /etc and its login records".to_string())
}
