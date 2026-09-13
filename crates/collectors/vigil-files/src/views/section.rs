use vigil_view::{Pane, Section};

use super::pane::WatchedFiles;

pub struct TheHostAndItsFiles;

impl Section for TheHostAndItsFiles {
    fn name(&self) -> &'static str {
        "system"
    }

    fn title(&self) -> &'static str {
        "The host and the files watched on it"
    }

    fn holds(&self) -> &'static str {
        "the host and its files"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        vec![Box::new(WatchedFiles)]
    }
}
