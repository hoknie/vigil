use vigil_view::{Pane, Section};

use super::pane::Contained;

pub struct WhatRunsInContainers;

impl Section for WhatRunsInContainers {
    fn name(&self) -> &'static str {
        "containers"
    }

    fn title(&self) -> &'static str {
        "What is running in containers"
    }

    fn holds(&self) -> &'static str {
        "what runs in containers"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        vec![Box::new(Contained)]
    }
}
