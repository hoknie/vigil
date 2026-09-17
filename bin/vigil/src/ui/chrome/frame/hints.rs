use vigil_model::{Changing, ControlTarget};

use crate::ui::Level;

#[derive(Debug, Clone, Copy, Default)]
pub struct Hints<'a> {
    pub editing: bool,
    pub listing: bool,
    pub history: bool,
    pub histories: bool,
    pub graph: bool,
    pub graphs: bool,
    pub kills: bool,
    pub controls: Option<ControlTarget>,
    pub changes: &'static [Changing],
    pub typing: bool,
    pub asking: Option<&'a str>,
    pub level: Level,
    pub message: Option<&'a str>,
    pub back: Back,
    pub panel: bool,
    pub panes: bool,
    pub choosing: bool,
    pub choosing_acts: bool,
    pub sorts: bool,
    pub filters: bool,
    pub marks: bool,
    pub stops: bool,
    pub buttons: bool,
    pub arranges: Option<char>,
    pub to_object: bool,
    pub mouse: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Back {
    #[default]
    MainScreen,
    Finding,
    Nowhere,
}

impl Back {
    pub fn named(self) -> &'static str {
        match self {
            Back::MainScreen => "back to the main screen",
            Back::Finding => "back to the finding",
            Back::Nowhere => "nothing above this",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_with_two_addresses_names_the_one_it_has_now() {
        assert_ne!(Back::MainScreen.named(), Back::Finding.named());
        assert!(Back::Finding.named().contains("finding"));
    }
}
