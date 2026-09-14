use vigil_view::{Pane, Section};

use super::plain::Plain;
use crate::ui::{Screen, View};

pub const TITLE: &str = "Readings this console has no screen for";

pub const HOLDS: &str = "what this build cannot draw";

pub struct Unknown {
    readings: Vec<String>,
}

impl Unknown {
    pub fn of(view: &View) -> Unknown {
        Unknown {
            readings: unknown_readings(view),
        }
    }
}

impl Section for Unknown {
    fn name(&self) -> &'static str {
        "unknown"
    }

    fn title(&self) -> &'static str {
        TITLE
    }

    fn holds(&self) -> &'static str {
        HOLDS
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        self.readings
            .iter()
            .map(|collector| Box::new(Plain::of(collector)) as Box<dyn Pane>)
            .collect()
    }
}

pub fn unknown_readings(view: &View) -> Vec<String> {
    let Some(status) = &view.status else {
        return Vec::new();
    };

    status
        .agent
        .collectors
        .iter()
        .map(|collector| collector.name.clone())
        .filter(|name| Screen::showing(name).is_none())
        .collect()
}
