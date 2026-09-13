use super::standing::Standing;
use crate::ui::{Group, Screen};

pub struct Row {
    pub opens: Screen,
    pub opens_reading: Option<String>,
    pub number: Option<u8>,
    pub name: String,
    pub holds: String,
    pub collector: String,
    pub standing: Standing,
}

impl Row {
    pub fn group(&self) -> Group {
        self.opens.group()
    }

    pub fn is_a_section_of_its_own(&self) -> bool {
        self.opens_reading.is_none()
    }
}
