use super::{Audience, Look, Palette};

pub fn monochrome() -> Palette {
    Palette::decide(Some("1"), Some("dumb"))
}

pub fn look() -> Look {
    Look::new(monochrome(), Audience::Person)
}
