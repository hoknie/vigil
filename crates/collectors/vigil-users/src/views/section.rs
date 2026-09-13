use vigil_view::{Pane, Section};

use super::pane::Of;
use crate::types::Subject;

pub struct WhoCanLogIn;

impl Section for WhoCanLogIn {
    fn name(&self) -> &'static str {
        "accounts"
    }

    fn title(&self) -> &'static str {
        "Who can log in"
    }

    fn holds(&self) -> &'static str {
        "who can log in"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        Subject::ALL
            .iter()
            .map(|subject| Box::new(Of(*subject)) as Box<dyn Pane>)
            .collect()
    }
}
