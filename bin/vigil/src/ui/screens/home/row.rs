use super::standing::Standing;
use crate::ui::{Group, Screen};

pub struct Row {
    pub opens: Option<Screen>,
    pub number: Option<u8>,
    pub name: String,
    pub holds: String,
    pub collector: String,
    pub standing: Standing,
}

impl Row {
    pub fn group(&self) -> Group {
        match self.opens {
            Some(screen) => screen.group(),
            None => Group::Reads,
        }
    }
}
