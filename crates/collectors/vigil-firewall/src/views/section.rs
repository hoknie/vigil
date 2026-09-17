use vigil_view::{Pane, Section};

use super::panes::{TheGroups, TheInterfaces, TheRuleset};

pub struct WhatTheHostLetsIn;

impl Section for WhatTheHostLetsIn {
    fn name(&self) -> &'static str {
        "firewall"
    }

    fn title(&self) -> &'static str {
        "What the host lets in"
    }

    fn holds(&self) -> &'static str {
        "what the host lets in"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        vec![
            Box::new(TheRuleset),
            Box::new(TheGroups),
            Box::new(TheInterfaces),
        ]
    }

    fn shows_what_has_gone(&self) -> bool {
        true
    }
}
