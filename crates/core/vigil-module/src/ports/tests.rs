use vigil_collect::{CollectError, Collector, Health};
use vigil_model::Snapshot;
use vigil_rules::RuleSet;

use super::*;
use crate::types::Settings;

struct Nothing;

impl Collector for Nothing {
    fn name(&self) -> &'static str {
        "network"
    }
    fn available(&self) -> Health {
        Health::Ok
    }
    fn collect(&self) -> Result<Snapshot, CollectError> {
        Err(CollectError::Absent("a module under test".into()))
    }
}

struct Network;

impl Module for Network {
    fn name(&self) -> &'static str {
        "network"
    }
    fn subject(&self) -> &'static str {
        "the sockets this host listens on"
    }
    fn every_seconds(&self) -> u32 {
        30
    }
    fn collector(&self, _settings: &Settings) -> Result<Box<dyn Collector>, String> {
        Ok(Box::new(Nothing))
    }
    fn rules(&self, _settings: &Settings) -> RuleSet {
        RuleSet::of(Vec::new())
    }
    fn families(&self) -> &[&'static str] {
        &["port.listen"]
    }
}

#[test]
fn a_finding_of_this_module_walks_to_the_row_of_its_own_reading() {
    assert_eq!(
        Network.row_of("port.listen|tcp|0.0.0.0:4444"),
        Some("tcp|0.0.0.0:4444".to_string()),
        "the key after the family is the collector's own, so the row on the screen is found \
         by it without a table in the console"
    );
}

#[test]
fn a_finding_raised_by_somebody_else_is_not_claimed_and_not_walked_to() {
    assert!(!Network.raised("user|account|backdoor"));
    assert_eq!(Network.row_of("user|account|backdoor"), None);
    assert_eq!(Network.row_of("nothing-shaped-like-a-key"), None);
}

#[test]
fn a_module_that_shows_nothing_is_a_module_and_not_a_half_written_one() {
    assert!(
        Network.section().is_none(),
        "a module reads a host; a screen for it is something it may also have, and a daemon \
         that refuses to run one without a screen would refuse to run a reading that has no \
         table in it"
    );
    assert_eq!(Network.unit(), None);
    assert_eq!(Network.settings_key(), None);
}

#[test]
fn a_module_that_says_nothing_about_it_is_read_once_at_start_up_and_never_while_running() {
    assert!(
        !Network.follows_the_file(),
        "a key the daemon picks up while it runs is a key an edit reaches without anybody \
         restarting anything, and that is a decision a module makes by name rather than one \
         it inherits by saying nothing"
    );
}
