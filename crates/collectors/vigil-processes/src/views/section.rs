use vigil_view::{Pane, Section};

use super::pane::Running;

pub struct WhatHasRunHere;

impl Section for WhatHasRunHere {
    fn name(&self) -> &'static str {
        "programs"
    }

    fn title(&self) -> &'static str {
        "What has run here"
    }

    fn holds(&self) -> &'static str {
        "what has run here"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        vec![Box::new(Running)]
    }
}
