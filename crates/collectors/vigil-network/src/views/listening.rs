use vigil_view::{Pane, Section};

use super::by_program::ByProgram;
use super::flat::Flat;

pub struct Listening;

impl Section for Listening {
    fn name(&self) -> &'static str {
        "network"
    }

    fn title(&self) -> &'static str {
        "What is listening"
    }

    fn holds(&self) -> &'static str {
        "what is listening"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        vec![Box::new(Flat), Box::new(ByProgram)]
    }
}
