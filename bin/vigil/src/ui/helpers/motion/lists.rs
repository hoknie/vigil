use std::collections::BTreeMap;

use super::along::Along;
use super::panes::Panes;
use crate::ui::{Program, Startup, System, sections};

#[derive(Debug, Clone)]
pub struct Lists {
    panes: BTreeMap<&'static str, Panes>,
    pub programs: Along<Program>,
    pub startup: Along<Startup>,
    pub system: Along<System>,
}

impl Default for Lists {
    fn default() -> Self {
        Lists {
            panes: sections()
                .iter()
                .map(|section| (section.name(), Panes::of(section.panes().len())))
                .collect(),
            programs: Along::default(),
            startup: Along::default(),
            system: Along::default(),
        }
    }
}

impl Lists {
    pub fn of(&self, section: &str) -> Option<&Panes> {
        self.panes.get(section)
    }

    pub fn of_mut(&mut self, section: &str) -> Option<&mut Panes> {
        self.panes.get_mut(section)
    }
}
