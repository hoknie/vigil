use super::room::Room;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Fixed(u16),
    Least(u16),
    Share(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub header: &'static str,
    pub width: Width,
    pub wanted: u16,
}

impl Column {
    pub fn new(header: &'static str, width: Width) -> Column {
        Column {
            header,
            width,
            wanted: 0,
        }
    }

    pub fn shown_from(self, columns: u16) -> Column {
        Column {
            wanted: columns,
            ..self
        }
    }

    pub fn fits(&self, room: Room) -> bool {
        room.holds(self.wanted)
    }
}

pub fn fitting(columns: Vec<Column>, room: Room) -> Vec<Column> {
    columns
        .into_iter()
        .filter(|column| column.fits(room))
        .collect()
}
