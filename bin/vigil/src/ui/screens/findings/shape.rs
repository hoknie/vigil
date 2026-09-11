const ROOM_FOR_THE_KIND: u16 = 72;

const ROOM_FOR_THE_OBJECT: u16 = 118;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shape {
    Cramped,
    Plain,
    Roomy,
}

impl Shape {
    pub(super) fn of(width: u16) -> Shape {
        match width {
            width if width >= ROOM_FOR_THE_OBJECT => Shape::Roomy,
            width if width >= ROOM_FOR_THE_KIND => Shape::Plain,
            _ => Shape::Cramped,
        }
    }
}
