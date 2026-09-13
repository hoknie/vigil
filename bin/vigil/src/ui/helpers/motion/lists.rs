use std::collections::BTreeMap;

use super::panes::Panes;
use crate::ui::sections;

#[derive(Debug, Clone)]
pub struct Lists {
    panes: BTreeMap<&'static str, Panes>,
}

impl Default for Lists {
    fn default() -> Self {
        Lists {
            panes: sections()
                .iter()
                .map(|section| (section.name(), Panes::of(section.panes().len())))
                .collect(),
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
