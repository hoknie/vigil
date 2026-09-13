use vigil_view::{Pane, Section};

use super::pane::Of;
use crate::types::List;

pub struct WhatStartsByItself;

impl Section for WhatStartsByItself {
    fn name(&self) -> &'static str {
        "startup"
    }

    fn title(&self) -> &'static str {
        "What starts by itself"
    }

    fn holds(&self) -> &'static str {
        "what starts by itself"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        List::ALL
            .iter()
            .map(|list| Box::new(Of(*list)) as Box<dyn Pane>)
            .collect()
    }
}
