use crate::ui::Spot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aim {
    Spot(Spot),
    Choice(usize),
    Option(usize),
    Popup,
}
