use vigil_view::{Pane, Section};

use super::pane::Of;
use crate::types::{Engine, List};

pub struct WhatTheEnginesHold;

impl Section for WhatTheEnginesHold {
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
        Engine::ALL
            .into_iter()
            .flat_map(|engine| {
                List::of(engine)
                    .into_iter()
                    .map(move |list| Box::new(Of::new(engine, list)) as Box<dyn Pane>)
            })
            .collect()
    }

    fn groups(&self) -> Vec<&'static str> {
        Engine::ALL.into_iter().map(Engine::name).collect()
    }
}
