use crate::ui::Screen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opening {
    pub screen: Screen,
    pub difference: bool,
}
